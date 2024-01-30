use super::Pixel;
use crate::util::macros::count_tts;
use bytemuck::{Pod, Zeroable};

macro_rules! impl_pixel {
    ($pixel:ident => $($field:ident),+) => {
        impl<C> Pixel for $pixel<C>
        where
            [C; count_tts!($($field)+)]: bytemuck::NoUninit,
            C: Pod,
        {
            type Channels = [C; count_tts!($($field)+)];

            #[inline(always)]
            fn new(channels: Self::Channels) -> Self {
                let [$($field),+] = channels;
                Self { $($field),+ }
            }

            #[inline(always)]
            fn channels(&self) -> &Self::Channels {
                bytemuck::must_cast_ref(self)
            }

            #[inline(always)]
            fn channels_mut(&mut self) -> &mut Self::Channels {
                bytemuck::must_cast_mut(self)
            }
        }
    };
    (tuple $pixel:ident => $($field:ident),+) => {
        impl<C> Pixel for $pixel<C>
        where
            [C; count_tts!($($field)+)]: bytemuck::NoUninit,
            C: Pod,
        {
            type Channels = [C; count_tts!($($field)+)];

            #[inline(always)]
            fn new(channels: Self::Channels) -> Self {
                let [$($field),+] = channels;
                Self($($field),+)
            }

            #[inline(always)]
            fn channels(&self) -> &Self::Channels {
                bytemuck::must_cast_ref(self)
            }

            #[inline(always)]
            fn channels_mut(&mut self) -> &mut Self::Channels {
                bytemuck::must_cast_mut(self)
            }
        }
    };
}

macro_rules! gen_pixel {
    ($name:ident => $($field:ident),+) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
        #[repr(C)]
        pub struct $name<C> {
            $(pub $field: C,)+
        }

        // SAFETY: this is ok because the types being generated are effectively just arrays with
        // named indexes, so if C is zeroable so is the type
        unsafe impl<C> Zeroable for $name<C> where C: Zeroable {}

        // SAFETY: same reasoning as above, just change zeroable for pod
        unsafe impl<C> Pod for $name<C> where C: Pod {}

        impl_pixel!($name => $($field),+);
    };
}

macro_rules! re_export {
    ($name:path) => {
        #[doc = "Re-export of the [`"]
        #[doc = std::stringify!($name)]
        #[doc = "`] pixel type from the [`rgb`] crate.\n\n"]
        pub use $name;
    };
    (alias $name:path) => {
        #[doc = "Re-export of the [`"]
        #[doc = std::stringify!($name)]
        #[doc = "`] pixel type alias from the [`rgb`] crate.\n\n"]
        pub use $name;
    };
}

re_export!(rgb::RGB);
impl_pixel!(RGB => r, g, b);

re_export!(rgb::RGBA);
impl_pixel!(RGBA => r, g, b, a);

re_export!(rgb::alt::BGR);
impl_pixel!(BGR => b, g, r);

re_export!(rgb::alt::BGRA);
impl_pixel!(BGRA => b, g, r, a);

re_export!(rgb::alt::Gray);
impl_pixel!(tuple Gray => g);

re_export!(rgb::alt::GrayAlpha);
impl_pixel!(tuple GrayAlpha => g, a);

re_export!(alias rgb::RGB8);
re_export!(alias rgb::RGB16);
re_export!(alias rgb::RGBA8);
re_export!(alias rgb::RGBA16);
re_export!(alias rgb::alt::BGR8);
re_export!(alias rgb::alt::BGR16);
re_export!(alias rgb::alt::GRAY8);
re_export!(alias rgb::alt::GRAY16);
re_export!(alias rgb::alt::GRAYA8);
re_export!(alias rgb::alt::GRAYA16);

gen_pixel!(CMY => c, m, y);
gen_pixel!(HSV => h, s, v);
gen_pixel!(HSL => h, s, l);
gen_pixel!(YCbCr => y, cb, cr);
