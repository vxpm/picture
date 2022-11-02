use super::ImageBuffer;
use crate::{
    pixel::Pixel,
    view::{ImageView, ImageViewMut},
    Dimension, Point,
};
#[cfg(feature = "unstable")]
use std::iter::TrustedLen;
use std::{
    iter::FusedIterator,
    ops::{Deref, DerefMut},
};

/// Iterator over the pixels of a [`ImageBuffer`].
pub type Pixels<'buffer_ref, P> = std::slice::Iter<'buffer_ref, P>;
/// Mutable iterator over the pixels of a [`ImageBuffer`].
pub type PixelsMut<'buffer_ref, P> = std::slice::IterMut<'buffer_ref, P>;

// this will result in a compile-error if either of these isn't TrustedLen.
// needed because iterators in this module implement TrustedLen based on the
// assumption that these >are< TrustedLen.
#[cfg(feature = "unstable")]
trait EnsureTrustedLen: TrustedLen {}
#[cfg(feature = "unstable")]
impl<'buffer_ref, P> EnsureTrustedLen for Pixels<'buffer_ref, P> {}
#[cfg(feature = "unstable")]
impl<'buffer_ref, P> EnsureTrustedLen for PixelsMut<'buffer_ref, P> {}

/// Iterator over the pixels of a [`ImageBuffer`] with their respective coordinates.
#[derive(Clone)]
pub struct PixelsWithCoords<'buffer_ref, P> {
    pixels: Pixels<'buffer_ref, P>,
    current_x: Dimension,
    current_y: Dimension,
    buffer_width: Dimension,
}

impl<'buffer_ref, P> PixelsWithCoords<'buffer_ref, P>
where
    P: Pixel,
{
    #[inline]
    pub fn new<C>(buffer: &'buffer_ref ImageBuffer<P, C>) -> Self
    where
        C: Deref<Target = [P]>,
    {
        Self {
            pixels: buffer.pixels(),
            current_x: 0,
            current_y: 0,
            buffer_width: buffer.width(),
        }
    }
}

impl<'buffer_ref, P> Iterator for PixelsWithCoords<'buffer_ref, P> {
    type Item = (Point, &'buffer_ref P);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let ret = self
            .pixels
            .next()
            .map(|p| ((self.current_x, self.current_y), p));

        self.current_x += 1;
        if self.current_x >= self.buffer_width {
            self.current_x = 0;
            self.current_y += 1;
        }

        ret
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.pixels.size_hint()
    }

    #[inline]
    #[cfg(feature = "unstable")]
    fn advance_by(&mut self, n: usize) -> Result<(), usize> {
        self.pixels.advance_by(n)?;

        self.current_x +=
            Dimension::try_from(n).expect("shouldn't advance iterator by more than Dimension::MAX");
        self.current_y += self.current_x / self.buffer_width;
        self.current_x %= self.buffer_width;

        Ok(())
    }
}

impl<'buffer_ref, P> ExactSizeIterator for PixelsWithCoords<'buffer_ref, P> {}
impl<'buffer_ref, P> FusedIterator for PixelsWithCoords<'buffer_ref, P> {}
#[cfg(feature = "unstable")]
// SAFETY: since PixelsWithCoords is just a wrapper around Pixels, and Pixels
// implements TrustedLen, PixelsWithCoords can be TrustedLen as well!
unsafe impl<'buffer_ref, P> TrustedLen for PixelsWithCoords<'buffer_ref, P> {}

/// Mutable iterator over the pixels of a [`ImageBuffer`] with their respective coordinates.
pub struct PixelsWithCoordsMut<'buffer_ref, P> {
    pixels: PixelsMut<'buffer_ref, P>,
    current_x: Dimension,
    current_y: Dimension,
    buffer_width: Dimension,
}

impl<'buffer_ref, P> PixelsWithCoordsMut<'buffer_ref, P>
where
    P: Pixel,
{
    #[inline]
    pub fn new<C>(buffer: &'buffer_ref mut ImageBuffer<P, C>) -> Self
    where
        C: DerefMut<Target = [P]>,
    {
        Self {
            buffer_width: buffer.width(),
            pixels: buffer.pixels_mut(),
            current_x: 0,
            current_y: 0,
        }
    }
}

impl<'buffer_ref, P> Iterator for PixelsWithCoordsMut<'buffer_ref, P> {
    type Item = (Point, &'buffer_ref mut P);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let ret = self
            .pixels
            .next()
            .map(|p| ((self.current_x, self.current_y), p));

        self.current_x += 1;
        if self.current_x >= self.buffer_width {
            self.current_x = 0;
            self.current_y += 1;
        }

        ret
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.pixels.size_hint()
    }

    #[inline]
    #[cfg(feature = "unstable")]
    fn advance_by(&mut self, n: usize) -> Result<(), usize> {
        self.pixels.advance_by(n)?;

        self.current_x +=
            Dimension::try_from(n).expect("shouldn't advance iterator by more than Dimension::MAX");
        self.current_y += self.current_x / self.buffer_width;
        self.current_x %= self.buffer_width;

        Ok(())
    }
}

impl<'buffer_ref, P> ExactSizeIterator for PixelsWithCoordsMut<'buffer_ref, P> {}
impl<'buffer_ref, P> FusedIterator for PixelsWithCoordsMut<'buffer_ref, P> {}
#[cfg(feature = "unstable")]
// SAFETY: since PixelsWithCoordsMut is just a wrapper around PixelsMut, and PixelsMut
// implements TrustedLen, PixelsWithCoordsMut can be TrustedLen as well!
unsafe impl<'buffer_ref, P> TrustedLen for PixelsWithCoordsMut<'buffer_ref, P> {}
