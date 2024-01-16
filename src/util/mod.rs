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
