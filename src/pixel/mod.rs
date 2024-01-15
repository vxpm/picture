/// Common pixel types.
pub mod common;

use crate::util::Array;
use bytemuck::NoUninit;

/// Trait for types that represent a Pixel.
///
/// [`Pixel`] contains a [`Channels`][Pixel::Channels] associated type that is an array of
/// it's channel type.
///
/// This trait has a blanket implementation for all arrays of [`bytemuck::NoUninit`]s which
/// covers the [`Channels`][Pixel::Channels] = `Self` case (since `Channels` must be an array)
/// where `write_data` is trivially implemented as simply writing the bytes of the [`NoUninit`].
pub trait Pixel {
    /// Array that represents the channels and, therefore, specifies the type of a channel and the channel count.
    type Channels: Array;

    /// Creates a new pixel from it's channels.
    fn new(channels: Self::Channels) -> Self;

    /// Returns a reference to the channels of this pixel.
    fn channels(&self) -> &Self::Channels;

    /// Returns a mutable reference to the channels of this pixel.
    fn channels_mut(&mut self) -> &mut Self::Channels;

    /// Writes the data of this pixel to a [writer][std::io::Write].
    fn write_data<W>(&self, writer: W) -> std::io::Result<()>
    where
        W: std::io::Write;
}

impl<T, const SIZE: usize> Pixel for [T; SIZE]
where
    Self: NoUninit,
{
    type Channels = Self;

    fn new(channels: Self::Channels) -> Self {
        channels
    }

    #[inline(always)]
    fn channels(&self) -> &Self::Channels {
        self
    }

    #[inline(always)]
    fn channels_mut(&mut self) -> &mut Self::Channels {
        self
    }

    #[inline(always)]
    fn write_data<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        writer.write_all(bytemuck::bytes_of(self))
    }
}
