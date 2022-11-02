use picture::{
    formats::png::{PngDecoder, PngEncoder, PngImage},
    prelude::*,
};
use std::io::Write;

fn swap<I1, I2, P>(a: &mut I1, b: &mut I2)
where
    I1: ImageViewMut<Pixel = P>,
    I2: ImageViewMut<Pixel = P>,
{
    assert!(a.dimensions() == b.dimensions());
    a.pixels_mut()
        .zip(b.pixels_mut())
        .for_each(|(a, b)| std::mem::swap(a, b));
}

fn main() {
    let colorful = PngDecoder
        .decode_from_path("examples/images/colorful.png")
        .unwrap();

    let PngImage::Rgb(mut colorful) = colorful else {
        unreachable!()
    };

    let (mut a, mut b) = colorful.split_x_at_mut(colorful.width() / 2).unwrap();
    swap(&mut a, &mut b);

    let encoded = PngEncoder::default().encode(colorful).unwrap();
    let mut f = std::fs::File::create("examples/images/out_swapped.png").unwrap();
    f.write_all(&encoded[..]).unwrap();
}
