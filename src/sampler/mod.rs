use crate::{
    texture::Texture,
    math::*,
};
use core::{
    ops::Mul,
    marker::PhantomData,
};

/// A trait that describes a sampler of a texture.
///
/// Samplers use normalised coordinates (between 0 and 1) to sample textures. Often, samplers will combine this with
/// a sampling algorithm such as filtering or domain warping.
pub trait Sampler<const N: usize>
where
    Self::Index: Denormalize<<Self::Texture as Texture<N>>::Index>,
{
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
    /// implementation is free to panic, or return any proper value.
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

/// A sampler that uses nearest-neighbor sampling.
pub struct Nearest<T, I>(T, PhantomData<I>);

impl<T, I> Nearest<T, I> {
    /// Create a new
    pub fn new(texture: T) -> Self {
        Self(texture, PhantomData)
    }
}

impl<'a, T, I, const N: usize> Sampler<N> for Nearest<T, I>
where
    T: Texture<N>,
    I: Clone + Mul<Output = I> + Denormalize<T::Index>,
{
    type Index = I;

    type Sample = T::Texel;

    type Texture = T;

    #[inline(always)]
    fn raw_texture(&self) -> &Self::Texture { &self.0 }

    #[inline(always)]
    fn sample(&self, index: [Self::Index; N]) -> Self::Sample {
        self.raw_texture().read(I::denormalize_array(index, self.raw_texture().size()))
    }

    #[inline(always)]
    unsafe fn sample_unchecked(&self, index: [Self::Index; N]) -> Self::Sample {
        self.raw_texture().read_unchecked(I::denormalize_array(index, self.raw_texture().size()))
    }
}
