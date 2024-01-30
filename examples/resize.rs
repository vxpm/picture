use picture::{
    buffer::common::CommonImgBuf,
    formats::{png::Encoder, ImgEncoder},
    processing::resize,
};

fn main() {
    let colorful = picture::open("examples/images/colorful.png").unwrap();
    let CommonImgBuf::Rgb8(colorful) = colorful else {
        unreachable!()
    };

    let small = resize(
        &colorful,
        (128, 64),
        picture::processing::ResizeFilter::Triangle,
    );
    let big = resize(
        &colorful,
        (4096, 2048),
        picture::processing::ResizeFilter::CatmullRom,
    );

    let file = std::fs::File::create("examples/images/out_resize_small.png").unwrap();
    Encoder::default().encode(file, small).unwrap();

    let file = std::fs::File::create("examples/images/out_resize_big.png").unwrap();
    Encoder::default().encode(file, big).unwrap();
}
