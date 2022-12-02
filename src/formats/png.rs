use crate::pixel::common::*;
use crate::prelude::{ImgView, Pixel};
use crate::util::{dimension_to_usize, Array};
use crate::{
    buffer::common::{Gray8Img, Graya8Img},
    prelude::{Rgb8Img, Rgba8Img},
};
use std::{io::Read, path::Path};
use thiserror::Error;

pub use png::BitDepth;
pub use png::ColorType;
pub use png::DecodingError;
pub use png::EncodingError;

#[derive(Debug, Error)]
pub enum PngError {
    #[error("decoding error: {0}")]
    DecodingError(#[from] DecodingError),
    #[error("encoding error: {0}")]
    EncodingError(#[from] EncodingError),
    #[error("PNG is indexed - unsupported")]
    Indexed,
    #[error("unsupported channel amount: {0}")]
    UnsupportedChannelAmount(usize),
}

pub enum PngImage {
    Gray(Gray8Img),
    GrayAlpha(Graya8Img),
    Rgb(Rgb8Img),
    Rgba(Rgba8Img),
}

pub struct PngDecoder;

impl PngDecoder {
    pub fn decode<B>(&mut self, data: B) -> Result<PngImage, PngError>
    where
        B: AsRef<[u8]>,
    {
        let decoder = png::Decoder::new(data.as_ref());
        let mut reader = decoder.read_info()?;
        let (width, height) = (reader.info().width, reader.info().height);

        macro_rules! inner {
            ($pixel:ty, $factor:expr, $variant:ident, $image:ident) => {{
                let def = <$pixel>::default();
                let mut container = vec![def; reader.output_buffer_size() / $factor];
                let info = reader.next_frame(bytemuck::cast_slice_mut(&mut container))?;
                container.shrink_to(info.buffer_size() / $factor);

                PngImage::$variant($image::from_container(container, width, height))
            }};
        }

        Ok(match reader.info().color_type {
            png::ColorType::Grayscale => {
                inner!(GRAY8, 1, Gray, Gray8Img)
            }
            png::ColorType::GrayscaleAlpha => {
                inner!(GRAYA8, 2, GrayAlpha, Graya8Img)
            }
            png::ColorType::Rgb => {
                inner!(RGB8, 3, Rgb, Rgb8Img)
            }
            png::ColorType::Rgba => {
                inner!(RGBA8, 4, Rgba, Rgba8Img)
            }
            png::ColorType::Indexed => return Err(PngError::Indexed),
        })
    }

    pub fn decode_from_path<P>(&mut self, path: P) -> Result<PngImage, PngError>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let mut file = std::fs::File::open(path).map_err(png::DecodingError::IoError)?;
        let mut buffer = Vec::with_capacity(
            file.metadata()
                .map(|meta| meta.len() as usize)
                .unwrap_or(512),
        );

        file.read_to_end(&mut buffer)
            .map_err(png::DecodingError::IoError)?;
        Self.decode(buffer)
    }
}

pub struct PngEncoder {
    color_type: Option<ColorType>,
    depth: BitDepth,
}

impl Default for PngEncoder {
    fn default() -> Self {
        Self {
            color_type: None,
            depth: BitDepth::Eight,
        }
    }
}

impl PngEncoder {
    pub fn encode<I>(self, view: I) -> Result<Vec<u8>, PngError>
    where
        I: ImgView,
        I::Pixel: Pixel,
    {
        let mut buffer: Vec<u8> = Vec::with_capacity(
            dimension_to_usize(view.size()) * <<I::Pixel as Pixel>::Channels as Array>::SIZE,
        );

        {
            let mut encoder = png::Encoder::new(
                std::io::Cursor::new(&mut buffer),
                view.width(),
                view.height(),
            );

            if let Some(c) = self.color_type {
                encoder.set_color(c);
            } else {
                match <<I::Pixel as Pixel>::Channels as Array>::SIZE {
                    1 => encoder.set_color(png::ColorType::Grayscale),
                    2 => encoder.set_color(png::ColorType::GrayscaleAlpha),
                    3 => encoder.set_color(png::ColorType::Rgb),
                    4 => encoder.set_color(png::ColorType::Rgba),
                    x => return Err(PngError::UnsupportedChannelAmount(x)),
                }
            }

            encoder.set_depth(self.depth);

            let mut writer = encoder.write_header()?;
            let mut writer = writer.stream_writer()?;
            view.write_data(&mut writer)
                .map_err(png::EncodingError::IoError)?;

            writer.finish()?;
        }

        buffer.shrink_to_fit();
        Ok(buffer)
    }
}
