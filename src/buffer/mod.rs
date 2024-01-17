/// Common buffer types.
pub mod common;
/// Buffer related iterators.
pub mod iter;
/// View types of the buffer.
pub mod view;

use crate::{
    pixel::Pixel,
    util::{checked_size, index_point, macros::debug_assertions},
    view::{Img, ImgMut},
    Point, Rect,
};
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};
use view::{ImgBufView, ImgBufViewMut};

/// An image buffer.
///
/// `P` is it's pixel type and `C` it's container type.
#[derive(Debug, Clone)]
pub struct ImgBuf<P, C = Vec<P>> {
    width: u32,
    height: u32,
    data: C,
    _phantom: PhantomData<P>,
}

impl<P> ImgBuf<P> {
    /// Creates a new [`ImgBuf`] with the specified `width` and `height` from a function that generates pixels.
    pub fn from_fn<F>(width: u32, height: u32, f: F) -> Self
    where
        F: FnMut(Point) -> P,
    {
        let container: Vec<_> = (0..height)
            .flat_map(|y| (0..width).map(move |x| (x, y)))
            .map(f)
            .collect();

        Self::from_container(container, width, height)
    }
}

impl<P> ImgBuf<P>
where
    P: Clone,
{
    /// Creates a new [`ImgBuf`] with the specified `width` and `height` and [`Vec`] as it's container type.
    #[inline]
    pub fn new(width: u32, height: u32) -> Self
    where
        P: Default,
    {
        Self {
            width,
            height,
            data: vec![
                P::default();
                (width as usize)
                    .checked_mul(height as usize)
                    .expect("size fits within usize")
            ],
            _phantom: PhantomData,
        }
    }
}

impl<P, C> ImgBuf<P, C>
where
    C: Deref<Target = [P]>,
{
    /// Returns a reference to the inner container of pixels.
    #[inline]
    pub fn container(&self) -> &C {
        &self.data
    }

    /// Consume this image buffer and return it's container.
    #[inline]
    pub fn into_container(self) -> C {
        self.data
    }

    /// Create an image buffer from a width, a height and a container with data.
    ///
    /// # Panics
    /// Panics if `container.len() != width * height`.
    #[inline(always)]
    pub fn from_container(container: C, width: u32, height: u32) -> Self {
        assert_eq!(container.len(), checked_size(width, height));
        Self {
            width,
            height,
            data: container,
            _phantom: PhantomData,
        }
    }

    /// Converts this image buffer into another by applying a mapping function to each
    /// of it's pixels.
    pub fn map<P2, C2, F>(self, f: F) -> ImgBuf<P2, C2>
    where
        C: IntoIterator<Item = P>,
        C2: Deref<Target = [P2]> + FromIterator<P2>,
        F: FnMut(P) -> P2,
    {
        <ImgBuf<P2, C2>>::from_container(
            self.data.into_iter().map(f).collect(),
            self.width,
            self.height,
        )
    }

    /// Converts this image buffer into another with [`Vec`] as it's container by applying
    /// a mapping function to each of it's pixels.
    pub fn map_vec<P2, F>(self, f: F) -> ImgBuf<P2, Vec<P2>>
    where
        C: IntoIterator<Item = P>,
        F: FnMut(P) -> P2,
    {
        <ImgBuf<P2, Vec<P2>>>::from_container(
            self.data.into_iter().map(f).collect(),
            self.width,
            self.height,
        )
    }

    /// Returns a slice containing the pixels of this buffer in row-major (top-left to bottom-right) order.
    #[inline]
    pub fn as_pixel_slice(&self) -> &[P] {
        &self.data
    }
}

impl<P, C> ImgBuf<P, C>
where
    P: Pixel,
    C: Deref<Target = [P]>,
{
    /// Returns an iterator over the pixels and coordinates of this buffer.
    #[inline]
    pub fn pixels_with_coords(&self) -> iter::PixelsWithCoords<'_, P> {
        iter::PixelsWithCoords::new(self)
    }
}

impl<P, C> ImgBuf<P, C>
where
    C: DerefMut<Target = [P]>,
{
    /// Returns a mutable slice containing the pixels of this buffer in row-major (top-left to bottom-right) order.
    #[inline]
    pub fn as_mut_pixel_slice(&mut self) -> &mut [P] {
        &mut self.data
    }

    /// Returns a mutable pointer to the first pixel of the image. All remaining pixels are subsequent in a row-major
    /// (top-left to bottom-right) order.
    ///
    /// The returned pointer may be _dangling_, but it won't be _null_.
    #[inline]
    pub fn as_mut_ptr(&mut self) -> NonNull<P> {
        NonNull::new(self.data.as_mut_ptr()).expect("slice reference is always non-null")
    }
}

impl<P, C> ImgBuf<P, C>
where
    P: Pixel,
    C: DerefMut<Target = [P]>,
{
    /// Copies an image buffer into this one.
    ///
    /// # Panics
    /// Panics if `self.dimensions() != buffer.dimensions()`
    #[inline]
    pub fn copy_from_buffer(&mut self, buffer: &Self)
    where
        P: Copy,
    {
        assert_eq!(self.dimensions(), buffer.dimensions());
        self.as_mut_pixel_slice()
            .copy_from_slice(buffer.as_pixel_slice());
    }

    /// Returns a mutable iterator over the pixels and coordinates of this buffer.
    #[inline]
    pub fn pixels_with_coords_mut(&mut self) -> iter::PixelsWithCoordsMut<'_, P> {
        iter::PixelsWithCoordsMut::new(self)
    }
}

impl<P, C> Img for ImgBuf<P, C>
where
    P: Pixel,
    C: Deref<Target = [P]>,
{
    type Pixel = P;
    type Pixels<'buffer_ref> = iter::Pixels<'buffer_ref, Self::Pixel>
    where
        Self: 'buffer_ref;

    type View<'buffer_ref> = ImgBufView<'buffer_ref, Self::Pixel>
    where
        Self::Pixel: 'buffer_ref,
        C: 'buffer_ref;

    #[inline]
    fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    fn height(&self) -> u32 {
        self.height
    }

    #[inline]
    fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    #[inline]
    fn pixel(&self, coords: Point) -> Option<&Self::Pixel> {
        self.data.get(index_point(coords, self.width))
    }

    #[inline]
    unsafe fn pixel_unchecked(&self, coords: Point) -> &Self::Pixel {
        debug_assertions! {
            on => self.data.get(index_point(coords, self.width)).unwrap(),
            off => self.data.get_unchecked(index_point(coords, self.width))
        }
    }

    #[inline]
    fn pixels(&self) -> Self::Pixels<'_> {
        self.as_pixel_slice().iter()
    }

    #[inline]
    unsafe fn view_unchecked(&self, bounds: Rect) -> Self::View<'_> {
        debug_assert!(self.bounds().contains_rect(&bounds));
        view::ImgBufView::new(self, bounds)
    }
}

impl<P, C> ImgMut for ImgBuf<P, C>
where
    P: Pixel,
    C: DerefMut<Target = [P]>,
{
    type PixelsMut<'buffer_ref> = iter::PixelsMut<'buffer_ref, Self::Pixel>
    where
        Self: 'buffer_ref;

    type ViewMut<'buffer_ref> = ImgBufViewMut<'buffer_ref, Self::Pixel>
    where
        Self::Pixel: 'buffer_ref, C: 'buffer_ref;

    #[inline]
    fn pixel_mut(&mut self, coords: Point) -> Option<&mut Self::Pixel> {
        self.data.get_mut(index_point(coords, self.width))
    }

    #[inline]
    unsafe fn pixel_mut_unchecked(&mut self, coords: Point) -> &mut Self::Pixel {
        debug_assertions! {
            on => self.data.get_mut(index_point(coords, self.width)).unwrap(),
            off => self.data.get_unchecked_mut(index_point(coords, self.width))
        }
    }

    #[inline]
    fn pixels_mut(&mut self) -> Self::PixelsMut<'_> {
        self.as_mut_pixel_slice().iter_mut()
    }

    #[inline]
    unsafe fn view_mut_unchecked(&mut self, bounds: Rect) -> Self::ViewMut<'_> {
        debug_assert!(self.bounds().contains_rect(&bounds));
        view::ImgBufViewMut::new(self, bounds)
    }

    unsafe fn view_mut_multiple_unchecked<const N: usize>(
        &mut self,
        bounds: [Rect; N],
    ) -> [Self::ViewMut<'_>; N] {
        let ptr = self.as_mut_ptr();

        // SAFETY: we trust the caller!
        bounds.map(|b| unsafe { view::ImgBufViewMut::from_ptr(ptr, self.width, b) })
    }

    fn split_x_at_mut(&mut self, mid: u32) -> Option<(Self::ViewMut<'_>, Self::ViewMut<'_>)> {
        let left_bounds = Rect::new((0, 0), (mid, self.height));
        let right_bounds = Rect::new((mid, 0), (self.width - mid, self.height));
        let ptr = self.as_mut_ptr();

        let left = self
            .bounds()
            .contains_rect(&left_bounds)
            // SAFETY: safe because 'left_bounds' is checked to be contained within the buffer.
            .then(|| unsafe { view::ImgBufViewMut::from_ptr(ptr, self.width, left_bounds) });
        let right = self
            .bounds()
            .contains_rect(&right_bounds)
            // SAFETY: safe because 'right_bounds' is checked to be contained within the buffer.
            .then(|| unsafe { view::ImgBufViewMut::from_ptr(ptr, self.width, right_bounds) });

        left.and_then(|left| right.map(|right| (left, right)))
    }

    fn split_y_at_mut(&mut self, mid: u32) -> Option<(Self::ViewMut<'_>, Self::ViewMut<'_>)> {
        let upper_bounds = Rect::new((0, 0), (self.width, mid));
        let lower_bounds = Rect::new((0, mid), (self.width, self.height - mid));
        let ptr = self.as_mut_ptr();

        let upper = self
            .bounds()
            .contains_rect(&upper_bounds)
            // SAFETY: safe because 'upper_bounds' is checked to be contained within the buffer.
            .then(|| unsafe { view::ImgBufViewMut::from_ptr(ptr, self.width, upper_bounds) });
        let lower = self
            .bounds()
            .contains_rect(&lower_bounds)
            // SAFETY: safe because 'lower_bounds' is checked to be contained within the buffer.
            .then(|| unsafe { view::ImgBufViewMut::from_ptr(ptr, self.width, lower_bounds) });

        upper.and_then(|upper| lower.map(|lower| (upper, lower)))
    }
}

impl<P, C> IntoIterator for ImgBuf<P, C>
where
    P: Pixel,
    C: IntoIterator<Item = P>,
{
    type Item = P;

    type IntoIter = C::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<'view_ref, P, C> IntoIterator for &'view_ref ImgBuf<P, C>
where
    P: Pixel,
    C: Deref<Target = [P]>,
{
    type Item = &'view_ref P;

    type IntoIter = <ImgBuf<P, C> as Img>::Pixels<'view_ref>;

    fn into_iter(self) -> Self::IntoIter {
        self.pixels()
    }
}

impl<'view_ref, P, C> IntoIterator for &'view_ref mut ImgBuf<P, C>
where
    P: Pixel,
    C: DerefMut<Target = [P]>,
{
    type Item = &'view_ref mut P;

    type IntoIter = <ImgBuf<P, C> as ImgMut>::PixelsMut<'view_ref>;

    fn into_iter(self) -> Self::IntoIter {
        self.pixels_mut()
    }
}

#[cfg(test)]
impl<P> proptest::arbitrary::Arbitrary for ImgBuf<P>
where
    P: Pixel + Default + Clone + std::fmt::Debug,
{
    type Parameters = ();
    type Strategy = proptest::strategy::BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        use proptest::prelude::*;
        const ALLOWED_DIMENSION_RANGE: std::ops::Range<u32> = 0..4096;

        let dimensions_strat = (ALLOWED_DIMENSION_RANGE, ALLOWED_DIMENSION_RANGE);

        let raw_strat = dimensions_strat.prop_map(|(width, height)| {
            (
                (width, height),
                proptest::collection::vec(
                    proptest::strategy::Just(P::default()),
                    (width * height) as usize,
                ),
            )
        });

        let buf_strat = raw_strat.prop_flat_map(|((width, height), container_strat)| {
            container_strat
                .prop_map(move |container| ImgBuf::from_container(container, width, height))
        });

        buf_strat.boxed()
    }
}
