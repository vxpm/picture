use picture::{drawing::Drawing, formats::png::PngEncoder, prelude::*};
use std::io::Write;

fn main() {
    let (width, height) = (512, 512);
    let mut img = Rgb8Img::new(width, height);

    let a = (130, 382);
    let b = (5, 45);
    let c = (430, 127);

    let mut i = 0;

    img.draw_line(a, b, |_| RGB8::new(255, 0, 0));
    img.draw_line(b, c, |_| RGB8::new(0, 255, 0));
    img.draw_line(c, a, |_| RGB8::new(0, 0, 255));
    img.draw_circle((256, 256), 128, |_| {
        i += 1;
        RGB8::new((i / 256) as u8, (i / 128) as u8, (i / 64) as u8)
    });
    img.draw_circle((256, 256), 128, |_| RGB8::new(120, 30, 220));

    let encoded = PngEncoder::default().encode(img).unwrap();
    let mut f = std::fs::File::create("examples/images/out_drawing.png").unwrap();
    f.write_all(&encoded[..]).unwrap();
}
