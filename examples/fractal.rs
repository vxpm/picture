use picture::{formats::png::PngEncoder, prelude::*};
use std::io::Write;

fn main() {
    // based on the fractal example of the 'image' crate
    let (width, height) = (1024, 1024);
    let mut img = Rgb8Img::new(width, height);

    let scalex = 3.0 / width as f32;
    let scaley = 3.0 / height as f32;

    for ((x, y), pixel) in img.pixels_with_coords_mut() {
        let cx = y as f32 * scalex - 1.5;
        let cy = x as f32 * scaley - 1.5;

        let c = num_complex::Complex::new(-0.4, 0.6);
        let mut z = num_complex::Complex::new(cx, cy);

        let mut g = 0;
        while g < 255 && z.norm() <= 2.0 {
            z = z * z + c;
            g += 1;
        }

        *pixel = RGB8 {
            r: ((x * 255) / width) as u8,
            g,
            b: ((y * 255) / height) as u8,
        };
    }

    let encoded = PngEncoder::default().encode(img).unwrap();
    let mut f = std::fs::File::create("examples/images/out_frac.png").unwrap();
    f.write_all(&encoded[..]).unwrap();
}
