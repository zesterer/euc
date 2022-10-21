use super::*;
use core::{
    ops::{Add, Mul},
    marker::PhantomData,
};

#[cfg(feature = "micromath")]
use micromath_::F32Ext;

/// A sampler that uses nearest-neighbor sampling.
pub struct Linear<T, I = f32>(pub(crate) T, pub(crate) PhantomData<I>);

impl<'a, T> Sampler<2> for Linear<T, f32>
where
    T: Texture<2, Index = usize>,
    T::Texel: Mul<f32, Output = T::Texel> + Add<Output = T::Texel>,
{
    type Index = f32;

    type Sample = T::Texel;

    type Texture = T;

    #[inline(always)]
    fn raw_texture(&self) -> &Self::Texture { &self.0 }

    #[inline(always)]
    fn sample(&self, mut index: [Self::Index; 2]) -> Self::Sample {
        assert!(index[0] <= 1.0, "{:?}", index);
        assert!(index[1] <= 1.0, "{:?}", index);

        let size = self.raw_texture().size();
        let size_f32 = size.map(|e| e as f32);
        // Index in texture coordinates
        let index_tex = [index[0].fract() * size_f32[0], index[1].fract() * size_f32[1]];
        // Find texel sample coordinates
        let posi = index_tex.map(|e| e.trunc() as usize);
        // Find interpolation values
        let fract = index_tex.map(|e| e.fract());

        assert!(posi[0] < size[0], "pos: {:?}, sz: {:?}, idx: {:?}", posi, size, index);
        assert!(posi[1] < size[1], "pos: {:?}, sz: {:?}, idx: {:?}", posi, size, index);

        let t00 = self.raw_texture().read([(posi[0] + 0).min(size[0] - 1), (posi[1] + 0).min(size[1] - 1)]);
        let t10 = self.raw_texture().read([(posi[0] + 1).min(size[0] - 1), (posi[1] + 0).min(size[1] - 1)]);
        let t01 = self.raw_texture().read([(posi[0] + 0).min(size[0] - 1), (posi[1] + 1).min(size[1] - 1)]);
        let t11 = self.raw_texture().read([(posi[0] + 1).min(size[0] - 1), (posi[1] + 1).min(size[1] - 1)]);

        let t0 = t00 * (1.0 - fract[1]) + t01 * fract[1];
        let t1 = t10 * (1.0 - fract[1]) + t11 * fract[1];

        let t = t0 * (1.0 - fract[0]) + t1 * fract[0];

        t
    }

    #[inline(always)]
    unsafe fn sample_unchecked(&self, index: [Self::Index; 2]) -> Self::Sample {
        // TODO: Not this
        self.sample(index)
    }
}
