use super::Pixel;
use crate::util::macros::count_tts;

macro_rules! impl_pixel {
    ($pixel:ident => $($field:ident),+) => {
        impl<C> Pixel for $pixel<C>
        where
            [C; count_tts!($($field)+)]: bytemuck::NoUninit,
        {
            type Channels = [C; count_tts!($($field)+)];

            fn new(channels: Self::Channels) -> Self {
                let [$($field),+] = channels;
                Self { $($field),+ }
            }

            #[inline]
            fn channels(&self) -> &Self::Channels {
                // SAFETY: this macro is only called on Repr(C) types with their channels
                // as their only fields
                unsafe {
                    (self as *const Self)
                        .cast::<Self::Channels>()
                        .as_ref()
                        .unwrap_unchecked()
                }
            }

            #[inline]
            fn channels_mut(&mut self) -> &mut Self::Channels {
                // SAFETY: see above.
                unsafe {
                    (self as *mut Self)
                        .cast::<Self::Channels>()
                        .as_mut()
                        .unwrap_unchecked()
                }
            }

            #[inline]
            fn write_data<W>(&self, writer: W) -> std::io::Result<()>
            where
                W: std::io::Write,
            {
                self.channels().write_data(writer)
            }
        }
    };
    (tuple $pixel:ident => $($field:ident),+) => {
        impl<C> Pixel for $pixel<C>
        where
            [C; count_tts!($($field)+)]: bytemuck::NoUninit,
        {
            type Channels = [C; count_tts!($($field)+)];

            fn new(channels: Self::Channels) -> Self {
                let [$($field),+] = channels;
                Self($($field),+)
            }

            #[inline]
            fn channels(&self) -> &Self::Channels {
                // SAFETY: this macro is only called on Repr(C) types with their channels
                // as their only fields
                unsafe {
                    (self as *const Self)
                        .cast::<Self::Channels>()
                        .as_ref()
                        .unwrap_unchecked()
                }
            }

            #[inline]
            fn channels_mut(&mut self) -> &mut Self::Channels {
                // SAFETY: see above.
                unsafe {
                    (self as *mut Self)
                        .cast::<Self::Channels>()
                        .as_mut()
                        .unwrap_unchecked()
                }
            }

            #[inline]
            fn write_data<W>(&self, writer: W) -> std::io::Result<()>
            where
                W: std::io::Write,
            {
                self.channels().write_data(writer)
            }
        }
    };
}

macro_rules! gen_pixel {
    ($name:ident => $($field:ident),+) => {
        #[derive(Debug, Clone, PartialEq, Eq, Default)]
        #[repr(C)]
        pub struct $name<C> {
            $($field: C,)+
        }

        impl_pixel!($name => $($field),+);
    };
    (tuple $name:ident => $($field:ident),+) => {
        #[derive(Debug, Clone, PartialEq, Eq, Default)]
        #[repr(C)]
        pub struct $name<C> {
            $($field: C,)+
        }

        impl_pixel!(tuple $name => $($field),+);
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
