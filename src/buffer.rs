use crate::texture::{Target, Texture};
use alloc::{boxed::Box, vec::Vec};
use core::cell::UnsafeCell;

/// A generic 1-dimensional buffer that may be used as a texture.
pub type Buffer1d<T> = Buffer<T, 2>;

/// A generic 2-dimensional buffer that may be used both as a texture and as a render target.
pub type Buffer2d<T> = Buffer<T, 2>;

/// A generic 3-dimensional buffer that may be used as a texture.
pub type Buffer3d<T> = Buffer<T, 3>;

/// A generic 4-dimensional buffer that may be used as a texture.
pub type Buffer4d<T> = Buffer<T, 4>;

/// A generic N-dimensional buffer that may be used both as a texture and as a render target.
#[derive(Debug)]
pub struct Buffer<T, const N: usize> {
    items: Box<[UnsafeCell<T>]>,
    size: [usize; N],
}

// SAFETY: Same behaviour as a slice upheld
unsafe impl<T: Send, const N: usize> Send for Buffer<T, N> {}
unsafe impl<T: Sync, const N: usize> Sync for Buffer<T, N> {}

impl<T, const N: usize> Buffer<T, N> {
    /// Create a new buffer with the given size, filled with duplicates of the given element.
    #[inline]
    pub fn fill(size: [usize; N], item: T) -> Self
    where
        T: Clone,
    {
        Self::fill_with(size, || item.clone())
    }

    /// Create a new buffer with the given size, filled by calling the function for each element.
    ///
    /// If your type implements [`Clone`], use [`Buffer::fill`] instead.
    #[inline]
    pub fn fill_with<F: FnMut() -> T>(size: [usize; N], mut f: F) -> Self {
        let mut len = 1usize;
        (0..N).for_each(|i| len = len.checked_mul(size[i]).unwrap());
        Self {
            size,
            items: (0..len)
                .map(|_| UnsafeCell::new(f()))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        }
    }

    /// Convert the given index into a linear index that can be used to index into the raw data of this buffer.
    #[inline]
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
    #[inline]
    pub fn raw(&self) -> &[T] {
        // SAFETY: Only `write_exclusive_unchecked` can violate the invariants
        unsafe { core::slice::from_raw_parts(self.items.as_ptr() as _, self.items.len()) }
    }

    /// View this buffer as a linear mutable slice of elements.
    #[inline]
    pub fn raw_mut(&mut self) -> &mut [T] {
        // SAFETY: We have &mut access
        unsafe { core::slice::from_raw_parts_mut(self.items.as_mut_ptr() as _, self.items.len()) }
    }

    /// Get a mutable reference to the item at the given index.
    ///
    /// # Panics
    ///
    /// This function will panic if the index is not within bounds.
    #[inline]
    pub fn get_mut(&mut self, index: [usize; N]) -> &mut T {
        let idx = self.linear_index(index);
        match self.items.get_mut(idx) {
            Some(item) => item.get_mut(),
            None => panic!(
                "Attempted to read buffer of size {:?} at out-of-bounds location {:?}",
                self.size, index
            ),
        }
    }

    /// Get a mutable reference to the item at the given assumed-valid index.
    ///
    /// # Safety
    ///
    /// Undefined behaviour will occur if the index is not within bounds.
    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, index: [usize; N]) -> &mut T {
        let idx = self.linear_index(index);
        self.items.get_unchecked_mut(idx).get_mut()
    }
}

impl<T> Buffer<T, 2> {
    #[inline]
    pub(crate) fn linear_index2(&self, x: usize, y: usize) -> usize {
        y * self.size[0] + x
    }
}

impl<T: Clone, const N: usize> Texture<N> for Buffer<T, N> {
    type Index = usize;

    type Texel = T;

    #[inline]
    fn size(&self) -> [Self::Index; N] {
        self.size
    }

    #[inline]
    fn read(&self, index: [Self::Index; N]) -> Self::Texel {
        let item = self.items.get(self.linear_index(index)).unwrap_or_else(|| {
            panic!(
                "Attempted to read buffer of size {:?} at out-of-bounds location {:?}",
                self.size(),
                index
            )
        });
        // SAFETY: Invariants can only be violated by `write_exclusive_unchecked`
        unsafe { (*item.get()).clone() }
    }

    #[inline]
    unsafe fn read_unchecked(&self, index: [Self::Index; N]) -> Self::Texel {
        let item = self.items.get_unchecked(self.linear_index(index));
        // SAFETY: Invariants can only be violated by `write_exclusive_unchecked`
        unsafe { (*item.get()).clone() }
    }
}

impl<T: Clone> Target for Buffer<T, 2> {
    #[inline]
    unsafe fn read_exclusive_unchecked(&self, x: usize, y: usize) -> Self::Texel {
        let item = self.items.get_unchecked(self.linear_index2(x, y));
        // SAFETY: Invariants can only be violated by `write_exclusive_unchecked`
        unsafe { (*item.get()).clone() }
    }

    #[inline]
    unsafe fn write_exclusive_unchecked(&self, x: usize, y: usize, texel: Self::Texel) {
        let item = self.items.get_unchecked(self.linear_index2(x, y));
        // This is safe to do provided the caller has guaranteed exclusive access to the texels being written to, as
        // per the contractual obligations of this method.
        unsafe {
            item.get().write(texel);
        }
    }

    #[inline]
    unsafe fn write_unchecked(&mut self, x: usize, y: usize, texel: Self::Texel) {
        let idx = self.linear_index2(x, y);
        *self.items.get_unchecked_mut(idx) = UnsafeCell::new(texel);
    }

    #[inline]
    fn write(&mut self, x: usize, y: usize, texel: Self::Texel) {
        let idx = self.linear_index2(x, y);
        self.items[idx] = UnsafeCell::new(texel);
    }

    #[inline]
    fn clear(&mut self, texel: Self::Texel) {
        self.items
            .iter_mut()
            .for_each(|item| *item = UnsafeCell::new(texel.clone()));
    }
}
