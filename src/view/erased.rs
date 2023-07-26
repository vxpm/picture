use super::{Img, ImgCore};
use crate::{pixel::Pixel, prelude::Dimension, util::Rect, Point};

/// A subtrait of [`ImgCore`] that adds the same functionality as [`Img`],
/// but in an object-safe manner through type erasure.
pub trait ErasedImg: ImgCore {
    /// Returns an iterator over the pixels of this view.
    fn pixels_erased<'view_ref>(
        &'view_ref self,
    ) -> Box<dyn Iterator<Item = &<Self as ImgCore>::Pixel> + 'view_ref>;

    /// Returns a view into this view. If the bounds don't fit in this view, returns `None`.
    #[inline]
    fn view_erased<'view_ref>(
        &'view_ref self,
        bounds: Rect,
    ) -> Option<Box<dyn ErasedImg<Pixel = <Self as ImgCore>::Pixel> + 'view_ref>> {
        self.bounds()
            .contains_rect(&bounds)
            // SAFETY: safe because 'bounds' is checked to be contained within the view.
            .then(|| unsafe { self.view_unchecked_erased(bounds) })
    }

    /// Returns a view into this view, without checking bounds.
    ///
    /// # Safety
    /// The bounds must fit in this view.
    unsafe fn view_unchecked_erased<'view_ref>(
        &'view_ref self,
        bounds: Rect,
    ) -> Box<dyn ErasedImg<Pixel = <Self as ImgCore>::Pixel> + 'view_ref>;

    /// Returns multiple views into this view. If any of the bounds don't fit in this view, returns `None`.
    fn view_multiple_erased<'view_ref>(
        &'view_ref self,
        bounds: &[Rect],
    ) -> Option<Vec<Box<dyn ErasedImg<Pixel = <Self as ImgCore>::Pixel> + 'view_ref>>> {
        bounds.into_iter().map(|b| self.view_erased(*b)).collect()
    }

    /// Returns multiple views into this view, without checking bounds.
    ///
    /// # Safety
    /// All bounds must fit in this view.
    unsafe fn view_multiple_unchecked_erased<'view_ref>(
        &'view_ref self,
        bounds: &[Rect],
    ) -> Vec<Box<dyn ErasedImg<Pixel = <Self as ImgCore>::Pixel> + 'view_ref>> {
        bounds
            .into_iter()
            // SAFETY: we trust the caller!
            .map(|b| unsafe { self.view_unchecked_erased(*b) })
            .collect()
    }

    /// Splits this view into two disjoint views, separated at the given x coordinate.
    #[inline]
    fn split_x_at_erased<'view_ref>(
        &'view_ref self,
        mid: Dimension,
    ) -> Option<(
        Box<dyn ErasedImg<Pixel = <Self as ImgCore>::Pixel> + 'view_ref>,
        Box<dyn ErasedImg<Pixel = <Self as ImgCore>::Pixel> + 'view_ref>,
    )> {
        let left_bounds = Rect::new((0, 0), (mid, self.height()));
        let right_bounds = Rect::new((mid, 0), (self.width() - mid, self.height()));

        self.view_erased(left_bounds)
            .and_then(|left| self.view_erased(right_bounds).map(|right| (left, right)))
    }

    /// Splits this view into two disjoint views, separated at the given y coordinate.
    #[inline]
    fn split_y_at_erased<'view_ref>(
        &'view_ref self,
        mid: Dimension,
    ) -> Option<(
        Box<dyn ErasedImg<Pixel = <Self as ImgCore>::Pixel> + 'view_ref>,
        Box<dyn ErasedImg<Pixel = <Self as ImgCore>::Pixel> + 'view_ref>,
    )> {
        let upper_bounds = Rect::new((0, 0), (self.width(), mid));
        let lower_bounds = Rect::new((0, mid), (self.width(), self.height() - mid));

        self.view_erased(upper_bounds)
            .and_then(|upper| self.view_erased(lower_bounds).map(|lower| (upper, lower)))
    }

    /// Writes the data of each pixel to a [writer][std::io::Write] in a row-major (top-left to bottom-right)
    /// order.
    #[inline]
    fn write_data(&self, mut writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        for pixel in self.pixels_erased() {
            pixel.write_data(&mut writer)?;
        }

        Ok(())
    }
}

impl<T> ErasedImg for T
where
    T: Img,
{
    #[inline]
    fn pixels_erased<'view_ref>(
        &'view_ref self,
    ) -> Box<dyn Iterator<Item = &'view_ref <Self as ImgCore>::Pixel> + 'view_ref> {
        Box::new(self.pixels())
    }

    #[inline]
    unsafe fn view_unchecked_erased<'view_ref>(
        &'view_ref self,
        bounds: Rect,
    ) -> Box<dyn ErasedImg<Pixel = <Self as ImgCore>::Pixel> + 'view_ref> {
        Box::new(self.view_unchecked(bounds))
    }
}

impl<P> Img for dyn ErasedImg<Pixel = P>
where
    P: Pixel,
{
    type Pixels<'view_ref> = Box<dyn Iterator<Item = &'view_ref <Self as ImgCore>::Pixel> + 'view_ref>
    where
        Self: 'view_ref;

    type View<'view_ref> = Box<dyn ErasedImg<Pixel = <Self as ImgCore>::Pixel> + 'view_ref>
    where
        Self: 'view_ref;

    fn pixels(&self) -> Self::Pixels<'_> {
        self.pixels_erased()
    }

    unsafe fn view_unchecked(&self, bounds: Rect) -> Self::View<'_> {
        self.view_unchecked_erased(bounds)
    }
}

impl<P> ImgCore for Box<dyn ErasedImg<Pixel = P> + '_>
where
    P: Pixel,
{
    type Pixel = P;

    fn width(&self) -> Dimension {
        (**self).width()
    }

    fn height(&self) -> Dimension {
        (**self).height()
    }

    unsafe fn pixel_unchecked(&self, coords: Point) -> &Self::Pixel {
        (**self).pixel_unchecked(coords)
    }
}

impl<P> Img for Box<dyn ErasedImg<Pixel = P> + '_>
where
    P: Pixel,
{
    type Pixels<'view_ref> = Box<dyn Iterator<Item = &'view_ref <Self as ImgCore>::Pixel> + 'view_ref>
    where
        Self: 'view_ref;

    type View<'view_ref> = Box<dyn ErasedImg<Pixel = <Self as ImgCore>::Pixel> + 'view_ref>
    where
        Self: 'view_ref;

    fn pixels(&self) -> Self::Pixels<'_> {
        self.pixels_erased()
    }

    unsafe fn view_unchecked(&self, bounds: Rect) -> Self::View<'_> {
        self.view_unchecked_erased(bounds)
    }
}
