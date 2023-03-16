use picture::{
    formats::png::{PngDecoder, PngEncoder, PngImage},
    processing::gaussian_blur,
};
use std::io::Write;

fn main() {
    let image = PngDecoder
        .decode_from_path("examples/images/space.png")
        .unwrap();

    let PngImage::Rgb(image) = image else {
        unreachable!()
    };

    let blurry = gaussian_blur(&image, 8.0);

    let encoded = PngEncoder::default().encode(blurry).unwrap();
    let mut f = std::fs::File::create("examples/images/out_blur.png").unwrap();
    f.write_all(&encoded[..]).unwrap();
}
