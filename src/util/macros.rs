macro_rules! replace_expr {
    ($_t:tt $sub:expr) => {
        $sub
    };
}
pub(crate) use replace_expr;

macro_rules! count_tts {
    ($($tts:tt)*) => {0usize $(+ crate::util::macros::replace_expr!($tts 1usize))*};
}
pub(crate) use count_tts;

macro_rules! debug_assertions {
    (on => $debug:expr, off => $not_debug:expr) => {{
        if cfg!(debug_assertions) {
            $debug
        } else {
            $not_debug
        }
    }};
}
pub(crate) use debug_assertions;

/// Given an expression `$e`, this expands to calling `unwrap` on `$e` when `debug_assertions` is
/// on, and `unwrap_unchecked` when it's not.
macro_rules! dbg_unwrap_unchecked {
    ($e:expr) => {{
        crate::util::macros::debug_assertions! {
            on => $e.unwrap(),
            off => $e.unwrap_unchecked()
        }
    }};
}
pub(crate) use dbg_unwrap_unchecked;
