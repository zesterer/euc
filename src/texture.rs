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
///
/// Targets necessarily require additional invariants to be upheld than textures for safe use. Because access to them
/// may be parallelised, it is essential that there is a 1:1 mapping between each index and a unique memory location.
/// If this is not upheld, Rust's 1 writer / many readers aliasing model may be broken. The `read_exclusive_unchecked`
/// and `write_exclusive_unchecked` methods may only be called by callers that have already ensured that nothing else
/// can access the target at the same time. In addition, the target must guarantee that no reads or writes escape
/// either method. This can be done by having each texel be an `UnsafeCell`.
pub trait Target: Texture<2, Index = usize> {
    /// Read a texel at the given assumed-valid index.
    ///
    /// # Safety
    ///
    /// If the index is invalid, undefined behaviour can be assumed to occur. Ensure that the index is valid before
    /// use. Access to this index *must* be exclusive to avoid undefined behaviour (i.e: nothing else may be reading or
    /// writing to this index during the duration of this call). The caller must enforce this through a lock or some
    /// other such mechanism with mutual exclusion properties.
    unsafe fn read_exclusive_unchecked(&self, index: [Self::Index; 2]) -> Self::Texel;

    /// Write a texel at the given assumed-valid index.
    ///
    /// # Safety
    ///
    /// If the index is invalid, undefined behaviour can be assumed to occur. Ensure that the index is valid before
    /// use. Access to this index *must* be exclusive to avoid undefined behaviour (i.e: nothing else may be reading or
    /// writing to this index during the duration of this call). The caller must enforce this through a lock or some
    /// other such mechanism with mutual exclusion properties.
    unsafe fn write_exclusive_unchecked(&self, index: [usize; 2], texel: Self::Texel);

    /// Write a texel at the given assumed-valid index.
    ///
    /// # Safety
    ///
    /// If the index is invalid, undefined behaviour can be assumed to occur. Ensure that the index is valid before
    /// use. Access to this index *must* be exclusive to avoid undefined behaviour (i.e: nothing else may be reading or
    /// writing to this index during the duration of this call).
    unsafe fn write_unchecked(&mut self, index: [usize; 2], texel: Self::Texel) {
        self.write_exclusive_unchecked(index, texel);
    }

    /// Write a texel at the given index.
    ///
    /// # Panics
    ///
    /// The behaviour of this function is *unspecified* (but not *undefined*) when the index is out of bounds. The
    /// implementation is free to panic, write to an entirely different texel, or do nothing.
    fn write(&mut self, [x, y]: [usize; 2], texel: Self::Texel) {
        if x < self.size()[0] && y < self.size()[1] {
            unsafe { self.write_unchecked([x, y], texel); }
        }
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
    unsafe fn read_exclusive_unchecked(&self, index: [Self::Index; 2]) -> Self::Texel { T::read_exclusive_unchecked(self, index) }
    unsafe fn write_exclusive_unchecked(&self, index: [usize; 2], texel: Self::Texel) { T::write_exclusive_unchecked(self, index, texel) }
    unsafe fn write_unchecked(&mut self, index: [usize; 2], texel: Self::Texel) { T::write_unchecked(self, index, texel) }
    fn write(&mut self, index: [usize; 2], texel: Self::Texel) { T::write(self, index, texel); }
    fn clear(&mut self, texel: Self::Texel) { T::clear(self, texel); }
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
    unsafe fn read_exclusive_unchecked(&self, _: [Self::Index; 2]) -> Self::Texel { panic!("Cannot read from an empty texture"); }
    unsafe fn write_exclusive_unchecked(&self, _: [usize; 2], _: Self::Texel) { panic!("Cannot write to an empty texture"); }
}

#[cfg(feature = "image")]
impl<P, C> Texture<2> for image_::ImageBuffer<P, C>
where
    P: image_::Pixel + Clone + 'static,
    C: core::ops::Deref<Target = [P::Subpixel]>,
{
    type Index = usize;
    type Texel = P;

    fn size(&self) -> [Self::Index; 2] {
        [self.width() as usize, self.height() as usize]
    }

    fn read(&self, [x, y]: [Self::Index; 2]) -> Self::Texel {
        self.get_pixel(x as u32, y as u32).clone()
    }
}

// #[cfg(feature = "image")]
// impl<P, C> Target for image_::ImageBuffer<P, C>
// where
//     P: image_::Pixel + 'static,
//     C: core::ops::DerefMut<Target = [P::Subpixel]>,
// {
//     fn write(&mut self, [x, y]: [usize; 2], texel: Self::Texel) {
//         self.put_pixel(x as u32, y as u32, texel);
//     }

//     unsafe fn write_unchecked(&mut self, [x, y]: [usize; 2], texel: Self::Texel) {
//         image_::GenericImage::unsafe_put_pixel(self, x as u32, y as u32, texel);
//     }
// }
