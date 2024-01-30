use super::{CommonImgDecoder, ImgDecoder, ImgEncoder};
use crate::buffer::common::CommonImgBuf;
use crate::pixel::common::*;
use crate::prelude::ImgBuf;
use paste::paste;
use std::io::Write;
use thiserror::Error;

pub use png::{
    AdaptiveFilterType, BitDepth, ColorType, Compression, DecodingError, EncodingError, FilterType,
    SrgbRenderingIntent,
};

/// Errors that can happen during encoding/decoding operations.
#[derive(Debug, Error)]
pub enum Error {
    #[error("decoding error: {0}")]
    Decoding(#[from] DecodingError),
    #[error("encoding error: {0}")]
    Encoding(#[from] EncodingError),
    #[error("PNG is indexed - unsupported")]
    Indexed,
    #[error("wrong color type: {0:?}")]
    WrongColorType(ColorType),
    #[error("wrong bit depth: {0:?}")]
    WrongBitDepth(BitDepth),
}

/// A PNG Decoder.
#[derive(Debug, Default, Clone, Copy)]
pub struct Decoder;

macro_rules! impl_decoder {
    (inner $primitive_ty:ty, $pixel_ty:ident, $color_ty:ident, $factor:literal) => {
        impl ImgDecoder<$pixel_ty<$primitive_ty>> for Decoder {
            type Output = ImgBuf<$pixel_ty<$primitive_ty>>;
            type Error = Error;

            fn decode<R>(&mut self, reader: R) -> Result<Self::Output, Self::Error>
            where
                R: std::io::Read,
            {
                let decoder = png::Decoder::new(reader);
                let mut reader = decoder.read_info()?;

                let (width, height, color_type, bit_depth) = {
                    let info = reader.info();
                    (info.width, info.height, info.color_type, info.bit_depth)
                };

                if color_type != ColorType::$color_ty {
                    return Err(Error::WrongColorType(color_type));
                }

                if bit_depth as u32 != <$primitive_ty>::BITS {
                    return Err(Error::WrongBitDepth(bit_depth));
                }

                let mut container =
                    vec![$pixel_ty::<$primitive_ty>::default(); reader.output_buffer_size() / ((<$primitive_ty>::BITS as usize / 8) * $factor)];
                reader.next_frame(bytemuck::must_cast_slice_mut(&mut container))?;

                Ok(ImgBuf::from_container(container, width, height))
            }
        }
    };
    ($pixel_ty:ident, $color_ty:ident, $factor:literal) => {
        impl_decoder!(inner u8, $pixel_ty, $color_ty, $factor);
        impl_decoder!(inner u16, $pixel_ty, $color_ty, $factor);
    };
}

impl_decoder!(Gray, Grayscale, 1);
impl_decoder!(GrayAlpha, GrayscaleAlpha, 2);
impl_decoder!(RGB, Rgb, 3);
impl_decoder!(RGBA, Rgba, 4);

impl CommonImgDecoder for Decoder {
    type Error = Error;

    fn decode_common<R>(&mut self, reader: R) -> Result<CommonImgBuf, Self::Error>
    where
        R: std::io::Read,
    {
        let decoder = png::Decoder::new(reader);
        let mut reader = decoder.read_info()?;

        let (width, height, color_type, bit_depth) = {
            let info = reader.info();
            (info.width, info.height, info.color_type, info.bit_depth)
        };

        macro_rules! branch {
            (inner $depth:literal, $pixel_ty:ident, $factor:literal) => {
                {
                    paste! {
                        let mut container = vec![
                            [<$pixel_ty:upper $depth>]::default();
                            reader.output_buffer_size() / (($depth / 8) * $factor)
                        ];

                        reader.next_frame(bytemuck::must_cast_slice_mut(&mut container))?;

                        Ok(CommonImgBuf::[<$pixel_ty:camel $depth>](ImgBuf::from_container(
                                    container, width, height,
                        )))
                    }
                }
            };
            ($pixel_ty:ident, $factor:literal) => {
                match bit_depth {
                    BitDepth::Eight => branch!(inner 8, $pixel_ty, $factor),
                    BitDepth::Sixteen => branch!(inner 16, $pixel_ty, $factor),
                    depth => Err(Error::WrongBitDepth(depth)),
                }
            };
        }

        match color_type {
            ColorType::Grayscale => branch!(Gray, 1),
            ColorType::GrayscaleAlpha => branch!(Graya, 2),
            ColorType::Rgb => branch!(Rgb, 3),
            ColorType::Rgba => branch!(Rgba, 4),
            ColorType::Indexed => Err(Error::Indexed),
        }
    }
}

/// A PNG Encoder.
pub struct Encoder {
    pub compression: Compression,
    pub rendering_intent: SrgbRenderingIntent,
    pub filter_type: FilterType,
    pub adaptive_filter_type: AdaptiveFilterType,
}

impl Default for Encoder {
    fn default() -> Self {
        Self {
            compression: Compression::default(),
            rendering_intent: SrgbRenderingIntent::Perceptual,
            filter_type: FilterType::default(),
            adaptive_filter_type: AdaptiveFilterType::default(),
        }
    }
}

macro_rules! impl_encoder {
    (inner depth 8) => {
        BitDepth::Eight
    };
    (inner depth 16) => {
        BitDepth::Sixteen
    };
    (inner $depth:literal, $pixel_ty:ident, $color_ty:ident) => {
        paste! {
            impl ImgEncoder<[<$pixel_ty:upper $depth>]> for Encoder {
                fn encode<W, I>(&mut self, writer: W, img: I) -> std::io::Result<()>
                where
                    W: std::io::Write,
                    I: crate::view::Img<Pixel = [<$pixel_ty:upper $depth>]>,
                {
                    let mut encoder = png::Encoder::new(writer, img.width(), img.height());

                    encoder.set_color(ColorType::$color_ty);
                    encoder.set_depth(impl_encoder!(inner depth $depth));
                    encoder.set_compression(self.compression);
                    encoder.set_srgb(self.rendering_intent);
                    encoder.set_filter(self.filter_type);
                    encoder.set_adaptive_filter(self.adaptive_filter_type);

                    let mut writer = encoder.write_header()?;

                    // WARN: not sure what exactly can fail here
                    let mut stream_writer = writer
                        .stream_writer()
                        .expect("turning into stream writer is ok");

                    for chunk in img.pixel_chunks() {
                        // TODO: review possible endianess problems
                        stream_writer.write_all(bytemuck::must_cast_slice(chunk))?;
                    }

                    stream_writer.finish()?;

                    Ok(())
                }
            }
        }
    };
    ($pixel_ty:ident, $color_ty:ident) => {
        impl_encoder!(inner 8, $pixel_ty, $color_ty);
        impl_encoder!(inner 16, $pixel_ty, $color_ty);
    };
}

impl_encoder!(Gray, Grayscale);
impl_encoder!(Graya, GrayscaleAlpha);
impl_encoder!(Rgb, Rgb);
impl_encoder!(Rgba, Rgba);
