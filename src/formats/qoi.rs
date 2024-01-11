use crate::{
    prelude::{ImgBuf, ImgView, Pixel, Rgb8Img, Rgba8Img},
    util::Array,
};
use either::Either;
use rgb::{RGB8, RGBA8};
use std::{io::Read, path::Path};

pub use either;
pub use qoi::Error as QoiError;

/// QOI Decoder.
pub struct QoiDecoder;

impl QoiDecoder {
    pub fn decode<B>(&mut self, data: B) -> Result<Either<Rgb8Img, Rgba8Img>, QoiError>
    where
        B: AsRef<[u8]>,
    {
        let mut decoder = qoi::Decoder::new(data.as_ref())?;
        match decoder.header().channels {
            qoi::Channels::Rgb => {
                let mut container: Vec<RGB8> =
                    vec![RGB8::new(0, 0, 0); decoder.required_buf_len() / 3];
                decoder.decode_to_buf(bytemuck::cast_slice_mut(&mut container))?;

                Ok(Either::Left(ImgBuf::from_container(
                    container,
                    decoder.header().width,
                    decoder.header().height,
                )))
            }
            qoi::Channels::Rgba => {
                let mut container: Vec<RGBA8> =
                    vec![RGBA8::new(0, 0, 0, 0); decoder.required_buf_len() / 4];
                decoder.decode_to_buf(bytemuck::cast_slice_mut(&mut container))?;

                Ok(Either::Right(ImgBuf::from_container(
                    container,
                    decoder.header().width,
                    decoder.header().height,
                )))
            }
        }
    }

    pub fn decode_from_path<P>(&mut self, path: P) -> Result<Either<Rgb8Img, Rgba8Img>, QoiError>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let mut file = std::fs::File::open(path)?;
        let mut buffer = Vec::with_capacity(
            file.metadata()
                .map(|meta| meta.len() as usize)
                .unwrap_or(512),
        );

        file.read_to_end(&mut buffer)?;
        Self.decode(buffer)
    }
}

/// QOI Encoder. Supports encoding images with either RGB8 or RGBA8 pixels.
pub struct QoiEncoder {
    pub colorspace: qoi::ColorSpace,
}

impl Default for QoiEncoder {
    fn default() -> Self {
        Self {
            colorspace: qoi::ColorSpace::Srgb,
        }
    }
}

impl QoiEncoder {
    pub fn encode<I>(self, view: I) -> Result<Vec<u8>, QoiError>
    where
        I: ImgView,
        I::Pixel: Pixel,
    {
        let mut buffer: Vec<u8> =
            Vec::with_capacity(view.size() * <<I::Pixel as Pixel>::Channels as Array>::SIZE);
        view.write_data(&mut buffer)?;

        let encoder = qoi::Encoder::new(&buffer, view.width(), view.height())?
            .with_colorspace(self.colorspace);
        encoder.encode_to_vec()
    }
}
