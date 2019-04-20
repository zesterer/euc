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
    pub fn new([width, height]: [usize; 2], fill: T) -> Self {
        Self {
            items: vec![fill; width * height],
            size: [width, height],
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
    unsafe fn set(&mut self, [x, y]: [usize; 2], item: Self::Item) {
        let [width, _] = self.size;
        *self.items.get_unchecked_mut(y * width + x) = item;
    }

    #[inline(always)]
    unsafe fn get(&self, [x, y]: [usize; 2]) -> Self::Item {
        let [width, _] = self.size;
        self.items.get_unchecked(y * width + x).clone()
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
