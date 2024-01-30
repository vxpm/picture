use picture::{
    buffer::common::CommonImgBuf,
    formats::{png::Encoder, ImgEncoder},
    processing::gaussian_blur,
};

fn main() {
    let image = picture::open("examples/images/space.png").unwrap();
    let CommonImgBuf::Rgb8(image) = image else {
        unreachable!()
    };

    let blurry = gaussian_blur(&image, 8.0);

    let file = std::fs::File::create("examples/images/out_blur.png").unwrap();
    Encoder::default().encode(file, blurry).unwrap();
}
