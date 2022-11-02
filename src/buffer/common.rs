use super::ImageBuffer;
use crate::pixel::common::*;

macro_rules! img_ty {
    ($pixel:ident) => {
        paste::paste! {
            pub type [<$pixel:camel Image>] = ImageBuffer<$pixel, Vec<$pixel>>;
        }
    };
}

img_ty!(RGB8);
img_ty!(RGBA8);
img_ty!(RGB16);
img_ty!(RGBA16);
img_ty!(BGR8);
img_ty!(BGR16);
img_ty!(GRAY8);
img_ty!(GRAYA8);
img_ty!(GRAY16);
img_ty!(GRAYA16);
