use crate::prelude::*;
use crate::util::{checked_size, index_point};

/// Common sampling filters.
pub mod filters;

// TODO: maybe think of a better name?
/// Trait for channel types that can be processed.
pub trait Processable: Copy {
    /// Converts this value to a [`f32`].
    fn to_f32(self) -> f32;

    /// Converts a [`f32`] into [`Self`].
    ///
    /// For numeric types, note that the value _will not_ necessarily be in the
    /// valid range (e.g. it might be 258.2 for a [`u8`]). You should clamp the
    /// value in these cases.
    fn from_f32(value: f32) -> Self;
}

macro_rules! impl_processable {
    ($($type:ty),*) => {
        $(
            impl Processable for $type {
                #[inline(always)]
                fn to_f32(self) -> f32 {
                    self as f32
                }

                #[inline(always)]
                fn from_f32(value: f32) -> Self {
                    value.clamp(Self::MIN as f32, Self::MAX as f32) as Self
                }
            }
        )*
    };
}

impl_processable!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64, usize, isize);

// useful resources:
// - https://entropymine.com/imageworsener
// - https://cs1230.graphics/lectures - specifically image processing I, II and III

/// Resamples a view horizontally to the given width using the given filter.
/// Height is kept the same.
///
/// `window` is the maximum distance a pixel can be to the one being currently
/// processed before being cut out of the filter.
#[must_use = "the resampled buffer is returned and the original view is left unmodified"]
pub fn resample_horizontal<I, P, C, F, const N: usize>(
    view: &I,
    new_width: u32,
    filter: F,
    window: f32,
) -> ImgBuf<P, Vec<P>>
where
    I: Img<Pixel = P>,
    P: Pixel<Channels = [C; N]>,
    C: Processable,
    F: Fn(f32) -> f32,
{
    if new_width == 0 {
        return ImgBuf::from_container(Vec::new(), new_width, view.height());
    }

    // create container for result
    let container_size = checked_size(new_width, view.height());
    let mut container = Vec::with_capacity(container_size);
    let container_pixels = container.spare_capacity_mut();

    // find the ratio between the source width and the target width
    let ratio = view.width() as f32 / new_width as f32;
    let sampling_ratio = ratio.max(1.0);
    let inverse_sampling_ratio = 1.0 / sampling_ratio;

    // if we're upsampling (ratio < 1), there's no need to scale things.
    // however, if we're downsampling (ratio > 1), we need to scale stuff
    // by the ratio so that we can preserve information.
    // sampling_ratio's purpose is exactly that: it's value is 1.0 if
    // upsampling, ratio if downsampling.

    // scale the window accordingly
    let window = window * sampling_ratio;

    // precalculate weights
    let offset_constant = 0.5 * (ratio - 1.0);
    let max_src_x_f32 = (view.width() - 1) as f32;
    let mut weights = Vec::with_capacity((2 * (window as usize) + 1) * (new_width as usize));
    let mut weights_start_index = Vec::with_capacity(new_width as usize);
    for target_x in 0..new_width {
        let equivalent_src_x = target_x as f32 * ratio + offset_constant;

        let left_src_pixel_x = (equivalent_src_x - window).clamp(0.0, max_src_x_f32) as u32;
        let right_src_pixel_x = (equivalent_src_x + window).clamp(0.0, max_src_x_f32) as u32;

        weights_start_index.push(weights.len());
        for src_pixel_x in left_src_pixel_x..(right_src_pixel_x + 1) {
            weights.push(filter(
                (src_pixel_x as f32 - equivalent_src_x) * inverse_sampling_ratio,
            ));
        }
    }

    // now actually resample
    let weights = weights.as_slice();
    for target_x in 0..new_width {
        // these could be cached as well, but it makes no performance difference (and increases
        // memory usage), so we just calculate them again
        let equivalent_src_x = target_x as f32 * ratio + offset_constant;

        let min_src_pixel_x = (equivalent_src_x - window).clamp(0.0, max_src_x_f32) as u32;
        let max_src_pixel_x = (equivalent_src_x + window).clamp(0.0, max_src_x_f32) as u32;

        let weights_start = weights_start_index[target_x as usize];
        for target_y in 0..view.height() {
            let mut weight_sum = 0f32;
            let mut channel_value_sum = [0f32; N];
            for (index, src_pixel_x) in (min_src_pixel_x..max_src_pixel_x + 1).enumerate() {
                // SAFETY: target_y is in the 0..img.height() range and src_pixel_x is clamped
                // between 0 and img.width() - 1. therefore, this coordinate is always in bounds.
                let src_pixel = unsafe { view.pixel_unchecked((src_pixel_x, target_y)) };
                let channels = src_pixel.channels();
                let weight = weights[weights_start + index];
                weight_sum += weight;

                for (sum, channel) in channel_value_sum.iter_mut().zip(channels.iter()) {
                    let value = weight * channel.to_f32();
                    *sum += value;
                }
            }

            let result = channel_value_sum.map(|v| C::from_f32(v / weight_sum));

            // SAFETY: this index will always be valid since target_x and target_y are always in
            // the correct range.
            unsafe {
                container_pixels
                    .get_unchecked_mut(index_point((target_x, target_y), new_width))
                    .write(P::new(result));
            }
        }
    }

    // SAFETY: all pixels have already been initialized in the previous loop.
    unsafe {
        container.set_len(container_size);
    }

    ImgBuf::from_container(container, new_width, view.height())
}

/// Resamples a view vertically to the given height using the given filter.
/// Width is kept the same.
///
/// `window` is the maximum distance a pixel can be to the one being currently
/// processed before being cut out of the filter.
#[must_use = "the resampled buffer is returned and the original view is left unmodified"]
pub fn resample_vertical<I, P, C, F, const N: usize>(
    view: &I,
    new_height: u32,
    filter: F,
    window: f32,
) -> ImgBuf<P, Vec<P>>
where
    I: Img<Pixel = P>,
    P: Pixel<Channels = [C; N]>,
    C: Processable,
    F: Fn(f32) -> f32,
{
    if new_height == 0 {
        return ImgBuf::from_container(Vec::new(), view.width(), new_height);
    }

    // create container for result
    let container_size = checked_size(view.width(), new_height);
    let mut container = Vec::with_capacity(container_size);
    let container_pixels = container.spare_capacity_mut();

    // find the ratio between the source height and the target height
    let ratio = view.height() as f32 / new_height as f32;
    let sampling_ratio = ratio.max(1.0);
    let inverse_sampling_ratio = 1.0 / sampling_ratio;

    // if we're upsampling (ratio < 1), there's no need to scale things.
    // however, if we're downsampling (ratio > 1), we need to scale stuff
    // by the ratio so that we can preserve information.
    // sampling_ratio's purpose is exactly that: it's value is 1.0 if
    // upsampling, ratio if downsampling.

    // scale the window accordingly
    let window = window * sampling_ratio;

    // precalculate weights
    let offset_constant = 0.5 * (ratio - 1.0);
    let max_src_y_f32 = (view.height() - 1) as f32;
    let mut weights = Vec::with_capacity((2 * (window as usize) + 1) * (new_height as usize));
    let mut weights_start_index = Vec::with_capacity(new_height as usize);
    for target_y in 0..new_height {
        let equivalent_src_y = target_y as f32 * ratio + offset_constant;

        let min_src_pixel_y = (equivalent_src_y - window).clamp(0.0, max_src_y_f32) as u32;
        let max_src_pixel_y = (equivalent_src_y + window).clamp(0.0, max_src_y_f32) as u32;

        weights_start_index.push(weights.len());
        for src_pixel_y in min_src_pixel_y..(max_src_pixel_y + 1) {
            weights.push(filter(
                (src_pixel_y as f32 - equivalent_src_y) * inverse_sampling_ratio,
            ));
        }
    }

    // now actually resample
    let weights = weights.as_slice();
    for target_y in 0..new_height {
        // these could be cached as well, but it makes no performance difference (and increases
        // memory usage), so we just calculate them again
        let equivalent_src_y = target_y as f32 * ratio + offset_constant;

        let min_src_pixel_y = (equivalent_src_y - window).clamp(0.0, max_src_y_f32) as u32;
        let max_src_pixel_y = (equivalent_src_y + window).clamp(0.0, max_src_y_f32) as u32;

        let weights_start = weights_start_index[target_y as usize];
        for target_x in 0..view.width() {
            let mut weight_sum = 0f32;
            let mut channel_value_sum = [0f32; N];
            for (index, src_pixel_y) in (min_src_pixel_y..max_src_pixel_y + 1).enumerate() {
                // SAFETY: target_x is in the 0..img.width() range and src_pixel_y is clamped
                // between 0 and img.height() - 1. therefore, this coordinate is always in bounds.
                let src_pixel = unsafe { view.pixel_unchecked((target_x, src_pixel_y)) };
                let channels = src_pixel.channels();
                let weight = weights[weights_start + index];
                weight_sum += weight;

                for (sum, channel) in channel_value_sum.iter_mut().zip(channels.iter()) {
                    let value = weight * channel.to_f32();
                    *sum += value;
                }
            }

            let result = channel_value_sum.map(|v| C::from_f32(v / weight_sum));

            // SAFETY: this index will always be valid since target_x and target_y are always in
            // the correct range.
            unsafe {
                container_pixels
                    .get_unchecked_mut(index_point((target_x, target_y), view.width()))
                    .write(P::new(result));
            }
        }
    }

    // SAFETY: all pixels have already been initialized in the previous loop.
    unsafe {
        container.set_len(container_size);
    }

    ImgBuf::from_container(container, view.width(), new_height)
}

/// Resamples a view to the given dimensions using the given filter. This is
/// equivalent to doing a horizontal resample followed by a vertical one.
///
/// `window` is the maximum distance a pixel can be to the one being currently
/// processed before being cut out of the filter.
#[must_use = "the resampled buffer is returned and the original view is left unmodified"]
pub fn resample<I, P, C, F, const N: usize>(
    view: &I,
    dimensions: (u32, u32),
    filter: F,
    window: f32,
) -> ImgBuf<P, Vec<P>>
where
    I: Img<Pixel = P>,
    P: Pixel<Channels = [C; N]>,
    C: Processable,
    F: Fn(f32) -> f32,
{
    let (width, height) = dimensions;
    let horizontal = resample_horizontal(view, width, &filter, window);
    resample_vertical(&horizontal, height, filter, window)
}

/// Performs a box blur in a view and returns the result.
#[must_use = "the blurred buffer is returned and the original view is left unmodified"]
pub fn box_blur<I, P, C, const N: usize>(view: &I, strength: f32) -> ImgBuf<P, Vec<P>>
where
    I: Img<Pixel = P>,
    P: Pixel<Channels = [C; N]>,
    C: Processable,
{
    assert!(strength > 0.0);
    resample(view, view.dimensions(), filters::box_filter, strength)
}

/// Performs a gaussian blur in a view and returns the result.
#[must_use = "the blurred buffer is returned and the original view is left unmodified"]
pub fn gaussian_blur<I, P, C, const N: usize>(view: &I, strength: f32) -> ImgBuf<P, Vec<P>>
where
    I: Img<Pixel = P>,
    P: Pixel<Channels = [C; N]>,
    C: Processable,
{
    assert!(strength > 0.0);
    resample(
        view,
        view.dimensions(),
        |x| filters::gaussian(x, strength),
        2.0 * strength,
    )
}

/// Filter type to use when resizing a view using the [`resize`] function.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResizeFilter {
    Box,
    Triangle,
    BSpline,
    Mitchell,
    CatmullRom,
    Lanczos2,
    Lanczos3,
}

/// Resizes a view to the given dimensions using the given resizing filter.
#[must_use = "the resized buffer is returned and the original view is left unmodified"]
pub fn resize<I, P, C, const N: usize>(
    view: &I,
    dimensions: (u32, u32),
    filter: ResizeFilter,
) -> ImgBuf<P, Vec<P>>
where
    I: Img<Pixel = P>,
    P: Pixel<Channels = [C; N]>,
    C: Processable,
{
    match filter {
        ResizeFilter::Box => resample(view, dimensions, filters::box_filter, 0.0),
        ResizeFilter::Triangle => resample(view, dimensions, filters::triangle, 1.0),
        ResizeFilter::BSpline => resample(view, dimensions, filters::b_spline, 2.0),
        ResizeFilter::Mitchell => resample(view, dimensions, filters::mitchell, 2.0),
        ResizeFilter::CatmullRom => resample(view, dimensions, filters::catmull_rom, 2.0),
        ResizeFilter::Lanczos2 => resample(view, dimensions, filters::lanczos2, 2.0),
        ResizeFilter::Lanczos3 => resample(view, dimensions, filters::lanczos3, 3.0),
    }
}

/// Flips the given view horizontally.
pub fn flip_horizontal<I>(view: &mut I)
where
    I: ImgMut,
{
    for y in 0..view.height() {
        for x in 0..(view.width() / 2) {
            let left_pixel_bounds = Rect::new((x, y), (1, 1));
            let right_pixel_bounds = Rect::new((view.width() - 1 - x, y), (1, 1));

            let [mut left, mut right] = view
                .view_mut_multiple([left_pixel_bounds, right_pixel_bounds])
                .unwrap();

            std::mem::swap(
                left.pixel_mut((0, 0)).unwrap(),
                right.pixel_mut((0, 0)).unwrap(),
            );
        }
    }
}

/// Flips the given view vertically.
pub fn flip_vertical<I>(view: &mut I)
where
    I: ImgMut,
{
    for x in 0..view.width() {
        for y in 0..(view.height() / 2) {
            let top_pixel_bounds = Rect::new((x, y), (1, 1));
            let bottom_pixel_bounds = Rect::new((x, view.height() - 1 - y), (1, 1));

            let [mut top, mut bottom] = view
                .view_mut_multiple([top_pixel_bounds, bottom_pixel_bounds])
                .unwrap();

            std::mem::swap(
                top.pixel_mut((0, 0)).unwrap(),
                bottom.pixel_mut((0, 0)).unwrap(),
            );
        }
    }
}
