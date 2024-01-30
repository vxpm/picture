use picture::{
    buffer::common::CommonImgBuf,
    formats::{png::Encoder, ImgEncoder},
    prelude::*,
};

fn main() {
    let image = picture::open("examples/images/star.png").unwrap();
    let CommonImgBuf::Rgba8(image) = image else {
        unreachable!()
    };

    let converted = image.map_vec(|p| RGB8::new(p.r, p.g, p.b));

    let file = std::fs::File::create("examples/images/out_converted.png").unwrap();
    Encoder::default().encode(file, converted).unwrap();
}
