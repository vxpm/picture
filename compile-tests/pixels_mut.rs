use picture::prelude::*;
fn main() {
    let mut img = Rgb8Img::new(1024, 1024);

    let mut pixels_mut = img.pixels_mut();
    let (_view_mut_a, _view_mut_b) = img.split_x_at(0).unwrap();

    let _ = pixels_mut.next();
}
