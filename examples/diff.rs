use picture::{
    buffer::common::CommonImgBuf,
    formats::{png::Encoder, ImgEncoder},
    prelude::*,
};

fn diff<I1, I2>(a: &I1, b: &I2) -> Rgba8Img
where
    I1: Img<Pixel = RGBA8>,
    I2: Img<Pixel = RGBA8>,
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
    let star = picture::open("examples/images/star.png").unwrap();
    let CommonImgBuf::Rgba8(star) = star else {
        unreachable!()
    };
    let rainbow = picture::open("examples/images/rainbow.png").unwrap();
    let CommonImgBuf::Rgba8(rainbow) = rainbow else {
        unreachable!()
    };

    let diff = diff(&star, &rainbow);
    let file = std::fs::File::create("examples/images/out_diff.png").unwrap();
    Encoder::default().encode(file, diff).unwrap();
}
