use crate::{
    texture::Texture,
    math::*,
};
use core::{
    ops::{Div, Deref},
    marker::PhantomData,
};

/// A trait implemented by texture samplers.
pub trait Sampler<const N: usize>
where
    Self::Index: Truncate<<Self::Texture as Texture<N>>::Index>,
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

impl<'a, T: Deref, I, const N: usize> Sampler<N> for Nearest<T, I>
where
    T::Target: Texture<N>,
    I: Clone + Div<Output = I> + Truncate<<T::Target as Texture<N>>::Index>,
{
    type Index = I;

    type Sample = <T::Target as Texture<N>>::Texel;

    type Texture = T::Target;

    #[inline(always)]
    fn raw_texture(&self) -> &Self::Texture { &self.0 }

    #[inline(always)]
    fn sample(&self, mut index: [Self::Index; N]) -> Self::Sample {
        let size = self.raw_texture().size();
        (0..N).for_each(|i| index[i] = index[i].clone() / I::detruncate(size[i].clone()));
        self.raw_texture().read(index.map(|x| x.truncate()))
    }

    #[inline(always)]
    unsafe fn sample_unchecked(&self, index: [Self::Index; N]) -> Self::Sample {
        self.raw_texture().read_unchecked(index.map(|x| x.truncate()))
    }
}
