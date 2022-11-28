/// Default iterator types.
pub mod iter;

use crate::{buffer::ImageBuffer, pixel::Pixel, prelude::Dimension, util::Rect, Point};

// types: ImageBuffer, ImageBufferView, ImageBufferViewMut
//
// Trait         | Implemented for...
// ImageView    -> ImageBuffer, ImageBufferView, ImageBufferViewMut
// ImageViewMut -> ImageBuffer, ImageBufferViewMut

/// Trait for types that can be treated as a view into some image.
pub trait ImageView {
    /// The pixel type of this view.
    type Pixel: Pixel;

    /// The type of the iterator through pixels of this view.
    type Pixels<'view_ref>: Iterator<Item = &'view_ref Self::Pixel>
    where
        Self: 'view_ref;

    /// The view type the `view` method returns.
    type View<'view_ref>: ImageView<Pixel = Self::Pixel>
    where
        Self: 'view_ref;

    /// The width of this view.
    fn width(&self) -> Dimension;
    /// The height of this view.
    fn height(&self) -> Dimension;

    /// The size, in pixels, of this view. Equivalent to `width * height`.
    #[inline]
    fn size(&self) -> Dimension {
        self.width() * self.height()
    }

    /// The dimensions of this view. Equivalent to `(width, height)`.
    #[inline]
    fn dimensions(&self) -> (Dimension, Dimension) {
        (self.width(), self.height())
    }

    /// Returns a [`Rect`] with top-left point `(0, 0)` and dimensions `self.dimensions()`.
    #[inline]
    fn bounds(&self) -> Rect {
        Rect::new((0, 0), self.dimensions())
    }

    /// Returns a reference to the pixel with coordinates `(x, y)` relative to this view. If the coordinates
    /// aren't within the bounds of this view, returns `None`.
    #[inline]
    fn pixel(&self, coords: Point) -> Option<&Self::Pixel> {
        self.bounds()
            .contains_relative(coords)
            // SAFETY: safe because the pixel is checked to be in bounds
            .then(|| unsafe { self.pixel_unchecked(coords) })
    }

    /// Returns a reference to the pixel with coordinates `(x, y)` relative to this view, without checking.
    ///
    /// # Safety
    /// The coordinate must be in the bounds of the view.
    unsafe fn pixel_unchecked(&self, coords: Point) -> &Self::Pixel;

    /// Returns an iterator over the pixels of this view.
    fn pixels(&self) -> Self::Pixels<'_>;

    /// Returns a view into this view. If the bounds don't fit in this view, returns `None`.
    #[inline]
    fn view(&self, bounds: Rect) -> Option<Self::View<'_>> {
        self.bounds()
            .contains_rect(&bounds)
            // SAFETY: safe because 'bounds' is checked to be contained within the view.
            .then(|| unsafe { self.view_unchecked(bounds) })
    }

    /// Returns a view into this view, without checking bounds.
    ///
    /// # Safety
    /// The bounds must fit in this view.
    unsafe fn view_unchecked(&self, bounds: Rect) -> Self::View<'_>;

    /// Returns multiple views into this view. If any of the bounds don't fit in this view, returns `None`.
    fn view_multiple<const N: usize>(&self, bounds: [Rect; N]) -> Option<[Self::View<'_>; N]> {
        let result: Option<arrayvec::ArrayVec<Self::View<'_>, N>> =
            bounds.into_iter().map(|b| self.view(b)).collect();

        result.map(|inner| {
            inner
                .into_inner()
                .ok()
                .expect("Inner result array and bounds array should have the same length")
        })
    }

    /// Returns multiple views into this view, without checking bounds.
    ///
    /// # Safety
    /// All bounds must fit in this view.
    fn view_multiple_unchecked<const N: usize>(&self, bounds: [Rect; N]) -> [Self::View<'_>; N] {
        let result: arrayvec::ArrayVec<Self::View<'_>, N> = bounds
            .into_iter()
            // SAFETY: we trust the caller!
            .map(|b| unsafe { self.view_unchecked(b) })
            .collect();

        debug_assert_eq!(result.len(), result.capacity());
        // SAFETY: safe because the 'bounds' array and the inner array of the 'result'
        // have the same length: N
        unsafe { result.into_inner_unchecked() }
    }

    /// Splits this view into two disjoint views, separated at the given x coordinate.
    #[inline]
    fn split_x_at(&self, mid: Dimension) -> Option<(Self::View<'_>, Self::View<'_>)> {
        let left_bounds = Rect::new((0, 0), (mid, self.height()));
        let right_bounds = Rect::new((mid, 0), (self.width() - mid, self.height()));

        self.view(left_bounds)
            .and_then(|left| self.view(right_bounds).map(|right| (left, right)))
    }

    /// Splits this view into two disjoint views, separated at the given y coordinate.
    #[inline]
    fn split_y_at(&self, mid: Dimension) -> Option<(Self::View<'_>, Self::View<'_>)> {
        let upper_bounds = Rect::new((0, 0), (self.width(), mid));
        let lower_bounds = Rect::new((0, mid), (self.width(), self.height() - mid));

        self.view(upper_bounds)
            .and_then(|upper| self.view(lower_bounds).map(|lower| (upper, lower)))
    }

    /// Writes the data of each pixel to a [writer][std::io::Write] in a row-major (top-left to bottom-right)
    /// order.
    #[inline]
    fn write_data<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        for pixel in self.pixels() {
            pixel.write_data(&mut writer)?;
        }

        Ok(())
    }

    /// Creates an [`ImageBuffer`] from this view with [`Vec`] as it's container.
    #[inline]
    fn to_buffer(&self) -> ImageBuffer<Self::Pixel, Vec<Self::Pixel>>
    where
        Self::Pixel: Clone,
    {
        // SAFETY: the coordinates are always going to be in bounds since the
        // new buffer and self have the same dimensions
        ImageBuffer::from_fn(self.width(), self.height(), |(x, y)| unsafe {
            self.pixel_unchecked((x, y)).clone()
        })
    }
}

/// Trait for types that can be treated as a mutable view into some image.
pub trait ImageViewMut: ImageView {
    /// The type of the iterator through mutable pixels of this view.
    type PixelsMut<'view_ref>: Iterator<Item = &'view_ref mut Self::Pixel>
    where
        Self: 'view_ref;

    /// The mutable view type the `view_mut` method returns.
    type ViewMut<'view_ref>: ImageViewMut<Pixel = Self::Pixel>
    where
        Self: 'view_ref;

    /// Returns a mutable reference to the pixel with coordinates `(x, y)` relative to this view. If the
    /// coordinates aren't within the bounds of this view, returns `None`.
    #[inline]
    fn pixel_mut(&mut self, coords: Point) -> Option<&mut Self::Pixel> {
        self.bounds()
            .contains(coords)
            // SAFETY: safe because the pixel is checked to be in bounds
            .then(|| unsafe { self.pixel_mut_unchecked(coords) })
    }

    /// Returns a mutable reference to the pixel with coordinates `(x, y)` relative to this view, without
    /// checking.
    ///
    /// # Safety
    /// The coordinate must be in the bounds of the view.
    unsafe fn pixel_mut_unchecked(&mut self, coords: Point) -> &mut Self::Pixel;

    /// Returns a mutable iterator over the pixels of this view.
    fn pixels_mut(&mut self) -> Self::PixelsMut<'_>;

    /// Returns a mutable view into this view. If the bounds don't fit in this view, returns `None`.
    #[inline]
    fn view_mut(&mut self, bounds: Rect) -> Option<Self::ViewMut<'_>> {
        self.bounds()
            .contains_rect(&bounds)
            // SAFETY: safe because 'bounds' is checked to be contained within the view.
            .then(|| unsafe { self.view_mut_unchecked(bounds) })
    }

    /// Returns a mutable view into this view, without checking.
    ///
    /// # Safety
    /// The bounds must fit in this view.
    unsafe fn view_mut_unchecked(&mut self, bounds: Rect) -> Self::ViewMut<'_>;

    /// Returns multiple mutable views into this view. If any of the bounds don't fit in this view or
    /// overlap, returns `None`.
    fn view_mut_multiple<const N: usize>(
        &mut self,
        bounds: [Rect; N],
    ) -> Option<[Self::ViewMut<'_>; N]> {
        for (index, bound_a) in bounds.iter().enumerate() {
            if !self.bounds().contains_rect(bound_a) {
                return None;
            }

            for bound_b in &bounds[index + 1..] {
                if bound_a.overlaps(bound_b) {
                    return None;
                }
            }
        }

        Some(self.view_mut_multiple_unchecked(bounds))
    }

    /// Returns multiple mutable views into this view, without checking bounds and overlaps.
    ///
    /// # Safety
    /// All bounds must fit in this view and not overlap with each other.
    fn view_mut_multiple_unchecked<const N: usize>(
        &mut self,
        bounds: [Rect; N],
    ) -> [Self::ViewMut<'_>; N];

    /// Splits this mutable view into two disjoint mutable views, separated at the given x coordinate.
    fn split_x_at_mut(&mut self, mid: Dimension) -> Option<(Self::ViewMut<'_>, Self::ViewMut<'_>)>;

    /// Splits this mutable view into two disjoint mutable views, separated at the given y coordinate.
    fn split_y_at_mut(&mut self, mid: Dimension) -> Option<(Self::ViewMut<'_>, Self::ViewMut<'_>)>;

    /// Copies a view into this one.
    ///
    /// # Panics
    /// Panics if `self.dimensions() != view.dimensions()`
    #[inline]
    fn copy_from<I>(&mut self, view: &I)
    where
        I: ImageView<Pixel = Self::Pixel>,
        Self::Pixel: Clone,
    {
        assert_eq!(self.dimensions(), view.dimensions());
        self.pixels_mut()
            .zip(view.pixels().cloned())
            .for_each(|(a, b)| *a = b);
    }

    /// Swaps the contents of this view with another one.
    ///
    /// # Panics
    /// Panics if `self.dimensions() != view.dimensions()`
    #[inline]
    fn swap_with<I>(&mut self, view: &mut I)
    where
        I: ImageViewMut<Pixel = Self::Pixel>,
    {
        assert_eq!(self.dimensions(), view.dimensions());
        self.pixels_mut()
            .zip(view.pixels_mut())
            .for_each(|(a, b)| std::mem::swap(a, b));
    }
}
