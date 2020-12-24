use core::ops::{Mul, Add};

pub trait WeightedSum: Sized {
    fn weighted_sum(values: &[Self], weights: &[f32]) -> Self;
}

impl<T: Clone + Mul<f32, Output = T> + Add<Output = T>> WeightedSum for T {
    #[inline(always)]
    fn weighted_sum(values: &[Self], weights: &[f32]) -> Self {
        values[1..].iter().zip(weights[1..].iter()).fold(values[0].clone() * weights[0], |a, (x, w)| a + x.clone() * *w)
    }
}

pub trait Denormalize<T>: Sized {
    fn denormalize_to(self, scale: T) -> T;
    fn denormalize_array<const N: usize>(this: [Self; N], other: [T; N]) -> [T; N];
}

macro_rules! impl_denormalize {
    ($this:ty, $other:ty) => {
        impl Denormalize<$other> for $this {
            fn denormalize_to(self, scale: $other) -> $other {
                ((self * scale as $this).max(0.0) as $other).min(scale - 1)
            }

            fn denormalize_array<const N: usize>(this: [Self; N], other: [$other; N]) -> [$other; N] {
                let mut out = [0; N];
                (0..N).for_each(|i| out[i] = this[i].denormalize_to(other[i]));
                out
            }
        }
    };
}

impl_denormalize!(f32, u8);
impl_denormalize!(f32, u16);
impl_denormalize!(f32, u32);
impl_denormalize!(f32, u64);
impl_denormalize!(f32, u128);
impl_denormalize!(f32, usize);

impl_denormalize!(f64, u8);
impl_denormalize!(f64, u16);
impl_denormalize!(f64, u32);
impl_denormalize!(f64, u64);
impl_denormalize!(f64, u128);
impl_denormalize!(f64, usize);
