use core::ops::{Add, Mul};

pub trait WeightedSum: Sized {
    fn weighted_sum<const N: usize>(values: [Self; N], weights: [f32; N]) -> Self;
    fn weighted_sum2(v0: Self, v1: Self, w0: f32, w1: f32) -> Self {
        Self::weighted_sum([v0, v1], [w0, w1])
    }
    fn weighted_sum3(v0: Self, v1: Self, v2: Self, w0: f32, w1: f32, w2: f32) -> Self {
        Self::weighted_sum([v0, v1, v2], [w0, w1, w2])
    }
}

#[derive(Copy, Clone)]
pub struct Unit;

impl WeightedSum for Unit {
    #[inline(always)]
    fn weighted_sum<const N: usize>(_: [Self; N], _: [f32; N]) -> Self {
        Unit
    }
}

impl<T: Clone + Mul<f32, Output = T> + Add<Output = T>> WeightedSum for T {
    #[inline(always)]
    fn weighted_sum<const N: usize>(values: [Self; N], weights: [f32; N]) -> Self {
        let a = values[0].clone() * weights[0];
        values
            .into_iter()
            .zip(weights)
            .skip(1)
            .fold(a, |a, (b, w)| a + b * w)
    }
    #[inline(always)]
    fn weighted_sum2(v0: Self, v1: Self, w0: f32, w1: f32) -> Self {
        v0 * w0 + v1 * w1
    }
    #[inline(always)]
    fn weighted_sum3(v0: Self, v1: Self, v2: Self, w0: f32, w1: f32, w2: f32) -> Self {
        v0 * w0 + v1 * w1 + v2 * w2
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

            fn denormalize_array<const N: usize>(
                this: [Self; N],
                other: [$other; N],
            ) -> [$other; N] {
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
