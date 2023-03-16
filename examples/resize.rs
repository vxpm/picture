use picture::{
    formats::png::{PngDecoder, PngEncoder, PngImage},
    processing::resize,
};
use std::io::Write;

fn main() {
    let image = PngDecoder
        .decode_from_path("examples/images/colorful.png")
        .unwrap();

    let PngImage::Rgb(image) = image else {
        unreachable!()
    };

    let small = resize(
        &image,
        (128, 64),
        picture::processing::ResizeFilter::Triangle,
    );
    let big = resize(
        &image,
        (4096, 2048),
        picture::processing::ResizeFilter::CatmullRom,
    );

    let encoded = PngEncoder::default().encode(small).unwrap();
    let mut f = std::fs::File::create("examples/images/out_resize_small.png").unwrap();
    f.write_all(&encoded[..]).unwrap();

    let encoded = PngEncoder::default().encode(big).unwrap();
    let mut f = std::fs::File::create("examples/images/out_resize_big.png").unwrap();
    f.write_all(&encoded[..]).unwrap();
}
