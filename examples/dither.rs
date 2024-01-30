use picture::{
    buffer::common::CommonImgBuf,
    formats::{png::Encoder, ImgEncoder},
    prelude::*,
};

// error diffusion dithering
fn dither<I1>(img: &mut I1)
where
    I1: ImgMut<Pixel = RGB8>,
{
    const BLACK: RGB8 = RGB8::new(0, 0, 0);
    const WHITE: RGB8 = RGB8::new(255, 255, 255);

    let mut error = 0i16;
    for p in img.pixels_mut() {
        // convert to grayscale
        let avg = ((p.r as u16 + p.g as u16 + p.b as u16) / 3) as u8;

        // propagate error
        let color = if error >= 0 {
            let taken = error.min(i16::from(avg.abs_diff(255)));
            error -= taken;

            avg + u8::try_from(taken).unwrap()
        } else {
            let taken = error.max(-(i16::from(avg)));
            error -= taken;

            avg - u8::try_from(taken.abs()).unwrap()
        };

        if color >= 128 {
            error -= i16::from(color.abs_diff(255));
            *p = WHITE;
        } else {
            error += i16::from(color);
            *p = BLACK;
        }
    }
}

fn main() {
    let space = picture::open("examples/images/space.png").unwrap();
    let CommonImgBuf::Rgb8(mut space) = space else {
        unreachable!()
    };

    dither(&mut space);

    let file = std::fs::File::create("examples/images/out_dither.png").unwrap();
    Encoder::default().encode(file, space).unwrap();
}
