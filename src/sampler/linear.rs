use super::*;
use core::{
    marker::PhantomData,
    ops::{Add, Mul},
};

#[cfg(feature = "micromath")]
use micromath::F32Ext;

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
    fn raw_texture(&self) -> &Self::Texture {
        &self.0
    }

    #[inline(always)]
    fn sample(&self, index: [Self::Index; 2]) -> Self::Sample {
        let size = self.raw_texture().size();
        // Index in texture coordinates
        let index_tex = [
            index[0].fract() * size[0] as f32,
            index[1].fract() * size[1] as f32,
        ];
        // Find texel sample coordinates
        let posi = index_tex.map(|e| e.trunc() as usize);
        // Find interpolation values
        let fract = index_tex.map(|e| e.fract());

        debug_assert!(
            posi[0] < size[0],
            "pos: {:?}, sz: {:?}, idx: {:?}",
            posi,
            size,
            index
        );
        debug_assert!(
            posi[1] < size[1],
            "pos: {:?}, sz: {:?}, idx: {:?}",
            posi,
            size,
            index
        );

        let p0x = (posi[0] + 0).min(size[0] - 1);
        let p0y = (posi[1] + 0).min(size[1] - 1);
        let p1x = (posi[0] + 1).min(size[0] - 1);
        let p1y = (posi[1] + 1).min(size[1] - 1);

        let (t00, t10, t01, t11);
        // SAFETY: the `min` above ensures we're in-bounds. Also, this type cannot be created with an underlying
        // texture with a zero size.
        unsafe {
            t00 = self.raw_texture().read_unchecked([p0x, p0y]);
            t10 = self.raw_texture().read_unchecked([p1x, p0y]);
            t01 = self.raw_texture().read_unchecked([p0x, p1y]);
            t11 = self.raw_texture().read_unchecked([p1x, p1y]);
        }

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
