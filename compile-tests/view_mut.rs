use picture::prelude::*;
fn main() {
    let mut img = Rgb8Image::new(1024, 1024);

    let view = img.view(Rect::empty((0, 0))).unwrap();
    let _view_mut = img.view_mut(Rect::empty((0, 0)));

    let _ = view.pixel((0, 0));
}
