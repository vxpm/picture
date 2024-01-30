use picture::{
    buffer::common::CommonImgBuf,
    formats::{png::Encoder, ImgEncoder},
    prelude::*,
};

// equivalent to ImgMut::swap_with.
// reimplemented here for demonstration purposes
fn swap<I1, I2, P>(a: &mut I1, b: &mut I2)
where
    I1: ImgMut<Pixel = P>,
    I2: ImgMut<Pixel = P>,
{
    assert!(a.dimensions() == b.dimensions());
    a.pixels_mut()
        .zip(b.pixels_mut())
        .for_each(|(a, b)| std::mem::swap(a, b));
}

fn main() {
    let colorful = picture::open("examples/images/colorful.png").unwrap();
    let CommonImgBuf::Rgb8(mut colorful) = colorful else {
        unreachable!()
    };

    let (mut a, mut b) = colorful.split_x_at_mut(colorful.width() / 2).unwrap();
    swap(&mut a, &mut b);

    let file = std::fs::File::create("examples/images/out_swap.png").unwrap();
    Encoder::default().encode(file, colorful).unwrap();
}
