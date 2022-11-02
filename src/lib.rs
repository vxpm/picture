#![cfg_attr(feature = "unstable", feature(trusted_len))]
#![cfg_attr(feature = "unstable", feature(iter_advance_by))]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(clippy::missing_safety_doc)]
#![warn(clippy::trivially_copy_pass_by_ref)]

//! # picture
//! a fast and flexible image manipulation crate.
//!
//! # quick start
//! take a look at the [`prelude`] for the most important items. start with the [`Pixel`][prelude::Pixel],
//! [`ImageView`][prelude::ImageView] and [`ImageViewMut`][prelude::ImageViewMut] traits. then, the
//! [`ImageBuffer`][prelude::ImageBuffer] type.

#[cfg(not(feature = "u64_dimensions"))]
pub type Dimension = u32;
#[cfg(feature = "u64_dimensions")]
pub type Dimension = u64;

pub type Point = (Dimension, Dimension);

/// [`ImageBuffer`][buffer::ImageBuffer] and everything related to it.
pub mod buffer;

/// Modules related to common image formats.
#[cfg(feature = "formats")]
pub mod formats;

/// [`Pixel`][pixel::Pixel] trait and common pixel formats.
pub mod pixel;
/// Overall utilities.
pub mod util;
/// [`ImageView`][view::ImageView] and [`ImageViewMut`][view::ImageViewMut] traits.
pub mod view;
/// Re-exports the most useful items of the crate.
pub mod prelude {
    use super::*;
    pub use super::{Dimension, Point};
    pub use buffer::{
        common::{Rgb16Image, Rgb8Image, Rgba16Image, Rgba8Image},
        ImageBuffer,
    };
    pub use pixel::{
        common::{RGB, RGB16, RGB8, RGBA, RGBA16, RGBA8},
        Pixel,
    };
    pub use util::Rect;
    pub use view::{ImageView, ImageViewExt, ImageViewMut, ImageViewMutExt};
}

#[cfg(test)]
mod tests {
    #[test]
    fn compile_tests() {
        let t = trybuild::TestCases::new();
        t.compile_fail("compile-tests/*.rs");
    }
}
