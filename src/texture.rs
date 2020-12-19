/// A trait implemented by types that may be treated as textures.
pub trait Texture<const N: usize> {
    /// The type used to index into the texture.
    type Index: Clone;

    /// The type of texture elements.
    type Texel: Clone;

    /// Get the size of the texture in texels.
    ///
    /// # Safety
    ///
    /// The function should report a correct size (i.e: a size for which all bounded indices are valid). While this is
    /// not by itself a requirement for safe use, failure to do so may result in invalid texture indices being accessed
    /// by users of the texture.
    fn size(&self) -> [Self::Index; N];

    /// Get the texture axis with highest contiguous access times.
    ///
    /// The ordering of textures in memory can have a very significant impact on the cost of accessing them. It is
    /// typical for textures to be ordered first in rows (i.e: a principal x axis) and then columns but this is not
    /// always the case. This function allows the texture to signal to users what access patterns are most performant.
    ///
    /// The default implementation is a principal axis of x, which corresponds to the most common in-memory texture layouts.
    fn principal_axis(&self) -> usize { 0 }

    /// Read a texel at the given index.
    ///
    /// # Panics
    ///
    /// The behaviour of this function is *unspecified* (but not *undefined*) when the index is out of bounds. The
    /// implementation is free to panic, return an entirely different texel, or return texel data not in the texture at
    /// all.
    fn read(&self, index: [Self::Index; N]) -> Self::Texel;

    /// Read a texel at the given assumed-valid index.
    ///
    /// # Safety
    ///
    /// If the index is invalid, undefined behaviour can be assumed to occur. Ensure that the index is valid before
    /// use.
    unsafe fn read_unchecked(&self, index: [Self::Index; N]) -> Self::Texel {
        self.read(index)
    }
}

impl<'a, T: Texture<N>, const N: usize> Texture<N> for &'a T {
    type Index = T::Index;
    type Texel = T::Texel;
    fn size(&self) -> [Self::Index; N] { (**self).size() }
    fn read(&self, index: [Self::Index; N]) -> Self::Texel { (**self).read(index) }
    unsafe fn read_unchecked(&self, index: [Self::Index; N]) -> Self::Texel { (**self).read_unchecked(index) }
}

impl<'a, T: Texture<N>, const N: usize> Texture<N> for &'a mut T {
    type Index = T::Index;
    type Texel = T::Texel;
    fn size(&self) -> [Self::Index; N] { (**self).size() }
    fn read(&self, index: [Self::Index; N]) -> Self::Texel { (**self).read(index) }
    unsafe fn read_unchecked(&self, index: [Self::Index; N]) -> Self::Texel { (**self).read_unchecked(index) }
}

/// A trait implemented by 2-dimensional textures that may be treated as render targets.
pub trait Target: Texture<2, Index = usize> {
    /// Write a texel at the given index.
    ///
    /// # Panics
    ///
    /// The behaviour of this function is *unspecified* (but not *undefined*) when the index is out of bounds. The
    /// implementation is free to panic, write to an entirely different texel, or do nothing.
    fn write(&mut self, index: [usize; 2], texel: Self::Texel);

    /// Write a texel at the given assumed-valid index.
    ///
    /// # Safety
    ///
    /// If the index is invalid, undefined behaviour can be assumed to occur. Ensure that the index is valid before
    /// use.
    unsafe fn write_unchecked(&mut self, index: [usize; 2], texel: Self::Texel) {
        self.write(index, texel);
    }

    /// Clears the entire target with the given texel.
    fn clear(&mut self, texel: Self::Texel) {
        for y in 0..self.size()[1] {
            for x in 0..self.size()[0] {
                unsafe { self.write_unchecked([x, y], texel.clone()); }
            }
        }
    }
}

impl<'a, T: Target> Target for &'a mut T {
    fn write(&mut self, index: [usize; 2], texel: Self::Texel) { (**self).write(index, texel); }
    unsafe fn write_unchecked(&mut self, index: [usize; 2], texel: Self::Texel) { (**self).write_unchecked(index, texel); }
    fn clear(&mut self, texel: Self::Texel) { (**self).clear(texel); }
}

/// An always-empty texture. Useful as a placeholder for an unused target.
pub struct Empty<T>(core::marker::PhantomData<T>);

impl<T> Default for Empty<T> {
    fn default() -> Self {
        Self(core::marker::PhantomData)
    }
}

impl<J: Clone, const N: usize> Texture<N> for Empty<J> {
    type Index = usize;
    type Texel = J;
    fn size(&self) -> [Self::Index; N] { [0; N] }
    fn read(&self, _: [Self::Index; N]) -> Self::Texel { panic!("Cannot read from an empty texture"); }
}

impl<T: Clone> Target for Empty<T> {
    fn write(&mut self, _: [usize; 2], _: Self::Texel) { panic!("Cannot write to an empty texture"); }
}
