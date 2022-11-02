use picture::{
    formats::png::{PngDecoder, PngEncoder, PngImage},
    prelude::*,
};
use std::io::Write;

fn grayscale<I1>(img: &mut I1)
where
    I1: ImageViewMut<Pixel = RGB8>,
{
    for p in img.pixels_mut() {
        let avg = ((p.r as u16 + p.g as u16 + p.b as u16) / 3) as u8;
        p.r = avg;
        p.g = avg;
        p.b = avg;
    }
}

fn main() {
    let colorful = PngDecoder
        .decode_from_path("examples/images/colorful.png")
        .unwrap();

    let PngImage::Rgb(mut colorful) = colorful else {
        unreachable!()
    };

    grayscale(&mut colorful);

    let encoded = PngEncoder::default().encode(colorful).unwrap();
    let mut f = std::fs::File::create("examples/images/out_grayscale.png").unwrap();
    f.write_all(&encoded[..]).unwrap();
}
