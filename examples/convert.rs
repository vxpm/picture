use picture::{
    formats::png::{PngDecoder, PngEncoder, PngImage},
    prelude::*,
};
use std::io::Write;

fn main() {
    let star = PngDecoder
        .decode_from_path("examples/images/star.png")
        .unwrap();

    let PngImage::Rgba(star) = star else {
        unreachable!()
    };

    let star = star.map_vec(|p| RGB8::new(p.r, p.g, p.b));

    let encoded = PngEncoder::default().encode(star).unwrap();
    let mut f = std::fs::File::create("examples/images/out_converted.png").unwrap();
    f.write_all(&encoded[..]).unwrap();
}
