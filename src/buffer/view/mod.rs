use super::ImgBuf;
use crate::{
    pixel::Pixel,
    util::index_point,
    view::{self, Img, ImgMut},
    Point, Rect,
};
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

pub mod iter;

/// A view into an [`ImgBuf`].
#[derive(Clone)]
pub struct ImgBufView<'buffer_ref, P> {
    ptr: NonNull<P>,
    buffer_width: u32,
    bounds: Rect,
    _phantom: PhantomData<&'buffer_ref [P]>,
}

// SAFETY: safe because 'ImgBufView' acts as a shared reference.
unsafe impl<'buffer_ref, P> Send for ImgBufView<'buffer_ref, P> {}
// SAFETY: see above.
unsafe impl<'buffer_ref, P> Sync for ImgBufView<'buffer_ref, P> {}

impl<'buffer_ref, P> ImgBufView<'buffer_ref, P>
where
    P: Pixel,
{
    /// SAFETY: it's up to the caller to ensure `bounds` is within the buffer
    #[inline]
    pub(super) unsafe fn new<C>(buffer: &'buffer_ref ImgBuf<P, C>, bounds: Rect) -> Self
    where
        C: Deref<Target = [P]>,
    {
        let ptr = NonNull::new(buffer.data.as_ptr().cast_mut())
            .expect("slice reference is always non-null");

        ImgBufView {
            ptr,
            buffer_width: buffer.width,
            bounds,
            _phantom: PhantomData,
        }
    }

    /// Returns an iterator over the pixels and coordinates of this view.
    #[inline]
    pub fn pixels_with_coords(&self) -> view::iter::PixelsWithCoords<'_, Self> {
        view::iter::PixelsWithCoords::new(self)
    }
}

impl<'buffer_ref, P> Img for ImgBufView<'buffer_ref, P>
where
    P: Pixel,
{
    type Pixel = P;
    type Pixels<'self_ref> = view::iter::Pixels<'self_ref, Self>
    where
        Self: 'self_ref;
    type View<'self_ref> = Self
    where
        Self: 'self_ref;

    #[inline]
    fn width(&self) -> u32 {
        self.bounds.dimensions().0
    }

    #[inline]
    fn height(&self) -> u32 {
        self.bounds.dimensions().1
    }

    #[inline]
    fn dimensions(&self) -> (u32, u32) {
        self.bounds.dimensions()
    }

    #[inline]
    unsafe fn pixel_unchecked(&self, coords: Point) -> &Self::Pixel {
        debug_assert!(self.bounds.contains_relative(coords));

        let buffer_coords = self.bounds.abs_point_from_relative(coords);
        let index = index_point(buffer_coords, self.buffer_width);
        let ptr = self.ptr.as_ptr();

        // SAFETY: assuming 'bounds' is a valid rect for this buffer, that is, it's contained within
        // the bounds of the buffer, the relative position being in 'bounds' means that 'index' is within
        // the buffer.
        //
        // returning a shared reference to the pixel in this case is safe because as long as this view is
        // valid we are "borrowing" the buffer, and therefore no mutable reference to this pixel can exist.
        unsafe { ptr.add(index).as_ref().unwrap_unchecked() }
    }

    #[inline]
    fn pixels(&self) -> Self::Pixels<'_> {
        Self::Pixels::new(self)
    }

    // TODO: this can be optimized to iterate over the rows of the view!
    // #[inline]
    // fn pixel_chunks(&self) -> impl Iterator<Item = &'_ [P]> {
    //     todo!()
    // }

    #[inline]
    unsafe fn view_unchecked(&self, bounds: Rect) -> Self::View<'_> {
        debug_assert!(self.bounds.contains_rect_relative(&bounds));
        let bounds = self.bounds.abs_rect_from_relative(bounds);

        ImgBufView {
            ptr: self.ptr,
            buffer_width: self.buffer_width,
            bounds,
            _phantom: PhantomData,
        }
    }
}

impl<'buffer_ref, 'view_ref, P> IntoIterator for &'view_ref ImgBufView<'buffer_ref, P>
where
    P: Pixel,
{
    type Item = &'view_ref P;

    type IntoIter = <ImgBufView<'buffer_ref, P> as Img>::Pixels<'view_ref>;

    fn into_iter(self) -> Self::IntoIter {
        self.pixels()
    }
}

/// A mutable view into an [`ImgBuf`].
pub struct ImgBufViewMut<'buffer_ref, P> {
    ptr: NonNull<P>,
    buffer_width: u32,
    bounds: Rect,
    _phantom: PhantomData<&'buffer_ref mut [P]>,
}

// SAFETY: safe because 'ImgBufViewMut' acts like a mutable reference.
unsafe impl<'buffer_ref, P> Send for ImgBufViewMut<'buffer_ref, P> {}
// SAFETY: see above.
unsafe impl<'buffer_ref, P> Sync for ImgBufViewMut<'buffer_ref, P> {}

impl<'buffer_ref, P> ImgBufViewMut<'buffer_ref, P>
where
    P: Pixel,
{
    /// SAFETY: it's up to the caller to ensure `bounds` is within the buffer
    #[inline]
    pub(super) unsafe fn new<C>(buffer: &'buffer_ref mut ImgBuf<P, C>, bounds: Rect) -> Self
    where
        C: DerefMut<Target = [P]>,
    {
        let ptr = buffer.as_mut_ptr();

        ImgBufViewMut {
            ptr,
            buffer_width: buffer.width,
            bounds,
            _phantom: PhantomData,
        }
    }

    /// SAFETY: it's up to the caller to ensure `bounds` is within the buffer and that
    /// this view does _not_ overlap with any other.
    #[inline]
    pub(super) unsafe fn from_ptr(ptr: NonNull<P>, buffer_width: u32, bounds: Rect) -> Self {
        ImgBufViewMut {
            ptr,
            buffer_width,
            bounds,
            _phantom: PhantomData,
        }
    }

    /// Returns an iterator over the pixels and coordinates of this view.
    #[inline]
    pub fn pixels_with_coords(&self) -> view::iter::PixelsWithCoords<'_, Self> {
        view::iter::PixelsWithCoords::new(self)
    }

    /// Returns a mutable iterator over the pixels and coordinates of this view.
    #[inline]
    pub fn pixels_with_coords_mut(&mut self) -> iter::PixelsWithCoordsMut<'_, P> {
        iter::PixelsWithCoordsMut::new(self)
    }
}

impl<'buffer_ref, P> Img for ImgBufViewMut<'buffer_ref, P>
where
    P: Pixel,
{
    type Pixel = P;
    type Pixels<'self_ref> = view::iter::Pixels<'self_ref, Self>
    where
        Self: 'self_ref;

    type View<'self_ref> = ImgBufView<'self_ref, Self::Pixel>
    where
        Self: 'self_ref;

    #[inline]
    fn width(&self) -> u32 {
        self.bounds.dimensions().0
    }

    #[inline]
    fn height(&self) -> u32 {
        self.bounds.dimensions().1
    }

    #[inline]
    fn dimensions(&self) -> (u32, u32) {
        self.bounds.dimensions()
    }

    #[inline]
    unsafe fn pixel_unchecked(&self, coords: Point) -> &Self::Pixel {
        debug_assert!(self.bounds.contains_relative(coords));

        let buffer_coords = self.bounds.abs_point_from_relative(coords);
        let index = index_point(buffer_coords, self.buffer_width);
        let ptr = self.ptr.as_ptr();

        // SAFETY: assuming 'bounds' is a valid rect for this buffer, that is, it's contained within
        // the bounds of the buffer, the relative position being in 'bounds' means that 'index' is within
        // the buffer.
        //
        // returning a shared reference to the pixel in this case is safe because as long as this view is
        // valid we are mutably "borrowing" the buffer and no mutable reference to this view can exist in
        // order to call this function, which means that there are no mutable references to any of the
        // pixels in this view.
        unsafe { ptr.add(index).as_ref().unwrap_unchecked() }
    }

    #[inline]
    fn pixels(&self) -> Self::Pixels<'_> {
        Self::Pixels::new(self)
    }

    // TODO: this can be optimized to iterate over the rows of the view!
    // #[inline]
    // fn pixel_chunks(&self) -> impl Iterator<Item = &'_ [P]> {
    //     todo!()
    // }

    #[inline]
    unsafe fn view_unchecked(&self, bounds: Rect) -> Self::View<'_> {
        debug_assert!(self.bounds.contains_rect_relative(&bounds));
        let bounds = self.bounds.abs_rect_from_relative(bounds);

        ImgBufView {
            ptr: self.ptr,
            buffer_width: self.buffer_width,
            bounds,
            _phantom: PhantomData,
        }
    }
}

impl<'buffer_ref, P> ImgMut for ImgBufViewMut<'buffer_ref, P>
where
    P: Pixel,
{
    type PixelsMut<'self_ref> = iter::PixelsMut<'self_ref, Self::Pixel>
    where
        Self: 'self_ref;

    type ViewMut<'self_ref> = ImgBufViewMut<'self_ref, Self::Pixel>
    where
        Self: 'self_ref;

    #[inline]
    unsafe fn pixel_mut_unchecked(&mut self, coords: Point) -> &mut Self::Pixel {
        debug_assert!(self.bounds.contains_relative(coords));

        let buffer_coords = self.bounds.abs_point_from_relative(coords);
        let index = index_point(buffer_coords, self.buffer_width);
        let ptr = self.ptr.as_ptr();

        // SAFETY: assuming 'bounds' is a valid rect for this buffer, that is, it's contained within
        // the bounds of the buffer, the relative position being in 'bounds' means that 'index' is within
        // the buffer.
        //
        // returning a mutable reference to the pixel in this case is safe because as long as this view is
        // valid we are mutably "borrowing" the buffer and no mutable reference to this view can exist in
        // order to call this function, which means that there are no mutable references to any of the
        // pixels in this view.
        unsafe { ptr.add(index).as_mut().unwrap_unchecked() }
    }

    #[inline]
    fn pixels_mut(&mut self) -> Self::PixelsMut<'_> {
        Self::PixelsMut::new(self)
    }

    #[inline]
    unsafe fn view_mut_unchecked(&mut self, bounds: Rect) -> Self::ViewMut<'_> {
        debug_assert!(self.bounds.contains_rect_relative(&bounds));
        let bounds = self.bounds.abs_rect_from_relative(bounds);

        // SAFETY: we trust the caller!
        unsafe { ImgBufViewMut::from_ptr(self.ptr, self.buffer_width, bounds) }
    }

    #[inline]
    unsafe fn view_mut_multiple_unchecked<const N: usize>(
        &mut self,
        bounds: [Rect; N],
    ) -> [Self::ViewMut<'_>; N] {
        // SAFETY: we trust the caller!
        bounds.map(|b| unsafe { ImgBufViewMut::from_ptr(self.ptr, self.buffer_width, b) })
    }

    fn split_x_at_mut(&mut self, mid: u32) -> Option<(Self::ViewMut<'_>, Self::ViewMut<'_>)> {
        let left_bounds = Rect::new((0, 0), (mid, self.height()));
        let right_bounds = Rect::new((mid, 0), (self.width() - mid, self.height()));

        let left = self
            .bounds
            .contains_rect_relative(&left_bounds)
            // SAFETY: safe because 'left_bounds' is checked to be contained within the view.
            .then(|| unsafe {
                ImgBufViewMut::from_ptr(
                    self.ptr,
                    self.buffer_width,
                    self.bounds.abs_rect_from_relative(left_bounds),
                )
            });
        let right = self
            .bounds
            .contains_rect_relative(&right_bounds)
            // SAFETY: safe because 'right_bounds' is checked to be contained within the view.
            .then(|| unsafe {
                ImgBufViewMut::from_ptr(
                    self.ptr,
                    self.buffer_width,
                    self.bounds.abs_rect_from_relative(right_bounds),
                )
            });

        left.and_then(|left| right.map(|right| (left, right)))
    }

    fn split_y_at_mut(&mut self, mid: u32) -> Option<(Self::ViewMut<'_>, Self::ViewMut<'_>)> {
        let upper_bounds = Rect::new((0, 0), (self.width(), mid));
        let lower_bounds = Rect::new((0, mid), (self.width(), self.height() - mid));

        let upper = self
            .bounds
            .contains_rect_relative(&upper_bounds)
            // SAFETY: safe because 'upper_bounds' is checked to be contained within the view.
            .then(|| unsafe {
                ImgBufViewMut::from_ptr(
                    self.ptr,
                    self.buffer_width,
                    self.bounds.abs_rect_from_relative(upper_bounds),
                )
            });
        let lower = self
            .bounds
            .contains_rect_relative(&lower_bounds)
            // SAFETY: safe because 'lower_bounds' is checked to be contained within the view.
            .then(|| unsafe {
                ImgBufViewMut::from_ptr(
                    self.ptr,
                    self.buffer_width,
                    self.bounds.abs_rect_from_relative(lower_bounds),
                )
            });

        upper.and_then(|upper| lower.map(|lower| (upper, lower)))
    }
}

impl<'buffer_ref, 'view_ref, P> IntoIterator for &'view_ref ImgBufViewMut<'buffer_ref, P>
where
    P: Pixel,
{
    type Item = &'view_ref P;

    type IntoIter = <ImgBufViewMut<'buffer_ref, P> as Img>::Pixels<'view_ref>;

    fn into_iter(self) -> Self::IntoIter {
        self.pixels()
    }
}

impl<'buffer_ref, 'view_ref, P> IntoIterator for &'view_ref mut ImgBufViewMut<'buffer_ref, P>
where
    P: Pixel,
{
    type Item = &'view_ref mut P;

    type IntoIter = <ImgBufViewMut<'buffer_ref, P> as ImgMut>::PixelsMut<'view_ref>;

    fn into_iter(self) -> Self::IntoIter {
        self.pixels_mut()
    }
}
