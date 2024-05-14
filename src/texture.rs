use super::sampler::{Linear, Nearest};
use core::marker::PhantomData;

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

    /// Get the texture's preferred access order, if it has one.
    ///
    /// Texture data is generally laid out in memory in such a way that iteration over a particular axis is preferred
    /// over others. For example, it is typical for framebuffers to be laid out in rows of columns, rather than columns
    /// of rows. As such, it is more performant to iterate over columns and then rows to take maximum advantage of the
    /// CPU's cache.
    ///
    /// You can use this function to switch between different iteration strategies to improve performance.
    ///
    /// In most cases, the preferred axes will be `[0, 1]` (i.e: sequential accesses to texels nearby in the x axis
    /// will be much faster than those in the y axis).
    fn preferred_axes(&self) -> Option<[usize; N]> {
        None
    }

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

    /// Create a linearly (bilinear or trilinear, if the texture is 2D or 3D) interpolated (i.e: filtered) sampler from
    /// this texture.
    ///
    /// See [`Linear`].
    fn linear(self) -> Linear<Self>
    where
        Self: Texture<2, Index = usize> + Sized,
    {
        assert!(
            <Self as Texture<2>>::size(&self)[0] >= 1 && <Self as Texture<2>>::size(&self)[1] >= 1,
            "Linearly-interpolated texture cannot have no size",
        );
        Linear(self, PhantomData)
    }

    /// Create a nearest-neighbour (i.e: unfiltered) sampler from this texture.
    ///
    /// See [`Nearest`].
    fn nearest(self) -> Nearest<Self>
    where
        Self: Sized,
    {
        Nearest {
            texture: self,
            phantom: PhantomData,
        }
    }

    /// Map the texels of this texture to another type using a mapping function.
    fn map<F, U>(self, f: F) -> Map<Self, F, U>
    where
        F: Fn(Self::Texel) -> U,
        Self: Sized,
    {
        Map {
            tex: self,
            f,
            phantom: PhantomData,
        }
    }
}

impl<'a, T: Texture<N>, const N: usize> Texture<N> for &'a T {
    type Index = T::Index;
    type Texel = T::Texel;
    #[inline(always)]
    fn size(&self) -> [Self::Index; N] {
        (**self).size()
    }
    #[inline(always)]
    fn preferred_axes(&self) -> Option<[usize; N]> {
        (**self).preferred_axes()
    }
    #[inline(always)]
    fn read(&self, index: [Self::Index; N]) -> Self::Texel {
        (**self).read(index)
    }
    #[inline(always)]
    unsafe fn read_unchecked(&self, index: [Self::Index; N]) -> Self::Texel {
        (**self).read_unchecked(index)
    }
}

impl<'a, T: Texture<N>, const N: usize> Texture<N> for &'a mut T {
    type Index = T::Index;
    type Texel = T::Texel;
    #[inline(always)]
    fn size(&self) -> [Self::Index; N] {
        (**self).size()
    }
    #[inline(always)]
    fn preferred_axes(&self) -> Option<[usize; N]> {
        (**self).preferred_axes()
    }
    #[inline(always)]
    fn read(&self, index: [Self::Index; N]) -> Self::Texel {
        (**self).read(index)
    }
    #[inline(always)]
    unsafe fn read_unchecked(&self, index: [Self::Index; N]) -> Self::Texel {
        (**self).read_unchecked(index)
    }
}

#[derive(Debug)]
pub struct Map<T, F, U> {
    tex: T,
    f: F,
    phantom: PhantomData<U>,
}

impl<T: Copy, F: Copy, U> Copy for Map<T, F, U> {}
impl<T: Clone, F: Clone, U> Clone for Map<T, F, U> {
    fn clone(&self) -> Self {
        Self {
            tex: self.tex.clone(),
            f: self.f.clone(),
            phantom: PhantomData,
        }
    }
}

impl<T: Texture<N>, U: Clone, F: Fn(T::Texel) -> U, const N: usize> Texture<N> for Map<T, F, U> {
    type Index = T::Index;
    type Texel = U;
    #[inline(always)]
    fn size(&self) -> [Self::Index; N] {
        self.tex.size()
    }
    #[inline(always)]
    fn preferred_axes(&self) -> Option<[usize; N]> {
        self.tex.preferred_axes()
    }
    #[inline(always)]
    fn read(&self, index: [Self::Index; N]) -> Self::Texel {
        (self.f)(self.tex.read(index))
    }
    #[inline(always)]
    unsafe fn read_unchecked(&self, index: [Self::Index; N]) -> Self::Texel {
        (self.f)(self.tex.read_unchecked(index))
    }
}

// impl<'a, T: Clone, F: Fn([usize; N]) -> T, const N: usize> Texture<N> for (F, [usize; N], PhantomData<T>) {
//     type Index = usize;
//     type Texel = T;
//     fn size(&self) -> [Self::Index; N] { self.1 }
//     fn read(&self, index: [Self::Index; N]) -> Self::Texel {
//         for i in 0..N {
//             assert!(index[i] < self.1[i]);
//         }
//         self.0(index)
//     }
//     unsafe fn read_unchecked(&self, index: [Self::Index; N]) -> Self::Texel { self.0(index) }
// }

/// A trait implemented by 2-dimensional textures that may be treated as render targets.
///
/// Targets necessarily require additional invariants to be upheld than textures for safe use. Because access to them
/// may be parallelised, it is essential that there is a 1:1 mapping between each index and a unique memory location.
/// If this is not upheld, Rust's one writer / many readers aliasing model may be broken. The
/// `read_exclusive_unchecked` and `write_exclusive_unchecked` methods may only be invoked by callers that have already
/// ensured that nothing else can access the target at the same time. In addition, the target must guarantee that no
/// reads or writes escape either method. This can be done by having each texel be accessed through an `UnsafeCell`.
pub trait Target: Texture<2, Index = usize> {
    /// Read a texel at the given assumed-valid index.
    ///
    /// # Safety
    ///
    /// If the index is invalid, undefined behaviour can be assumed to occur. Ensure that the index is valid before
    /// use. Access to this index *must* be exclusive to avoid undefined behaviour (i.e: nothing else may be reading or
    /// writing to this index during the duration of this call). The caller must enforce this through a lock or some
    /// other such mechanism with mutual exclusion properties. A sure-fire way to ensure that access is exclusive is to
    /// first obtain an owned buffer or a mutable reference to one since both guarantee exclusivity.
    unsafe fn read_exclusive_unchecked(&self, x: usize, y: usize) -> Self::Texel;

    /// Write a texel at the given assumed-valid index.
    ///
    /// # Safety
    ///
    /// If the index is invalid, undefined behaviour can be assumed to occur. Ensure that the index is valid before
    /// use. Access to this index *must* be exclusive to avoid undefined behaviour (i.e: nothing else may be reading or
    /// writing to this index during the duration of this call). The caller must enforce this through a lock or some
    /// other such mechanism with mutual exclusion properties. A sure-fire way to ensure that access is exclusive is to
    /// first obtain an owned buffer or a mutable reference to one since both guarantee exclusivity.
    unsafe fn write_exclusive_unchecked(&self, x: usize, y: usize, texel: Self::Texel);

    /// Write a texel at the given assumed-valid index.
    ///
    /// # Safety
    ///
    /// If the index is invalid, undefined behaviour can be assumed to occur. Ensure that the index is valid before
    /// use.
    #[inline]
    unsafe fn write_unchecked(&mut self, x: usize, y: usize, texel: Self::Texel) {
        self.write_exclusive_unchecked(x, y, texel);
    }

    /// Write a texel at the given index.
    ///
    /// # Panics
    ///
    /// The behaviour of this function is *unspecified* (but not *undefined*) when the index is out of bounds. The
    /// implementation is free to panic, write to an entirely different texel, or do nothing.
    #[inline]
    fn write(&mut self, x: usize, y: usize, texel: Self::Texel) {
        if x < self.size()[0] && y < self.size()[1] {
            unsafe {
                self.write_unchecked(x, y, texel);
            }
        }
    }

    /// Clears the entire target with the given texel.
    #[inline]
    fn clear(&mut self, texel: Self::Texel) {
        for y in 0..self.size()[1] {
            for x in 0..self.size()[0] {
                unsafe {
                    self.write_unchecked(x, y, texel.clone());
                }
            }
        }
    }
}

impl<T: Target> Target for &mut T {
    #[inline(always)]
    unsafe fn read_exclusive_unchecked(&self, x: usize, y: usize) -> Self::Texel {
        T::read_exclusive_unchecked(self, x, y)
    }
    #[inline(always)]
    unsafe fn write_exclusive_unchecked(&self, x: usize, y: usize, texel: Self::Texel) {
        T::write_exclusive_unchecked(self, x, y, texel)
    }
    #[inline(always)]
    unsafe fn write_unchecked(&mut self, x: usize, y: usize, texel: Self::Texel) {
        T::write_unchecked(self, x, y, texel)
    }
    #[inline(always)]
    fn write(&mut self, x: usize, y: usize, texel: Self::Texel) {
        T::write(self, x, y, texel);
    }
    #[inline(always)]
    fn clear(&mut self, texel: Self::Texel) {
        T::clear(self, texel);
    }
}

/// An always-empty texture. Useful as a placeholder for an unused target.
pub struct Empty<T>(core::marker::PhantomData<T>);

impl<T> Empty<T> {
    pub const fn new() -> Self {
        Self(core::marker::PhantomData)
    }
}

impl<T> Default for Empty<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone, const N: usize> Texture<N> for Empty<T> {
    type Index = usize;
    type Texel = T;
    #[inline(always)]
    fn size(&self) -> [Self::Index; N] {
        [0; N]
    }
    #[inline]
    fn read(&self, _: [Self::Index; N]) -> Self::Texel {
        panic!("Cannot read from an empty texture");
    }
}

impl<T: Clone + Default> Target for Empty<T> {
    #[inline(always)]
    unsafe fn read_exclusive_unchecked(&self, _: usize, _: usize) -> Self::Texel {
        T::default()
    }
    #[inline(always)]
    unsafe fn write_exclusive_unchecked(&self, _: usize, _: usize, _: Self::Texel) {}
}

#[cfg(feature = "image")]
impl<P, C> Texture<2> for image::ImageBuffer<P, C>
where
    P: image::Pixel + Clone + 'static,
    C: core::ops::Deref<Target = [P::Subpixel]>,
{
    type Index = usize;
    type Texel = P;

    #[inline(always)]
    fn size(&self) -> [Self::Index; 2] {
        [self.width() as usize, self.height() as usize]
    }

    #[inline(always)]
    fn preferred_axes(&self) -> Option<[usize; 2]> {
        Some([0, 1])
    }

    #[inline(always)]
    fn read(&self, [x, y]: [Self::Index; 2]) -> Self::Texel {
        *self.get_pixel(x as u32, y as u32)
    }
}

// #[cfg(feature = "image")]
// impl<P, C> Target for image::ImageBuffer<P, C>
// where
//     P: image::Pixel + 'static,
//     C: core::ops::DerefMut<Target = [P::Subpixel]>,
// {
//     fn write(&mut self, [x, y]: [usize; 2], texel: Self::Texel) {
//         self.put_pixel(x as u32, y as u32, texel);
//     }

//     unsafe fn write_unchecked(&mut self, [x, y]: [usize; 2], texel: Self::Texel) {
//         image::GenericImage::unsafe_put_pixel(self, x as u32, y as u32, texel);
//     }
// }
