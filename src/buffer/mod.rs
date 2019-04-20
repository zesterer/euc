use core::fmt;

#[cfg(feature = "nightly")]
use alloc::prelude::*;

use crate::Target;

/// A 2-dimensional buffer.
///
/// This type may be used to contain colour data, depth data, or arbitrary pixel data.
#[derive(Clone)]
pub struct Buffer2d<T> {
    items: Vec<T>,
    size: [usize; 2],
}

impl<T: Clone> Buffer2d<T> {
    pub fn new(size: [usize; 2], fill: T) -> Self {
        Self {
            items: vec![fill; size[0] * size[1]],
            size,
        }
    }
}

impl<T: Clone> Target for Buffer2d<T> {
    type Item = T;

    #[inline(always)]
    fn size(&self) -> [usize; 2] {
        self.size
    }

    #[inline(always)]
    unsafe fn set(&mut self, pos: [usize; 2], item: Self::Item) {
        *self.items.get_unchecked_mut(pos[1] * self.size[0] + pos[0]) = item;
    }

    #[inline(always)]
    unsafe fn get(&self, pos: [usize; 2]) -> &Self::Item {
        &self.items.get_unchecked(pos[1] * self.size[0] + pos[0])
    }

    fn clear(&mut self, fill: Self::Item) {
        for item in &mut self.items {
            *item = fill.clone();
        }
    }
}

impl<T> AsRef<[T]> for Buffer2d<T> {
    fn as_ref(&self) -> &[T] {
        &self.items
    }
}

impl<T> AsMut<[T]> for Buffer2d<T> {
    fn as_mut(&mut self) -> &mut [T] {
        &mut self.items
    }
}

impl<T> fmt::Debug for Buffer2d<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Buffer2d(dimensions: {:?})", self.size)
    }
}
