use crate::prelude::*;
use crate::util::{dimension_to_usize, index_point};
use crate::Dimension;
use easy_cast::{CastApprox, ConvApprox};

/// Common sampling filters.
pub mod filters;

// useful resources:
// - https://entropymine.com/imageworsener
// - https://cs1230.graphics/lectures - specifically image processing I, II and III

/// Resamples an image horizontally to the given width using the given filter.
/// Height is kept the same.
///
/// `window` is the maximum distance a pixel can be to the one being currently
/// processed before being cut out of the filter.
pub fn resample_horizontal<I, P, C, F, const N: usize>(
    img: &I,
    width: Dimension,
    filter: F,
    window: f32,
) -> ImageBuffer<P, Vec<P>>
where
    I: ImageView<Pixel = P>,
    P: Pixel<Channels = [C; N]>,
    C: Copy + ConvApprox<f32> + CastApprox<f32>,
    F: Fn(f32) -> f32,
{
    if width == 0 {
        return ImageBuffer::from_container(Vec::new(), width, img.height());
    }

    // create container for result
    let mut container =
        Vec::with_capacity(dimension_to_usize(width) * dimension_to_usize(img.height()));
    let container_pixels = container.spare_capacity_mut();

    // find the ratio between the source width and the target width
    let ratio = img.width() as f32 / width as f32;
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
    let max_src_x_f32 = (img.width() - 1) as f32;
    let mut weights = Vec::with_capacity((2 * (window as usize) + 1) * dimension_to_usize(width));
    let mut weights_start_index = Vec::with_capacity(width as usize);
    for target_x in 0..width {
        let equivalent_src_x = target_x as f32 * ratio + 0.5 * (ratio - 1.0);

        let min_src_pixel_x = (equivalent_src_x - window).clamp(0.0, max_src_x_f32) as Dimension;
        let max_src_pixel_x = (equivalent_src_x + window).clamp(0.0, max_src_x_f32) as Dimension;

        weights_start_index.push(weights.len());
        for src_pixel_x in min_src_pixel_x..=max_src_pixel_x {
            weights.push(filter(
                (src_pixel_x as f32 - equivalent_src_x) * inverse_sampling_ratio,
            ));
        }
    }

    // now actually resample
    for target_x in 0..width {
        // these could be cached as well, but it makes no performance difference (and increases
        // memory usage), so we just calculate them again
        let equivalent_src_x = target_x as f32 * ratio + (1.0 - 1.0 / ratio) / (2.0 / ratio);

        let min_src_pixel_x = (equivalent_src_x - window).clamp(0.0, max_src_x_f32) as Dimension;
        let max_src_pixel_x = (equivalent_src_x + window).clamp(0.0, max_src_x_f32) as Dimension;

        let weights_start = weights_start_index[target_x as usize];
        for target_y in 0..img.height() {
            let mut weight_sum = 0f32;
            let mut channel_value_sum = [0f32; N];
            for (index, src_pixel_x) in (min_src_pixel_x..=max_src_pixel_x).enumerate() {
                // SAFETY: target_y is in the 0..img.height() range and src_pixel_x is clamped
                // between 0 and img.width() - 1. therefore, this coordinate is always in bounds.
                let src_pixel = unsafe { img.pixel_unchecked((src_pixel_x, target_y)) };
                let channels = src_pixel.channels();
                let weight = weights[weights_start + index];
                weight_sum += weight;

                for channel_index in 0..N {
                    let value = weight * channels[channel_index].cast_approx();
                    channel_value_sum[channel_index] += value;
                }
            }

            let result: arrayvec::ArrayVec<_, N> = channel_value_sum
                .into_iter()
                .map(|v| C::conv_approx(v / weight_sum))
                .collect();

            // SAFETY: this index will always be valid since target_x and target_y are always in
            // the correct range.
            unsafe {
                container_pixels
                    .get_unchecked_mut(index_point((target_x, target_y), width))
                    .write(P::new(result.into_inner_unchecked()));
            }
        }
    }

    // SAFETY: all pixels have already been initialized in the previous loop.
    unsafe {
        let size = dimension_to_usize(width) * dimension_to_usize(img.height());
        container.set_len(size);
    }

    ImageBuffer::from_container(container, width, img.height())
}

/// Resamples an image vertically to the given height using the given filter.
/// Width is kept the same.
///
/// `window` is the maximum distance a pixel can be to the one being currently
/// processed before being cut out of the filter.
pub fn resample_vertical<I, P, C, F, const N: usize>(
    img: &I,
    height: Dimension,
    filter: F,
    window: f32,
) -> ImageBuffer<P, Vec<P>>
where
    I: ImageView<Pixel = P>,
    P: Pixel<Channels = [C; N]>,
    C: Copy + ConvApprox<f32> + CastApprox<f32>,
    F: Fn(f32) -> f32,
{
    if height == 0 {
        return ImageBuffer::from_container(Vec::new(), img.width(), height);
    }

    // create container for result
    let mut container =
        Vec::with_capacity(dimension_to_usize(height) * dimension_to_usize(img.width()));
    let container_pixels = container.spare_capacity_mut();

    // find the ratio between the source height and the target height
    let ratio = img.height() as f32 / height as f32;
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
    let max_src_y_f32 = (img.height() - 1) as f32;
    let mut weights = Vec::with_capacity((2 * (window as usize) + 1) * dimension_to_usize(height));
    let mut weights_start_index = Vec::with_capacity(height as usize);
    for target_y in 0..height {
        let equivalent_src_y = target_y as f32 * ratio + 0.5 * (ratio - 1.0);

        let min_src_pixel_y = (equivalent_src_y - window).clamp(0.0, max_src_y_f32) as Dimension;
        let max_src_pixel_y = (equivalent_src_y + window).clamp(0.0, max_src_y_f32) as Dimension;

        weights_start_index.push(weights.len());
        for src_pixel_y in min_src_pixel_y..=max_src_pixel_y {
            weights.push(filter(
                (src_pixel_y as f32 - equivalent_src_y) * inverse_sampling_ratio,
            ));
        }
    }

    // now actually resample
    for target_y in 0..height {
        // these could be cached as well, but it makes no performance difference (and increases
        // memory usage), so we just calculate them again
        let equivalent_src_y = target_y as f32 * ratio + 0.5 * (ratio - 1.0);

        let min_src_pixel_y = (equivalent_src_y - window).clamp(0.0, max_src_y_f32) as Dimension;
        let max_src_pixel_y = (equivalent_src_y + window).clamp(0.0, max_src_y_f32) as Dimension;

        let weights_start = weights_start_index[target_y as usize];
        for target_x in 0..img.width() {
            let mut weight_sum = 0f32;
            let mut channel_value_sum = [0f32; N];
            for (index, src_pixel_y) in (min_src_pixel_y..=max_src_pixel_y).enumerate() {
                // SAFETY: target_x is in the 0..img.width() range and src_pixel_y is clamped
                // between 0 and img.height() - 1. therefore, this coordinate is always in bounds.
                let src_pixel = unsafe { img.pixel_unchecked((target_x, src_pixel_y)) };
                let channels = src_pixel.channels();
                let weight = weights[weights_start + index];
                weight_sum += weight;

                for channel_index in 0..N {
                    let value = weight * channels[channel_index].cast_approx();
                    channel_value_sum[channel_index] += value;
                }
            }

            let result: arrayvec::ArrayVec<_, N> = channel_value_sum
                .into_iter()
                .map(|v| C::conv_approx(v / weight_sum))
                .collect();

            // SAFETY: this index will always be valid since target_x and target_y are always in
            // the correct range.
            unsafe {
                container_pixels
                    .get_unchecked_mut(index_point((target_x, target_y), img.width()))
                    .write(P::new(result.into_inner_unchecked()));
            }
        }
    }

    // SAFETY: all pixels have already been initialized in the previous loop.
    unsafe {
        let size = dimension_to_usize(height) * dimension_to_usize(img.width());
        container.set_len(size);
    }

    ImageBuffer::from_container(container, img.width(), height)
}

/// Resamples an image to the given dimensions using the given filter. This is
/// equivalent to doing a horizontal resample followed by a vertical one.
///
/// `window` is the maximum distance a pixel can be to the one being currently
/// processed before being cut out of the filter.
pub fn resample<I, P, C, F, const N: usize>(
    img: &I,
    (width, height): (Dimension, Dimension),
    filter: F,
    window: f32,
) -> ImageBuffer<P, Vec<P>>
where
    I: ImageView<Pixel = P>,
    P: Pixel<Channels = [C; N]>,
    C: Copy + ConvApprox<f32> + CastApprox<f32>,
    F: Fn(f32) -> f32,
{
    let horizontal = resample_horizontal(img, width, &filter, window);
    resample_vertical(&horizontal, height, filter, window)
}

/// Performs a box blur in an image and returns the result.
pub fn box_blur<I, P, C, const N: usize>(img: &I, strength: f32) -> ImageBuffer<P, Vec<P>>
where
    I: ImageView<Pixel = P>,
    P: Pixel<Channels = [C; N]>,
    C: Copy + ConvApprox<f32> + CastApprox<f32>,
{
    assert!(strength > 0.0);
    resample(img, img.dimensions(), filters::box_filter, strength)
}

/// Performs a gaussian blur in an image and returns the result.
pub fn gaussian_blur<I, P, C, const N: usize>(img: &I, strength: f32) -> ImageBuffer<P, Vec<P>>
where
    I: ImageView<Pixel = P>,
    P: Pixel<Channels = [C; N]>,
    C: Copy + ConvApprox<f32> + CastApprox<f32>,
{
    assert!(strength > 0.0);
    resample(
        img,
        img.dimensions(),
        |x| filters::gaussian(x, strength),
        2.0 * strength,
    )
}

/// Filter type to use when resizing an image using the [`resize`] function.
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

/// Resizes an image to the given dimensions using the given resizing filter.
pub fn resize<I, P, C, const N: usize>(
    img: &I,
    dimensions: (Dimension, Dimension),
    filter: ResizeFilter,
) -> ImageBuffer<P, Vec<P>>
where
    I: ImageView<Pixel = P>,
    P: Pixel<Channels = [C; N]>,
    C: Copy + ConvApprox<f32> + CastApprox<f32>,
{
    match filter {
        ResizeFilter::Box => resample(img, dimensions, filters::box_filter, 0.0),
        ResizeFilter::Triangle => resample(img, dimensions, filters::triangle, 1.0),
        ResizeFilter::BSpline => resample(img, dimensions, filters::b_spline, 2.0),
        ResizeFilter::Mitchell => resample(img, dimensions, filters::mitchell, 2.0),
        ResizeFilter::CatmullRom => resample(img, dimensions, filters::catmull_rom, 2.0),
        ResizeFilter::Lanczos2 => resample(img, dimensions, filters::lanczos2, 2.0),
        ResizeFilter::Lanczos3 => resample(img, dimensions, filters::lanczos3, 3.0),
    }
}

/// Flips the given image horizontally.
pub fn flip_horizontal<I>(img: &mut I)
where
    I: ImageViewMut,
{
    for y in 0..img.height() {
        for x in 0..(img.width() / 2) {
            let left_pixel_bounds = Rect::new((x, y), (1, 1));
            let right_pixel_bounds = Rect::new((img.width() - 1 - x, y), (1, 1));

            let [mut left, mut right] = img
                .view_mut_multiple([left_pixel_bounds, right_pixel_bounds])
                .unwrap();

            std::mem::swap(
                left.pixel_mut((0, 0)).unwrap(),
                right.pixel_mut((0, 0)).unwrap(),
            );
        }
    }
}

/// Flips the given image vertically.
pub fn flip_vertical<I>(img: &mut I)
where
    I: ImageViewMut,
{
    for x in 0..img.width() {
        for y in 0..(img.height() / 2) {
            let top_pixel_bounds = Rect::new((x, y), (1, 1));
            let bottom_pixel_bounds = Rect::new((x, img.height() - 1 - y), (1, 1));

            let [mut top, mut bottom] = img
                .view_mut_multiple([top_pixel_bounds, bottom_pixel_bounds])
                .unwrap();

            std::mem::swap(
                top.pixel_mut((0, 0)).unwrap(),
                bottom.pixel_mut((0, 0)).unwrap(),
            );
        }
    }
}
