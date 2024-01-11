use picture::prelude::*;
fn main() {
    let mut img = Rgb8Img::new(1024, 1024);

    let (mut view_mut_a, mut view_mut_b) = img.split_x_at_mut(0).unwrap();
    let (view_mut_a_1, view_mut_a_2) = view_mut_a.split_x_at_mut(0).unwrap();
    let (view_mut_b_1, view_mut_b_2) = view_mut_b.split_x_at_mut(0).unwrap();

    let _view = img.view(Rect::empty((0, 0)));

    let _ = view_mut_a_1.pixel((0, 0));
    let _ = view_mut_a_2.pixel((0, 0));
    let _ = view_mut_b_1.pixel((0, 0));
    let _ = view_mut_b_2.pixel((0, 0));
}
