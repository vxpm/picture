use picture::{
    formats::png::{PngDecoder, PngEncoder, PngImage},
    prelude::*,
};
use std::io::Write;

// error diffusion dithering
fn dither<I1>(img: &mut I1)
where
    I1: ImageViewMut<Pixel = RGB8>,
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
    let image = PngDecoder
        .decode_from_path("examples/images/space.png")
        .unwrap();

    let PngImage::Rgb(mut image) = image else {
        unreachable!()
    };

    dither(&mut image);

    let encoded = PngEncoder::default().encode(image).unwrap();
    let mut f = std::fs::File::create("examples/images/out_dithered.png").unwrap();
    f.write_all(&encoded[..]).unwrap();
}
