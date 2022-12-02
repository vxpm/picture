use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use image::GenericImageView;
use picture::{
    formats::png::{PngDecoder, PngImage},
    prelude::*,
};

#[inline]
fn picture_fractal((width, height): (Dimension, Dimension)) -> Rgb8Img {
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

    img
}

#[inline]
fn image_fractal((width, height): (u32, u32)) -> image::ImageBuffer<image::Rgb<u8>, Vec<u8>> {
    let mut img = image::ImageBuffer::new(width, height);

    let scalex = 3.0 / width as f32;
    let scaley = 3.0 / height as f32;

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let cx = y as f32 * scalex - 1.5;
        let cy = x as f32 * scaley - 1.5;

        let c = num_complex::Complex::new(-0.4, 0.6);
        let mut z = num_complex::Complex::new(cx, cy);

        let mut g = 0;
        while g < 255 && z.norm() <= 2.0 {
            z = z * z + c;
            g += 1;
        }

        *pixel = image::Rgb([((x * 255) / width) as u8, g, ((y * 255) / height) as u8]);
    }

    img
}

fn fractal(c: &mut Criterion) {
    let mut group = c.benchmark_group("Fractal 256x256");
    group.bench_function(BenchmarkId::new("Picture", ""), |b| {
        b.iter(|| picture_fractal(black_box((256, 256))))
    });
    group.bench_function(BenchmarkId::new("Image", ""), |b| {
        b.iter(|| image_fractal(black_box((256, 256))))
    });
    group.finish();
}

fn picture_diff<I1, I2>(a: &I1, b: &I2) -> u64
where
    I1: ImgView<Pixel = RGB8>,
    I2: ImgView<Pixel = RGB8>,
{
    a.pixels()
        .flat_map(|p| p.channels())
        .zip(b.pixels().flat_map(|p| p.channels()))
        .map(|(a, b)| a.abs_diff(*b) as u64)
        .sum()
}

fn image_diff<I1, I2>(a: &I1, b: &I2) -> u64
where
    I1: image::GenericImageView<Pixel = image::Rgb<u8>>,
    I2: image::GenericImageView<Pixel = image::Rgb<u8>>,
{
    a.pixels()
        .flat_map(|(_, _, p)| p.0)
        .zip(b.pixels().flat_map(|(_, _, p)| p.0))
        .map(|(a, b)| a.abs_diff(b) as u64)
        .sum()
}

fn diff(c: &mut Criterion) {
    let picture_img_a = Rgb8Img::from_fn(256, 256, |(x, y)| RGB8::new(x as u8, y as u8, 0));
    let picture_img_b = Rgb8Img::from_fn(256, 256, |(x, y)| RGB8::new(0, x as u8, y as u8));

    let image_img_a = image::RgbImage::from_fn(256, 256, |x, y| image::Rgb([x as u8, y as u8, 0]));
    let image_img_b = image::RgbImage::from_fn(256, 256, |x, y| image::Rgb([0, x as u8, y as u8]));

    let mut group = c.benchmark_group("Diff 256x256");
    group.bench_function(BenchmarkId::new("Picture", ""), |b| {
        b.iter(|| picture_diff(black_box(&picture_img_a), black_box(&picture_img_b)))
    });
    group.bench_function(BenchmarkId::new("Image", ""), |b| {
        b.iter(|| image_diff(black_box(&image_img_a), black_box(&image_img_b)))
    });
    group.finish();
}

fn picture_match<I1, I2>(source: &I1, target: &I2) -> Rect
where
    I1: ImgView<Pixel = RGB8>,
    I2: ImgView<Pixel = RGB8>,
{
    // given 'source', find the block that matches most closely the 'target'
    let max_x = source.dimensions().0 - target.dimensions().0;
    let max_y = source.dimensions().1 - target.dimensions().1;

    let mut rect = Rect::new((0, 0), (0, 0));
    let mut best = u64::MAX;
    for x in 0..max_x {
        for y in 0..max_y {
            let r = Rect::new((x, y), target.dimensions());
            let view = source.view(r).unwrap();
            let diff = picture_diff(&view, target);
            if diff < best {
                best = diff;
                rect = r;
            }
        }
    }

    rect
}

fn image_match<I1, I2>(source: &I1, target: &I2) -> Rect
where
    I1: image::GenericImageView<Pixel = image::Rgb<u8>>,
    I2: image::GenericImageView<Pixel = image::Rgb<u8>>,
{
    // given 'source', find the block that matches most closely the 'target'
    let (width, height) = target.dimensions();
    let max_x = source.dimensions().0 - width;
    let max_y = source.dimensions().1 - height;

    let mut rect = Rect::new((0, 0), (0, 0));
    let mut best = u64::MAX;
    for x in 0..max_x {
        for y in 0..max_y {
            let view = source.view(x, y, width, height);
            let diff = image_diff(&*view, target);
            if diff < best {
                best = diff;
                rect = Rect::new((x, y), (width, height));
            }
        }
    }

    rect
}

fn closest_match(c: &mut Criterion) {
    let picture_img_a = Rgb8Img::from_fn(256, 256, |(x, y)| RGB8::new(x as u8, y as u8, 0));
    let picture_img_b = picture_img_a.view(Rect::new((128, 128), (16, 16))).unwrap();

    let image_img_a = image::RgbImage::from_fn(256, 256, |x, y| image::Rgb([x as u8, y as u8, 75]));
    let image_img_b = image_img_a.view(128, 128, 16, 16);

    assert_eq!(
        picture_match(black_box(&picture_img_a), black_box(&picture_img_b)),
        image_match(black_box(&image_img_a), black_box(&*image_img_b))
    );

    let mut group = c.benchmark_group("Match 16x16 in 256x256");
    group.bench_function(BenchmarkId::new("Picture", ""), |b| {
        b.iter(|| picture_match(black_box(&picture_img_a), black_box(&picture_img_b)))
    });
    group.bench_function(BenchmarkId::new("Image", ""), |b| {
        b.iter(|| image_match(black_box(&image_img_a), black_box(&*image_img_b)))
    });
    group.finish();
}

fn lanczos_downsample(c: &mut Criterion) {
    let picture_img = PngDecoder
        .decode_from_path("examples/images/space.png")
        .unwrap();

    let PngImage::Rgb(picture_img) = picture_img else {
        unreachable!()
    };

    let image_img = image::open("examples/images/space.png").unwrap();

    let mut group = c.benchmark_group("Lanczos Downsample");
    group.bench_function(BenchmarkId::new("Picture", ""), |b| {
        b.iter(|| {
            picture::processing::resize(
                &picture_img,
                black_box((512, 256)),
                picture::processing::ResizeFilter::Lanczos3,
            )
        })
    });
    group.bench_function(BenchmarkId::new("Image", ""), |b| {
        b.iter(|| {
            image_img.resize(
                black_box(512),
                black_box(256),
                image::imageops::FilterType::Lanczos3,
            )
        })
    });
    group.finish();
}

fn lanczos_upsample(c: &mut Criterion) {
    let picture_img = PngDecoder
        .decode_from_path("examples/images/colorful.png")
        .unwrap();

    let PngImage::Rgb(picture_img) = picture_img else {
        unreachable!()
    };

    let image_img = image::open("examples/images/colorful.png").unwrap();

    let mut group = c.benchmark_group("Lanczos Upsample");
    group.bench_function(BenchmarkId::new("Picture", ""), |b| {
        b.iter(|| {
            picture::processing::resize(
                &picture_img,
                black_box((4096, 2048)),
                picture::processing::ResizeFilter::Lanczos3,
            )
        })
    });
    group.bench_function(BenchmarkId::new("Image", ""), |b| {
        b.iter(|| {
            image_img.resize(
                black_box(4096),
                black_box(2048),
                image::imageops::FilterType::Lanczos3,
            )
        })
    });
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().warm_up_time(std::time::Duration::from_secs_f32(3.0)).measurement_time(std::time::Duration::from_secs_f32(15.0)).sample_size(50);
    targets = diff, fractal, closest_match, lanczos_downsample, lanczos_upsample
}
criterion_main!(benches);
