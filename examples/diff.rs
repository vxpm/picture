use picture::{
    formats::png::{PngDecoder, PngEncoder, PngImage},
    prelude::*,
};
use std::io::Write;

fn diff<I1, I2>(a: &I1, b: &I2) -> Rgba8Img
where
    I1: ImgView<Pixel = RGBA8>,
    I2: ImgView<Pixel = RGBA8>,
{
    assert!(a.dimensions() == b.dimensions());

    let mut iter = a.pixels().zip(b.pixels());

    Rgba8Img::from_fn(a.width(), a.height(), move |_| {
        let (a, b) = iter.next().unwrap();
        RGBA8 {
            r: a.r.abs_diff(b.r),
            g: a.g.abs_diff(b.g),
            b: a.b.abs_diff(b.b),
            a: a.a.abs_diff(b.a),
        }
    })
}

fn main() {
    let star = PngDecoder
        .decode_from_path("examples/images/star.png")
        .unwrap();
    let rainbow = PngDecoder
        .decode_from_path("examples/images/rainbow.png")
        .unwrap();

    let PngImage::Rgba(star) = star else {
        unreachable!()
    };

    let PngImage::Rgba(rainbow) = rainbow else {
        unreachable!()
    };

    let diff = diff(&star, &rainbow);
    let encoded = PngEncoder::default().encode(diff).unwrap();
    let mut f = std::fs::File::create("examples/images/out_diff.png").unwrap();
    f.write_all(&encoded[..]).unwrap();
}
