use super::{BlockOps, Img};
use crate::{
    pixel::Pixel,
    prelude::Dimension,
    util::{dimension_to_usize, index_point},
    view::ImgCore,
    Point,
};
use std::{iter::FusedIterator, marker::PhantomData};

/// Iterator over the pixels of an [`ImgView`] and their relative coordinates.
#[derive(Debug, Clone)]
pub struct PixelsWithCoords<'view_ref, V>
where
    V: ImgCore,
{
    view: &'view_ref V,
    current_x: Dimension,
    current_y: Dimension,
}

impl<'view_ref, V> PixelsWithCoords<'view_ref, V>
where
    V: ImgCore,
{
    #[inline]
    pub fn new(view: &'view_ref V) -> Self {
        Self {
            view,
            current_x: 0,
            current_y: 0,
        }
    }
}

impl<'view_ref, V> Iterator for PixelsWithCoords<'view_ref, V>
where
    V: ImgCore,
{
    type Item = (Point, &'view_ref V::Pixel);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let current_coords = (self.current_x, self.current_y);
        let p = self.view.pixel(current_coords).map(|p| (current_coords, p));

        self.current_x += 1;
        if self.current_x >= self.view.width() {
            self.current_x = 0;
            self.current_y += 1;
        }

        p
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let total_size = dimension_to_usize(self.view.size());
        let current_size = total_size
            .checked_sub(index_point(
                (self.current_x, self.current_y),
                self.view.width(),
            ))
            .expect("size shouldn't underflow");
        (current_size, Some(current_size))
    }

    #[inline]
    #[cfg(feature = "unstable")]
    fn advance_by(&mut self, n: usize) -> Result<(), usize> {
        self.current_x +=
            Dimension::try_from(n).expect("shouldn't advance iterator by more than Dimension::MAX");
        self.current_y += self.current_x / self.view.width();
        self.current_x %= self.view.width();
        Ok(())
    }
}

impl<'view_ref, V> ExactSizeIterator for PixelsWithCoords<'view_ref, V> where V: ImgCore {}
impl<'view_ref, V> FusedIterator for PixelsWithCoords<'view_ref, V> where V: ImgCore {}

/// Iterator over the pixels of an [`ImgView`].
pub struct Pixels<'view_ref, V>(PixelsWithCoords<'view_ref, V>)
where
    V: ImgCore;

impl<'view_ref, V> Pixels<'view_ref, V>
where
    V: ImgCore,
{
    #[inline]
    pub fn new(view: &'view_ref V) -> Self {
        Self(PixelsWithCoords::new(view))
    }
}

impl<'view_ref, V> Iterator for Pixels<'view_ref, V>
where
    V: ImgCore,
{
    type Item = &'view_ref V::Pixel;

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

impl<'view_ref, V> ExactSizeIterator for Pixels<'view_ref, V> where V: ImgCore {}
impl<'view_ref, V> FusedIterator for Pixels<'view_ref, V> where V: ImgCore {}

// TODO: better specify the behaviour of this iterator
/// Iterator over block views of an [`ImgView`] and their relative coordinates.
///
/// While most blocks will have the specified dimensions, some might be smaller due to the original view dimensions
/// not being exactly divisible.
#[derive(Debug, Clone)]
pub struct Blocks<'view_ref, P, V> {
    view: &'view_ref V,
    current_x: Dimension,
    current_y: Dimension,
    block_dimensions: (Dimension, Dimension),
    _phantom: PhantomData<&'view_ref [P]>,
}

impl<'view_ref, P, V> Blocks<'view_ref, P, V>
where
    P: Pixel,
    V: ImgCore<Pixel = P>,
{
    #[inline]
    pub fn new(view: &'view_ref V, block_dimensions: (Dimension, Dimension)) -> Self {
        Self {
            view,
            current_x: 0,
            current_y: 0,
            block_dimensions,
            _phantom: PhantomData,
        }
    }
}

impl<'view_ref, P, V> Iterator for Blocks<'view_ref, P, V>
where
    P: Pixel,
    V: Img<Pixel = P>,
{
    type Item = (Point, V::View<'view_ref>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let current_coords = (self.current_x, self.current_y);

        let v = self
            .view
            .block(current_coords, self.block_dimensions)
            .map(|p| (current_coords, p));

        self.current_x += 1;
        if self.current_x >= self.view.width_in_blocks(self.block_dimensions.0) {
            self.current_x = 0;
            self.current_y += 1;
        }

        v
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // can be made into an exact hint but i'm too lazy
        let size = dimension_to_usize(self.view.width_in_blocks(self.block_dimensions.0))
            .checked_mul(dimension_to_usize(
                self.view.height_in_blocks(self.block_dimensions.1),
            ))
            .expect("size shouldn't overflow");

        (0, Some(size))
    }
}

#[rustfmt::skip]
impl<'view_ref, P, V> FusedIterator for Blocks<'view_ref, P, V> where P: Pixel, V: Img<Pixel = P>, {}
