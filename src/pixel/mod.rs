/// Common pixel types.
pub mod common;
/// Pixel related iterators.
mod iter;

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

    /// Returns an iterator over the channels of this pixel.
    #[inline]
    fn channels_iter(&self) -> iter::Channels<'_, <<Self as Pixel>::Channels as Array>::Elem> {
        self.channels().iter()
    }

    /// Returns a mutable iterator over the channels of this pixel.
    #[inline]
    fn channels_mut_iter(
        &mut self,
    ) -> iter::ChannelsMut<'_, <<Self as Pixel>::Channels as Array>::Elem> {
        self.channels_mut().iter_mut()
    }

    /// Returns a pointer to the first channel.
    #[inline]
    fn channels_ptr(&self) -> *const <Self::Channels as Array>::Elem {
        (self.channels() as *const Self::Channels).cast()
    }

    /// Returns a mutable pointer to the first channel.
    #[inline]
    fn channels_mut_ptr(&mut self) -> *mut <Self::Channels as Array>::Elem {
        (self.channels_mut() as *mut Self::Channels).cast()
    }

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

    #[inline]
    fn channels(&self) -> &Self::Channels {
        self
    }

    #[inline]
    fn channels_mut(&mut self) -> &mut Self::Channels {
        self
    }

    #[inline]
    fn channels_iter(&self) -> iter::Channels<'_, <<Self as Pixel>::Channels as Array>::Elem> {
        self.iter()
    }

    #[inline]
    fn channels_mut_iter(
        &mut self,
    ) -> iter::ChannelsMut<'_, <<Self as Pixel>::Channels as Array>::Elem> {
        self.iter_mut()
    }

    #[inline]
    fn write_data<W>(&self, mut writer: W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        writer.write_all(bytemuck::bytes_of(self))
    }

    #[inline]
    fn channels_ptr(&self) -> *const <Self::Channels as Array>::Elem {
        (self.channels() as *const Self::Channels).cast()
    }

    #[inline]
    fn channels_mut_ptr(&mut self) -> *mut <Self::Channels as Array>::Elem {
        (self.channels_mut() as *mut Self::Channels).cast()
    }
}
