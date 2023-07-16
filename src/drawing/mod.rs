use crate::{view::ImgViewMut, Dimension, Point};

pub trait Drawing: ImgViewMut {
    fn draw_line<F>(&mut self, start: Point, end: Point, f: F)
    where
        F: FnMut(Point) -> Self::Pixel;

    fn draw_circle<F>(&mut self, center: Point, radius: u32, f: F)
    where
        F: FnMut(Point) -> Self::Pixel;

    fn draw_circumference<F>(&mut self, center: Point, radius: u32, f: F)
    where
        F: FnMut(Point) -> Self::Pixel;
}

impl<I> Drawing for I
where
    I: ImgViewMut,
{
    /// Draws a line starting at `start` and ending at `end`, both inclusive, with
    /// pixel colors calculated by the given function.
    ///
    /// # Panics
    /// Panics if either `start` or `end` are out of bounds.
    fn draw_line<F>(&mut self, start: Point, end: Point, mut f: F)
    where
        F: FnMut(Point) -> Self::Pixel,
    {
        assert!(self.bounds().contains(start));
        assert!(self.bounds().contains(end));

        let start = (start.0 as i64, start.1 as i64);
        let end = (end.0 as i64, end.1 as i64);

        let delta_x = end.0 - start.0;
        let delta_y = end.1 - start.1;
        let abs_delta_x = delta_x.abs();
        let abs_delta_y = delta_y.abs();

        let x_rate = if delta_x > 0 { 1 } else { -1 };
        let y_rate = if delta_y > 0 { 1 } else { -1 };

        if abs_delta_x >= abs_delta_y {
            let (mut x, mut y) = start;
            let mut acc = 0;

            for _ in 0..abs_delta_x {
                acc += abs_delta_y;
                if acc > abs_delta_x {
                    acc -= abs_delta_x;
                    y += y_rate;
                }

                let coords = (x as Dimension, y as Dimension);
                *unsafe { self.pixel_mut_unchecked(coords) } = f(coords);

                x += x_rate;
            }
        } else {
            let (mut x, mut y) = start;
            let mut acc = 0;

            for _ in 0..abs_delta_y {
                acc += abs_delta_x;
                if acc > abs_delta_y {
                    acc -= abs_delta_y;
                    x += x_rate;
                }

                let coords = (x as Dimension, y as Dimension);
                *unsafe { self.pixel_mut_unchecked(coords) } = f(coords);

                y += y_rate;
            }
        }
    }

    fn draw_circle<F>(&mut self, center: Point, radius: u32, mut f: F)
    where
        F: FnMut(Point) -> Self::Pixel,
    {
        let mut rel_x = 0;
        let mut rel_y = radius;
        // approximation for (radius + 1/2)² = radius² + radius + 1/4
        let radius_squared = radius * radius + radius;

        while rel_x <= rel_y {
            macro_rules! put {
                ($x:expr, $y:expr) => {
                    let start = (
                        (center.0 + $x).min(self.width()),
                        (center.1 + $y).min(self.height()),
                    );
                    let end = (
                        (center.0.saturating_sub($x)),
                        (center.1 + $y).min(self.height()),
                    );
                    self.draw_line(start, end, &mut f);
                };
                ($x:expr, neg $y:expr) => {
                    let start = (
                        (center.0 + $x).min(self.width()),
                        (center.1.saturating_sub($y)),
                    );
                    let end = ((center.0.saturating_sub($x)), (center.1.saturating_sub($y)));
                    self.draw_line(start, end, &mut f);
                };
            }

            put!(rel_x, rel_y);
            put!(rel_x, neg rel_y);
            put!(rel_y, rel_x);
            put!(rel_y, neg rel_x);

            if rel_y * rel_y >= radius_squared - rel_x * rel_x {
                rel_y -= 1;
            }

            rel_x += 1;
        }
    }

    fn draw_circumference<F>(&mut self, center: Point, radius: u32, mut f: F)
    where
        F: FnMut(Point) -> Self::Pixel,
    {
        let mut rel_x = 0;
        let mut rel_y = radius;
        // approximation for (radius + 1/2)² = radius² + radius + 1/4
        let radius_squared = radius * radius + radius;

        while rel_x <= rel_y {
            macro_rules! put {
                ($x:expr, $y:expr) => {
                    let coords = (
                        (center.0 + $x).min(self.width()),
                        (center.1 + $y).min(self.height()),
                    );
                    *unsafe { self.pixel_mut_unchecked(coords) } = f(coords);
                };
                (neg $x:expr, $y:expr) => {
                    let coords = (
                        (center.0.saturating_sub($x)),
                        (center.1 + $y).min(self.height()),
                    );
                    *unsafe { self.pixel_mut_unchecked(coords) } = f(coords);
                };
                ($x:expr, neg $y:expr) => {
                    let coords = (
                        (center.0 + $x).min(self.width()),
                        (center.1.saturating_sub($y)),
                    );
                    *unsafe { self.pixel_mut_unchecked(coords) } = f(coords);
                };
                (neg $x:expr, neg $y:expr) => {
                    let coords = ((center.0.saturating_sub($x)), (center.1.saturating_sub($y)));
                    *unsafe { self.pixel_mut_unchecked(coords) } = f(coords);
                };
            }

            put!(rel_x, rel_y);
            put!(neg rel_x, rel_y);
            put!(rel_x, neg rel_y);
            put!(neg rel_x, neg rel_y);
            put!(rel_y, rel_x);
            put!(neg rel_y, rel_x);
            put!(rel_y, neg rel_x);
            put!(neg rel_y, neg rel_x);

            if rel_y * rel_y >= radius_squared - rel_x * rel_x {
                rel_y -= 1;
            }

            rel_x += 1;
        }
    }
}
