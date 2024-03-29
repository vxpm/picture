use crate::{util::index_point, view::Img, Point};
use std::iter::FusedIterator;

/// Iterator over the pixels of an [`Img`] and their relative coordinates.
#[derive(Debug, Clone)]
pub struct PixelsWithCoords<'view_ref, V>
where
    V: Img,
{
    view: &'view_ref V,
    current_x: u32,
    current_y: u32,
}

impl<'view_ref, V> PixelsWithCoords<'view_ref, V>
where
    V: Img,
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
    V: Img,
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
        let total_size = self.view.size();
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
            u32::try_from(n).expect("shouldn't advance iterator by more than u32::MAX");
        self.current_y += self.current_x / self.view.width();
        self.current_x %= self.view.width();
        Ok(())
    }
}

impl<'view_ref, V> ExactSizeIterator for PixelsWithCoords<'view_ref, V> where V: Img {}
impl<'view_ref, V> FusedIterator for PixelsWithCoords<'view_ref, V> where V: Img {}

/// Iterator over the pixels of an [`Img`].
pub struct Pixels<'view_ref, V>(PixelsWithCoords<'view_ref, V>)
where
    V: Img;

impl<'view_ref, V> Pixels<'view_ref, V>
where
    V: Img,
{
    #[inline]
    pub fn new(view: &'view_ref V) -> Self {
        Self(PixelsWithCoords::new(view))
    }
}

impl<'view_ref, V> Iterator for Pixels<'view_ref, V>
where
    V: Img,
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

impl<'view_ref, V> ExactSizeIterator for Pixels<'view_ref, V> where V: Img {}
impl<'view_ref, V> FusedIterator for Pixels<'view_ref, V> where V: Img {}
