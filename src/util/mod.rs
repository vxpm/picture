use crate::prelude::Point;

pub(crate) mod macros;

mod private {
    pub trait Sealed {}
    impl<T, const SIZE: usize> Sealed for [T; SIZE] {}
}

/// Trait for arrays, exclusively.
pub trait Array: private::Sealed {
    type Elem;
    const SIZE: usize;
    fn as_slice(&self) -> &[Self::Elem];
    fn as_mut_slice(&mut self) -> &mut [Self::Elem];
    fn iter(&self) -> std::slice::Iter<'_, Self::Elem>;
    fn iter_mut(&mut self) -> std::slice::IterMut<'_, Self::Elem>;
}

impl<T, const SIZE: usize> Array for [T; SIZE] {
    type Elem = T;
    const SIZE: usize = SIZE;

    #[inline]
    fn as_slice(&self) -> &[T] {
        self.as_slice()
    }

    #[inline]
    fn as_mut_slice(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }

    #[inline]
    fn iter(&self) -> std::slice::Iter<'_, Self::Elem> {
        #[allow(clippy::into_iter_on_ref)]
        self.into_iter()
    }

    #[inline]
    fn iter_mut(&mut self) -> std::slice::IterMut<'_, Self::Elem> {
        #[allow(clippy::into_iter_on_ref)]
        self.into_iter()
    }
}

/// Type that represents a bounding rect.
///
/// This rect is top-left inclusive, bottom-right exclusive. This means that a
/// rect with the same point for top-left and bottom-right won't have any points
/// contained within it.
///
/// # Example
/// Rect with top-left: (0,0) and bottom-right: (2, 2)
/// ```t
///   0 1 2
/// 0 ■ ■ □
/// 1 ■ ■ □
/// 2 □ □ □ <- bottom right
/// ```
///
/// (■ = contained, □ = out of bounds)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    top_left: Point,
    dimensions: (u32, u32),
}

impl Rect {
    /// Creates a new [`Rect`] from a [`Point`] and [`u32`]s.
    ///
    /// # Panics
    /// Panics if the coordinates of the bottom-right point of the rect would not fit into [`u32`]s.
    ///
    /// See [`Rect::try_new`] for a fallible alternative.
    #[inline]
    pub const fn new(top_left: Point, dimensions: (u32, u32)) -> Self {
        assert!(top_left.0.checked_add(dimensions.0).is_some());
        assert!(top_left.1.checked_add(dimensions.1).is_some());

        Self {
            top_left,
            dimensions,
        }
    }

    /// Creates a new [`Rect`] from a [`Point`] and [`u32`]s. Returns [`None`] if the coordinates
    /// of the bottom-right point of the rect would not fit into [`u32`]s.
    #[inline]
    pub const fn try_new(top_left: Point, dimensions: (u32, u32)) -> Option<Self> {
        if top_left.0.checked_add(dimensions.0).is_some()
            && top_left.1.checked_add(dimensions.1).is_some()
        {
            Some(Self {
                top_left,
                dimensions,
            })
        } else {
            None
        }
    }

    /// Creates a new [`Rect`] from a top-left [`Point`] and a bottom-right [`Point`] (exclusive).
    ///
    /// # Panics
    /// Panics if the coordinates of the top-left point of the [`Rect`] are greater than the coordinates
    /// of the bottom-right point of the [`Rect`] (i.e. the given top-left point is actually the bottom-right
    /// one, and vice-versa).
    ///  
    /// See [`Rect::try_new`] for a fallible alternative.
    #[inline]
    pub const fn from_extremes(top_left: Point, bottom_right: Point) -> Self {
        assert!(top_left.0 <= bottom_right.0);
        assert!(top_left.1 <= bottom_right.1);

        Self::new(
            top_left,
            (bottom_right.0 - top_left.0, bottom_right.1 - top_left.1),
        )
    }

    /// Creates a new empty [`Rect`] with a given top-left [`Point`].
    #[inline]
    pub const fn empty(top_left: Point) -> Self {
        Self::new(top_left, (0, 0))
    }

    /// Returns the top-left [`Point`] of this [`Rect`].
    #[inline]
    pub const fn top_left(&self) -> Point {
        self.top_left
    }

    /// Returns the bottom-right [`Point`] of this [`Rect`].
    #[inline]
    pub const fn bottom_right(&self) -> Point {
        (
            self.top_left.0 + self.dimensions.0,
            self.top_left.1 + self.dimensions.1,
        )
    }

    /// Returns the inclusive bottom-right [`Point`] of this [`Rect`]. If the [`Rect`] is empty,
    /// returns [`None`].
    #[inline]
    pub const fn inclusive_bottom_right(&self) -> Option<Point> {
        if self.is_empty() {
            None
        } else {
            Some((
                self.top_left.0 + self.dimensions.0 - 1,
                self.top_left.1 + self.dimensions.1 - 1,
            ))
        }
    }

    /// Returns the [`u32`]s of this [`Rect`].
    #[inline]
    pub const fn dimensions(&self) -> (u32, u32) {
        self.dimensions
    }

    /// Returns whether this [`Rect`] is empty (either width or height are zero) or not.
    pub const fn is_empty(&self) -> bool {
        self.dimensions.0 == 0 || self.dimensions.1 == 0
    }

    /// Returns the length of this [`Rect`], i.e. `width x height`.
    #[inline]
    pub const fn len(&self) -> u64 {
        self.dimensions.0 as u64 * self.dimensions.1 as u64
    }

    /// Returns whether this [`Rect`] contains a given point.
    #[inline]
    pub const fn contains(&self, point: Point) -> bool {
        (self.top_left.0 <= point.0)
            && (point.0 < self.top_left.0 + self.dimensions.0)
            && (self.top_left.1 <= point.1)
            && (point.1 < self.top_left.1 + self.dimensions.1)
    }

    /// Returns whether this [`Rect`] contains a given point relative to it's top-left point.
    #[inline]
    pub const fn contains_relative(&self, relative_point: Point) -> bool {
        (relative_point.0 < self.dimensions.0) && (relative_point.1 < self.dimensions.1)
    }

    /// Returns the absolute point equivalent to a point relative to the top-left point of this [`Rect`].
    #[inline]
    pub const fn abs_point_from_relative(&self, relative_point: Point) -> Point {
        let (top_left_x, top_left_y) = self.top_left;
        let x = top_left_x + relative_point.0;
        let y = top_left_y + relative_point.1;
        (x, y)
    }

    /// Returns whether this [`Rect`] contains another [`Rect`].
    #[inline]
    pub const fn contains_rect(&self, other: &Rect) -> bool {
        let other_br = other.bottom_right();

        !self.is_empty()
            && !other.is_empty()
            && self.contains(other.top_left)
            && self.contains((other_br.0 - 1, other_br.1 - 1))
    }

    /// Returns whether this [`Rect`] contains another [`Rect`] that is relative to the top-left point
    /// of this [`Rect`].
    #[inline]
    pub const fn contains_rect_relative(&self, other: &Rect) -> bool {
        let r = Rect::new((0, 0), self.dimensions);
        r.contains_rect(other)
    }

    /// Returns the absolute [`Rect`] equivalent to a [`Rect`] relative to the top-left point of this [`Rect`].
    #[inline]
    pub const fn abs_rect_from_relative(&self, other: Rect) -> Rect {
        let (top_left_x, top_left_y) = self.top_left;
        let (relative_x, relative_y) = other.top_left;
        let x = top_left_x + relative_x;
        let y = top_left_y + relative_y;
        Rect::new((x, y), other.dimensions)
    }

    /// Returns whether this [`Rect`] is completely below another [`Rect`].
    #[inline]
    pub const fn is_completely_below(&self, other: &Rect) -> bool {
        self.top_left.1 >= other.bottom_right().1
    }

    /// Returns whether this [`Rect`] is completely to the right of another [`Rect`].
    #[inline]
    pub const fn is_completely_to_the_right(&self, other: &Rect) -> bool {
        self.top_left.0 >= other.bottom_right().0
    }

    /// Returns whether this [`Rect`] overlaps with another [`Rect`].
    #[inline]
    pub const fn overlaps(&self, other: &Rect) -> bool {
        !(self.is_empty()
            || other.is_empty()
            || self.is_completely_below(other)
            || other.is_completely_below(self)
            || self.is_completely_to_the_right(other)
            || other.is_completely_to_the_right(self))
    }
}

#[inline(always)]
pub fn checked_size(width: u32, height: u32) -> usize {
    (width as usize)
        .checked_mul(height as usize)
        .expect("size should fit within usize")
}

/// Calculates an index from a `point` and a `width`: `point.1 * width + point.0`.
///
/// This has [no overhead](https://godbolt.org/z/fGPq71b41) if [`u32`] is smaller
/// than [`usize`].
///
/// # Panics
/// Panics if either
/// 1. A [`u32`] to [`usize`] conversion panics (number doesn't fit), _or..._
/// 2. The result overflows.
#[inline(always)]
pub fn index_point((x, y): Point, width: u32) -> usize {
    (y as usize)
        .checked_mul(width as usize)
        .and_then(|res| res.checked_add(x as usize))
        .expect("index calculation shouldn't overflow")
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use proptest::prelude::*;

    fn rect_strat() -> impl Strategy<Value = Rect> {
        (any::<Point>(), any::<(u32, u32)>()).prop_filter_map(
            "only valid rectangles accepted",
            |(top_left, dimensions)| Rect::try_new(top_left, dimensions),
        )
    }

    proptest! {
        #[test]
        fn rect_contains_rect(a in rect_strat(), b in rect_strat()) {
            if !b.is_empty() && a.contains(b.top_left()) && a.contains(b.inclusive_bottom_right().unwrap()) {
                prop_assert!(a.contains_rect(&b));
            } else {
                prop_assert!(!a.contains_rect(&b));
            }
        }
    }
}
