use crate::texture::{Texture, Target};
use alloc::vec::Vec;

/// A generic 1-dimensional buffer that may be used as a texture.
pub type Buffer1d<T> = Buffer<T, 2>;

/// A generic 2-dimensional buffer that may be used both as a texture and as a render target.
pub type Buffer2d<T> = Buffer<T, 2>;

/// A generic 3-dimensional buffer that may be used as a texture.
pub type Buffer3d<T> = Buffer<T, 3>;

/// A generic 4-dimensional buffer that may be used as a texture.
pub type Buffer4d<T> = Buffer<T, 4>;

/// A generic N-dimensional buffer that may be used both as a texture and as a render target.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Buffer<T, const N: usize> {
    size: [usize; N],
    items: Vec<T>,
}

impl<T, const N: usize> Buffer<T, N> {
    /// Create a new buffer with the given size, filled with duplicates of the given element.
    pub fn fill(size: [usize; N], item: T) -> Self where T: Clone {
        Self::fill_with(size, || item.clone())
    }

    /// Create a new buffer with the given size, filled by calling the function for each element.
    ///
    /// If your type implements [`Clone`], use [`Buffer::fill`] instead.
    pub fn fill_with<F: FnMut() -> T>(size: [usize; N], mut f: F) -> Self {
        let mut len = 1usize;
        (0..N).for_each(|i| len = len.checked_mul(size[i]).unwrap());
        Self {
            size,
            items: (0..len).map(|_| f()).collect(),
        }
    }

    /// Convert the given index into a linear index that can be used to index into the raw data of this buffer.
    #[inline(always)]
    pub fn linear_index(&self, index: [usize; N]) -> usize {
        let mut idx = 0;
        let mut factor = 1;
        (0..N).for_each(|i| {
            idx += index[i] * factor;
            factor *= self.size[i];
        });
        idx
    }

    /// View this buffer as a linear slice of elements.
    pub fn raw(&self) -> &[T] { &self.items }

    /// View this buffer as a linear mutable slice of elements.
    pub fn raw_mut(&mut self) -> &mut [T] { &mut self.items }
}

impl<T: Clone, const N: usize> Texture<N> for Buffer<T, N> {
    type Index = usize;

    type Texel = T;

    #[inline(always)]
    fn size(&self) -> [Self::Index; N] { self.size }

    #[inline(always)]
    fn read(&self, index: [Self::Index; N]) -> Self::Texel {
        self.items[self.linear_index(index)].clone()
    }

    #[inline(always)]
    unsafe fn read_unchecked(&self, index: [Self::Index; N]) -> Self::Texel {
        self.items.get_unchecked(self.linear_index(index)).clone()
    }
}

impl<T: Clone> Target for Buffer<T, 2> {
    #[inline(always)]
    fn write(&mut self, index: [usize; 2], texel: Self::Texel) {
        let idx = self.linear_index(index);
        self.items[idx] = texel;
    }

    #[inline(always)]
    unsafe fn write_unchecked(&mut self, index: [usize; 2], texel: Self::Texel) {
        let idx = self.linear_index(index);
        *self.items.get_unchecked_mut(idx) = texel;
    }

    fn clear(&mut self, texel: Self::Texel) {
        self.items.iter_mut().for_each(|item| *item = texel.clone());
    }
}
