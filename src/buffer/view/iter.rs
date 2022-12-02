use super::ImgBufViewMut;
use crate::{
    pixel::Pixel,
    prelude::Rect,
    util::{dimension_to_usize, index_point},
    Dimension, Point,
};
#[cfg(feature = "unstable")]
use std::iter::TrustedLen;
use std::{iter::FusedIterator, marker::PhantomData, ptr::NonNull};

/// Iterator over the pixels of a [`ImgBufViewMut`] and their relative coordinates.
#[derive(Debug, Clone)]
pub struct PixelsWithCoordsMut<'buffer_ref, P> {
    ptr: NonNull<P>,
    buffer_width: Dimension,
    bounds: Rect,
    current_x: Dimension,
    current_y: Dimension,
    _phantom: PhantomData<&'buffer_ref mut [P]>,
}

impl<'buffer_ref, P> PixelsWithCoordsMut<'buffer_ref, P> {
    #[inline]
    pub fn new<'view_ref>(view: &'view_ref mut ImgBufViewMut<'buffer_ref, P>) -> Self {
        Self {
            ptr: view.ptr,
            buffer_width: view.buffer_width,
            bounds: view.bounds,
            current_x: 0,
            current_y: 0,
            _phantom: PhantomData,
        }
    }
}

impl<'buffer_ref, P> Iterator for PixelsWithCoordsMut<'buffer_ref, P>
where
    P: Pixel,
{
    type Item = (Point, &'buffer_ref mut P);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let view_coords = (self.current_x, self.current_y);
        if !self.bounds.contains_relative(view_coords) {
            return None;
        }

        let buffer_coords = self.bounds.abs_point_from_relative(view_coords);
        let current_index = index_point(buffer_coords, self.buffer_width);

        // SAFETY: this is safe because we already assured the coordinate is in bounds
        // which implies a valid index
        let p = unsafe {
            let ptr = self.ptr.as_ptr().add(current_index);
            ptr.as_mut()
        }
        .map(|p| (view_coords, p));

        self.current_x += 1;
        if self.current_x >= self.bounds.dimensions().0 {
            self.current_x = 0;
            self.current_y += 1;
        }

        p
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let total_size = dimension_to_usize(self.bounds.len());
        let current_size = total_size
            .checked_sub(index_point(
                (self.current_x, self.current_y),
                self.buffer_width,
            ))
            .expect("size shouldn't underflow");
        (current_size, Some(current_size))
    }

    #[inline]
    #[cfg(feature = "unstable")]
    fn advance_by(&mut self, n: usize) -> Result<(), usize> {
        self.current_x +=
            Dimension::try_from(n).expect("shouldn't advance iterator by more than Dimension::MAX");
        self.current_y += self.current_x / self.buffer_width;
        self.current_x %= self.buffer_width;
        Ok(())
    }
}

#[rustfmt::skip]
impl<'buffer_ref, P> ExactSizeIterator for PixelsWithCoordsMut<'buffer_ref, P> where P: Pixel {}
#[rustfmt::skip]
impl<'buffer_ref, P> FusedIterator for PixelsWithCoordsMut<'buffer_ref, P> where P: Pixel {}
#[rustfmt::skip]
#[cfg(feature = "unstable")]
// SAFETY: this is safe because, well, we know the contract is upheld.
// there's a test for this: with_coords_mut_trusted_len.
unsafe impl<'buffer_ref, P> TrustedLen for PixelsWithCoordsMut<'buffer_ref, P> where P: Pixel {}

/// Iterator over the pixels of a [`ImgBufViewMut`].
pub struct PixelsMut<'view_ref, P>(PixelsWithCoordsMut<'view_ref, P>);

impl<'buffer_ref, P> PixelsMut<'buffer_ref, P> {
    #[inline]
    pub fn new<'view_ref>(view: &'view_ref mut ImgBufViewMut<'buffer_ref, P>) -> Self {
        Self(PixelsWithCoordsMut::new(view))
    }
}

impl<'buffer_ref, P> Iterator for PixelsMut<'buffer_ref, P>
where
    P: Pixel,
{
    type Item = &'buffer_ref mut P;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, p)| p)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    #[inline]
    #[cfg(feature = "unstable")]
    fn advance_by(&mut self, n: usize) -> Result<(), usize> {
        self.0.advance_by(n)
    }
}

#[rustfmt::skip]
impl<'buffer_ref, P> ExactSizeIterator for PixelsMut<'buffer_ref, P> where P: Pixel {}
#[rustfmt::skip]
impl<'buffer_ref, P> FusedIterator for PixelsMut<'buffer_ref, P> where P: Pixel {}
#[rustfmt::skip]
#[cfg(feature = "unstable")]
// SAFETY: this is safe because, well, we know the contract is upheld.
// there's a test for this: mut_trusted_len.
unsafe impl<'buffer_ref, P> TrustedLen for PixelsMut<'buffer_ref, P> where P: Pixel {}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn with_coords_mut_trusted_len(width in 1..256u32, height in 1..256u32) {
            let mut buffer = Rgb8Img::new(width, height);
            let mut view = buffer.view_mut(buffer.bounds()).unwrap();
            let mut iter = view.pixels_with_coords_mut();

            let (lower, higher) = iter.size_hint();
            assert_eq!(lower, higher.unwrap());

            for _ in 0..lower {
                assert!(iter.next().is_some());
            }

            assert!(iter.next().is_none());
        }

        #[test]
        fn mut_trusted_len(width in 1..256u32, height in 1..256u32) {
            let mut buffer = Rgb8Img::new(width, height);
            let mut view = buffer.view_mut(buffer.bounds()).unwrap();
            let mut iter = view.pixels_mut();

            let (lower, higher) = iter.size_hint();
            assert_eq!(lower, higher.unwrap());

            for _ in 0..lower {
                assert!(iter.next().is_some());
            }

            assert!(iter.next().is_none());
        }
    }
}
