#[inline]
pub fn box_filter(_: f32) -> f32 {
    1.0
}

#[inline]
pub fn triangle(x: f32) -> f32 {
    if x.abs() < 1.0 {
        1.0 - x.abs()
    } else {
        0.0
    }
}

#[inline]
pub fn cubic(x: f32, b: f32, c: f32) -> f32 {
    let abs_x = x.abs();
    let v = if abs_x < 1.0 {
        (12.0 - 9.0 * b - 6.0 * c) * abs_x.powi(3)
            + (-18.0 + 12.0 * b + 6.0 * c) * abs_x.powi(2)
            + (6.0 - 2.0 * b)
    } else if abs_x < 2.0 {
        (-b - 6.0 * c) * abs_x.powi(3)
            + (6.0 * b + 30.0 * c) * abs_x.powi(2)
            + (-12.0 * b - 48.0 * c) * abs_x
            + (8.0 * b + 24.0 * c)
    } else {
        0.0
    };

    v / 6.0
}

#[inline]
pub fn b_spline(x: f32) -> f32 {
    cubic(x, 1.0, 0.0)
}

#[inline]
pub fn mitchell(x: f32) -> f32 {
    cubic(x, 1.0 / 3.0, 1.0 / 3.0)
}

#[inline]
pub fn catmull_rom(x: f32) -> f32 {
    cubic(x, 0.0, 0.5)
}

#[inline]
pub fn gaussian(x: f32, d: f32) -> f32 {
    1.0 / ((2.0 * std::f32::consts::PI).sqrt() * d) * (-x.powi(2) / (2.0 * d.powi(2))).exp()
}

#[inline]
pub fn sinc(x: f32) -> f32 {
    if x == 0.0 {
        1.0
    } else {
        x.sin() / x
    }
}

#[inline]
pub fn normalized_sinc(x: f32) -> f32 {
    sinc(std::f32::consts::PI * x)
}

#[inline]
pub fn lanczos(x: f32, a: f32) -> f32 {
    if x.abs() < a {
        normalized_sinc(x) * normalized_sinc(x / a)
    } else {
        0.0
    }
}

#[inline]
pub fn lanczos2(x: f32) -> f32 {
    lanczos(x, 2.0)
}

#[inline]
pub fn lanczos3(x: f32) -> f32 {
    lanczos(x, 3.0)
}
