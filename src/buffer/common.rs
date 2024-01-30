use super::ImgBuf;
use crate::pixel::common::*;

macro_rules! buf_types {
    ($($pixel:ident),*) => {
        paste::paste! {
            $(
                pub type [<$pixel:camel Img>] = ImgBuf<$pixel, Vec<$pixel>>;
            )*

            pub enum CommonImgBuf {
                $(
                    [<$pixel:camel>]([<$pixel:camel Img>]),
                )*
            }
        }
    };
}

buf_types! {
    RGB8,
    RGBA8,
    RGB16,
    RGBA16,
    BGR8,
    BGR16,
    GRAY8,
    GRAYA8,
    GRAY16,
    GRAYA16
}
