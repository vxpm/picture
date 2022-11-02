use picture::prelude::*;
fn main() {
    let mut img = Rgb8Image::new(1024, 1024);

    let (view_mut_a, view_mut_b) = img.split_x_at_mut(0).unwrap();
    let _view = img.view(Rect::empty((0, 0)));

    let _ = view_mut_a.pixel((0, 0));
    let _ = view_mut_b.pixel((0, 0));
}
