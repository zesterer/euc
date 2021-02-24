pub mod nearest;
pub mod linear;

pub use self::{
    nearest::Nearest,
    linear::Linear,
};

use crate::{
    texture::Texture,
    math::*,
};

/// A trait that describes a sampler of a texture.
///
/// Samplers use normalised coordinates (between 0 and 1) to sample textures. Often, samplers will combine this with
/// a sampling algorithm such as filtering or domain warping.
///
/// Please note that texture coordinate axes are, where possible, consistent with the underlying texture implementation
/// (i.e: +x and +y in sampler space correspond to the same directions as +x and +y in texture space). This behaviour
/// is equivalent to that of Vulkan's texture access API.
pub trait Sampler<const N: usize> {
    /// The type used to perform sampling.
    type Index: Clone;

    // The type the sampler emits when sampled.
    type Sample: Clone;

    /// The underlying texture accessed by this sampler.
    type Texture: Texture<N> + ?Sized;

    /// Access the underlying texture accessed by this sampler.
    fn raw_texture(&self) -> &Self::Texture;

    /// Sample the texture at the given index.
    ///
    /// # Panics
    ///
    /// The behaviour of this function is *unspecified* (but not *undefined*) when the index is out of bounds. The
    /// implementation is free to panic, or return any proper value. Alternatively, some implementers may use out of
    /// bounds access to implement special behaviours such as border colours or texture tiling.
    fn sample(&self, index: [Self::Index; N]) -> Self::Sample;

    /// Sample the texture at the given assumed-valid index.
    ///
    /// # Safety
    ///
    /// If the index is invalid, undefined behaviour can be assumed to occur. Ensure that the index is valid before
    /// use.
    unsafe fn sample_unchecked(&self, index: [Self::Index; N]) -> Self::Sample {
        self.sample(index)
    }
}
