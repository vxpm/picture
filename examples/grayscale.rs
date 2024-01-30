use picture::{
    buffer::common::CommonImgBuf,
    formats::{png::Encoder, ImgEncoder},
    prelude::*,
};

fn make_grayscale<I1>(img: &mut I1)
where
    I1: ImgMut<Pixel = RGB8>,
{
    for p in img.pixels_mut() {
        let avg = ((p.r as u16 + p.g as u16 + p.b as u16) / 3) as u8;
        p.r = avg;
        p.g = avg;
        p.b = avg;
    }
}

fn main() {
    let colorful = picture::open("examples/images/colorful.png").unwrap();
    let CommonImgBuf::Rgb8(mut colorful) = colorful else {
        unreachable!()
    };

    make_grayscale(&mut colorful);

    let file = std::fs::File::create("examples/images/out_grayscale.png").unwrap();
    Encoder::default().encode(file, colorful).unwrap();
}
