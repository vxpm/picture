#[cfg(feature = "png")]
pub mod png;

use crate::{buffer::common::CommonImgBuf, view::Img};

/// Trait for types capable of encoding images to a specific format.
pub trait ImgEncoder<P> {
    /// Encodes an image and writes the output to a writer.
    fn encode<W, I>(&mut self, writer: W, img: I) -> std::io::Result<()>
    where
        W: std::io::Write,
        I: Img<Pixel = P>;
}

/// Trait for types capable of decoding images with a specific format and pixel type.
pub trait ImgDecoder<P> {
    type Output: Img<Pixel = P>;
    type Error: std::error::Error + 'static;

    /// Reads an image from a reader and decodes it.
    fn decode<R>(&mut self, reader: R) -> Result<Self::Output, Self::Error>
    where
        R: std::io::Read;
}

/// Trait for types capable of decoding images with a specific format supporting many common pixel
/// types.
pub trait CommonImgDecoder {
    type Error: std::error::Error + 'static;

    /// Reads an image from a reader and decodes it.
    fn decode_common<R>(&mut self, reader: R) -> Result<CommonImgBuf, Self::Error>
    where
        R: std::io::Read;
}
