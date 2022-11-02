use picture::prelude::*;
fn main() {
    let mut img = Rgb8Image::new(1024, 1024);

    let (view_a, view_b) = img.split_x_at(0).unwrap();
    let _view_mut = img.view_mut(Rect::empty((0, 0)));

    let _ = view_a.pixel((0, 0));
    let _ = view_b.pixel((0, 0));
}
