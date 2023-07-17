#![cfg_attr(feature = "unstable", feature(trusted_len))]
#![cfg_attr(feature = "unstable", feature(iter_advance_by))]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(clippy::missing_safety_doc)]
#![warn(clippy::trivially_copy_pass_by_ref)]

//! # Picture
//! A fast and flexible image manipulation crate.
//!
//! # Quick Start
//! Take a look at the [`prelude`] for the most important items. Start with the [`Pixel`][prelude::Pixel],
//! [`ImgView`][prelude::ImgView] and [`ImgViewMut`][prelude::ImgViewMut] traits. Then, the
//! [`ImgBuf`][prelude::ImgBuf] type.

#[cfg(not(feature = "u64_dimensions"))]
pub type Dimension = u32;
#[cfg(feature = "u64_dimensions")]
pub type Dimension = u64;

pub type Point = (Dimension, Dimension);

/// [`ImgBuf`][buffer::ImgBuf] and everything related to it.
pub mod buffer;

/// Modules related to common image formats.
#[cfg(feature = "formats")]
pub mod formats;

/// [`Pixel`][pixel::Pixel] trait and common pixel formats.
pub mod pixel;
/// Image processing, like resizing and blurring.
pub mod processing;
/// Overall utilities.
pub mod util;
/// [`ImgView`][view::ImgView] and [`ImgViewMut`][view::ImgViewMut] traits (+ extensions).
pub mod view;
/// Re-exports the most useful items of the crate.
pub mod prelude {
    use super::*;
    pub use super::{Dimension, Point};
    pub use buffer::{
        common::{Rgb16Img, Rgb8Img, Rgba16Img, Rgba8Img},
        ImgBuf,
    };
    pub use pixel::{
        common::{RGB, RGB16, RGB8, RGBA, RGBA16, RGBA8},
        Pixel,
    };
    pub use util::Rect;
    pub use view::{Img, ImgCore, ImgMut, ImgMutCore};
}

#[cfg(test)]
mod tests {
    #[test]
    fn compile_tests() {
        let t = trybuild::TestCases::new();
        t.compile_fail("compile-tests/*.rs");
    }
}
