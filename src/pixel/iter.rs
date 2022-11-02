/// Iterator over the channels of a pixel.
pub type Channels<'pixel_ref, C> = std::slice::Iter<'pixel_ref, C>;
/// Mutable iterator over the channels of a pixel.
pub type ChannelsMut<'pixel_ref, C> = std::slice::IterMut<'pixel_ref, C>;
