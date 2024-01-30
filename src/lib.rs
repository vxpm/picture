//! # Picture
//! A fast and flexible image manipulation crate.
//!
//! # Quick Start
//! Take a look at the [`prelude`] for the most important items. Start with the [`Pixel`][prelude::Pixel],
//! [`Img`][prelude::Img] and [`ImgMut`][prelude::ImgMut] traits. Then, the [`ImgBuf`][prelude::ImgBuf] type.
#![cfg_attr(feature = "unstable", feature(trusted_len))]
#![cfg_attr(feature = "unstable", feature(iter_advance_by))]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(clippy::missing_safety_doc)]
#![warn(
    clippy::bool_to_int_with_if,
    clippy::borrow_as_ptr,
    clippy::case_sensitive_file_extension_comparisons,
    clippy::cast_lossless,
    clippy::cast_ptr_alignment,
    clippy::checked_conversions,
    clippy::cloned_instead_of_copied,
    clippy::copy_iterator,
    clippy::default_union_representation,
    clippy::deref_by_slicing,
    clippy::doc_link_with_quotes,
    clippy::doc_markdown,
    clippy::empty_drop,
    clippy::empty_structs_with_brackets,
    clippy::enum_glob_use,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_deref_methods,
    clippy::explicit_into_iter_loop,
    clippy::filter_map_next,
    clippy::flat_map_option,
    clippy::float_cmp,
    clippy::float_cmp_const,
    clippy::fn_params_excessive_bools,
    clippy::fn_to_numeric_cast_any,
    clippy::format_push_string,
    clippy::if_then_some_else_none,
    clippy::ignored_unit_patterns,
    clippy::implicit_clone,
    clippy::inefficient_to_string,
    clippy::invalid_upcast_comparisons,
    clippy::large_digit_groups,
    clippy::large_stack_arrays,
    clippy::large_types_passed_by_value,
    clippy::linkedlist,
    clippy::lossy_float_literal,
    clippy::macro_use_imports,
    clippy::manual_assert,
    clippy::manual_instant_elapsed,
    clippy::manual_let_else,
    clippy::manual_ok_or,
    clippy::manual_ok_or,
    clippy::manual_string_new,
    clippy::map_unwrap_or,
    clippy::match_bool,
    clippy::mem_forget,
    clippy::mismatching_type_param_order,
    clippy::multiple_inherent_impl,
    clippy::mut_mut,
    clippy::mutex_atomic,
    clippy::needless_bitwise_bool,
    clippy::needless_borrow,
    clippy::needless_continue,
    clippy::needless_for_each,
    clippy::needless_raw_string_hashes,
    clippy::needless_raw_strings,
    clippy::negative_feature_names,
    clippy::no_mangle_with_rust_abi,
    clippy::non_send_fields_in_send_ty,
    clippy::option_option,
    clippy::partial_pub_fields,
    clippy::ptr_cast_constness,
    clippy::range_minus_one,
    clippy::rc_mutex,
    clippy::redundant_else,
    clippy::redundant_feature_names,
    clippy::ref_option_ref,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_functions_in_if_condition,
    clippy::semicolon_if_nothing_returned,
    clippy::single_char_lifetime_names,
    clippy::single_match_else,
    clippy::string_add,
    clippy::string_add_assign,
    clippy::string_lit_chars_any,
    clippy::string_to_string,
    clippy::struct_field_names,
    clippy::transmute_ptr_to_ptr,
    clippy::trivially_copy_pass_by_ref,
    clippy::unnested_or_patterns,
    clippy::unreadable_literal,
    clippy::unsafe_derive_deserialize,
    clippy::unused_self,
    clippy::verbose_file_reads,
    clippy::zero_sized_map_values,
    nonstandard_style
)]

use buffer::common::CommonImgBuf;
use formats::{png::Decoder, CommonImgDecoder};
use std::path::Path;
use thiserror::Error;

/// [`ImgBuf`][buffer::ImgBuf] and everything related to it.
pub mod buffer;
/// Modules related to common image formats.
#[cfg(feature = "formats")]
pub mod formats;
/// [`Pixel`][pixel::Pixel] trait and common pixel formats.
pub mod pixel;
/// Image processing, like resizing and blurring.
pub mod processing;
/// Overall utilities.
pub mod util;
/// [`Img`][view::Img] and [`ImgMut`][view::ImgMut] traits.
pub mod view;
/// Re-exports the most useful items of the crate.
pub mod prelude {
    use super::*;

    pub use super::Point;
    pub use crate::Rect;
    pub use buffer::{
        common::{Rgb16Img, Rgb8Img, Rgba16Img, Rgba8Img},
        ImgBuf,
    };
    pub use pixel::{
        common::{RGB, RGB16, RGB8, RGBA, RGBA16, RGBA8},
        Pixel,
    };
    pub use view::{Img, ImgMut};
}

pub type Point = (u32, u32);

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

#[cfg(test)]
impl proptest::arbitrary::Arbitrary for Rect {
    type Parameters = ();
    type Strategy = proptest::strategy::BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        use proptest::prelude::*;
        (any::<Point>(), any::<Point>())
            .prop_filter_map(
                "only valid rectangles accepted",
                |(top_left, dimensions)| Rect::try_new(top_left, dimensions),
            )
            .boxed()
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use crate::prelude::*;
    use proptest::prelude::*;

    #[cfg(not(miri))]
    #[test]
    fn compile_tests() {
        let t = trybuild::TestCases::new();
        t.compile_fail("compile-tests/*.rs");
    }

    proptest! {
        #[cfg(not(miri))]
        #[test]
        fn rect_contains_rect(a: Rect, b: Rect) {
            if !b.is_empty() && a.contains(b.top_left()) && a.contains(b.inclusive_bottom_right().unwrap()) {
                prop_assert!(a.contains_rect(&b));
            } else {
                prop_assert!(!a.contains_rect(&b));
            }
        }
    }
}
