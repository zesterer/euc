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

/// A sampler that clamps the index's components to the 0.0 <= x <= 1.0 range.
#[derive(Copy, Clone)]
pub struct Clamped<S>(S);

impl<S> Clamped<S> {
    pub fn new(sampler: S) -> Self {
        Self(sampler)
    }
}

impl<S: Sampler<N, Index = f32>, const N: usize> Sampler<N> for Clamped<S> {
    type Index = S::Index;
    type Sample = S::Sample;
    type Texture = S::Texture;

    fn raw_texture(&self) -> &Self::Texture { self.0.raw_texture() }
    fn sample(&self, index: [Self::Index; N]) -> Self::Sample {
        let index = index.map(|e| e.max(0.0).min(1.0));
        self.0.sample(index)
    }
    unsafe fn sample_unchecked(&self, index: [Self::Index; N]) -> Self::Sample {
        let index = index.map(|e| e.max(0.0).min(1.0));
        self.0.sample_unchecked(index)
    }
}
