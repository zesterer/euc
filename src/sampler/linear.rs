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
    fn sample(&self, [x, y]: [Self::Index; 2]) -> Self::Sample {
        let [w, h] = self.raw_texture().size();
        // Index in texture coordinates
        let index_tex_x = x.fract() * w as f32;
        let index_tex_y = y.fract() * h as f32;
        // Find texel sample coordinates
        let posi_x = index_tex_x.trunc() as usize;
        let posi_y = index_tex_y.trunc() as usize;
        // Find interpolation values
        let fract_x = index_tex_x.fract();
        let fract_y = index_tex_y.fract();

        debug_assert!(posi_x < w, "pos: {:?}, w: {:?}", posi_x, w,);
        debug_assert!(posi_y < h, "pos: {:?}, h: {:?}", posi_y, h,);

        let p0x = (posi_x + 0).min(w - 1);
        let p0y = (posi_y + 0).min(h - 1);
        let p1x = (posi_x + 1).min(w - 1);
        let p1y = (posi_y + 1).min(h - 1);

        let (t00, t10, t01, t11);
        // SAFETY: the `min` above ensures we're in-bounds. Also, this type cannot be created with an underlying
        // texture with a zero size.
        unsafe {
            t00 = self.raw_texture().read_unchecked([p0x, p0y]);
            t10 = self.raw_texture().read_unchecked([p1x, p0y]);
            t01 = self.raw_texture().read_unchecked([p0x, p1y]);
            t11 = self.raw_texture().read_unchecked([p1x, p1y]);
        }

        let t0 = t00 * (1.0 - fract_y) + t01 * fract_y;
        let t1 = t10 * (1.0 - fract_y) + t11 * fract_y;

        let t = t0 * (1.0 - fract_x) + t1 * fract_x;

        t
    }

    #[inline(always)]
    unsafe fn sample_unchecked(&self, index: [Self::Index; 2]) -> Self::Sample {
        // TODO: Not this
        self.sample(index)
    }
}
