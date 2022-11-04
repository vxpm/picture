# picture
a fast and flexible image manipulation crate.

# warning: wip
this crate is still a WIP. while it's foundations are there (but not stable), it's still missing
many convenience and quality of life features - mostly image processing (like resizing) and formats
related.

it's also in necessity of tests™ (there's _very_ few).

# todo
- add image processing utilities (resizing, filtering, etc...)
- better & expanded image formats support (currently very half-baked and only supports png and qoi)
- add way more tests
- try to reduce usage of unsafe blocks

# fractal example
based on the fractal example of the [`image`](https://crates.io/crates/image) crate:
```rust
use picture::{formats::png::PngEncoder, prelude::*};
use std::io::Write;

fn main() {
    let (width, height) = (1024, 1024);
    let mut img = Rgb8Image::new(width, height);

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
    let mut f = std::fs::File::create("frac.png").unwrap();
    f.write_all(&encoded[..]).unwrap();
}
```
(you can find this code in the [examples](examples/fractal.rs))

# disjoint mutable views example
one of the coolest features of `picture` is the ability to have disjoint mutable views into a view.
the following example swaps the two horizontal halves of an `ImageBuffer` and then saves the result.
```rust
use picture::{
    formats::png::{PngDecoder, PngEncoder, PngImage},
    prelude::*,
};
use std::io::Write;

fn swap<I1, I2, P>(a: &mut I1, b: &mut I2)
where
    I1: ImageViewMut<Pixel = P>,
    I2: ImageViewMut<Pixel = P>,
{
    assert!(a.dimensions() == b.dimensions());
    a.pixels_mut()
        .zip(b.pixels_mut())
        .for_each(|(a, b)| std::mem::swap(a, b));
}

fn main() {
    let colorful = PngDecoder
        .decode_from_path("examples/images/colorful.png")
        .unwrap();

    let PngImage::Rgb(mut colorful) = colorful else {
        unreachable!()
    };

    let (mut a, mut b) = colorful.split_x_at_mut(colorful.width() / 2).unwrap();
    swap(&mut a, &mut b);

    let encoded = PngEncoder::default().encode(colorful).unwrap();
    let mut f = std::fs::File::create("swapped.png").unwrap();
    f.write_all(&encoded[..]).unwrap();
}
```
(you can find this code in the [examples](examples/swap.rs) as well!)

the `ImageViewMut` trait has many methods to obtain disjoint mutable views: `split_x_at_mut`, 
`split_y_at_mut` and `view_mut_multiple`.

# performance
i've written a few [benchmarks](benches/picture_bench.rs) to compare `picture` and 
[`image`](https://crates.io/crates/image), but i'm not experienced at writing these - so take these
results with a grain of salt. benchmarks were executed on my ryzen 5 1600 with lto = "thin".

| Benchmark                       | Time                            |
| ------------------------------- | ------------------------------- |
| Diff 256x256/Picture/           | [652.17 µs 655.06 µs 658.63 µs] |
| Diff 256x256/Image/             | [933.05 µs 938.71 µs 944.43 µs] |
| Fractal 256x256/Picture/        | [8.2990 ms 8.4081 ms 8.5218 ms] |
| Fractal 256x256/Image/          | [21.815 ms 21.867 ms 21.922 ms] |
| Match 16x16 in 256x256/Picture/ | [179.98 ms 180.64 ms 181.34 ms] |
| Match 16x16 in 256x256/Image/   | [231.94 ms 233.09 ms 234.24 ms] |

you can try running these yourself with a simple `cargo bench`.

# unsafe
this crate uses `unsafe` in many places, but i've tried my best to make it all sound - including
adding `SAFETY:` comments above every `unsafe` block. it's my first big project with abundancy of
`unsafe`, however, so if you find any soundness holes please let me know!
