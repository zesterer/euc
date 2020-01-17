/// A trait used to enable types to be interpolated throughout the rasterization process
pub trait Interpolate {
    /// Linearly scale two items of this type and sum them
    #[inline(always)]
    fn lerp2(a: Self, b: Self, x: f32, y: f32) -> Self;

    /// Linearly scale three items of this type and sum them
    #[inline(always)]
    fn lerp3(a: Self, b: Self, c: Self, x: f32, y: f32, z: f32) -> Self;
}

// Default impls for certain types
macro_rules! impl_interpolate_for_primitive {
    ($t:ty) => {
        impl Interpolate for $t {
            #[inline(always)]
            fn lerp2(a: Self, b: Self, x: f32, y: f32) -> Self {
                a.mul_add(x, b * y)
            }
            #[inline(always)]
            fn lerp3(a: Self, b: Self, c: Self, x: f32, y: f32, z: f32) -> Self {
                a.mul_add(x, b.mul_add(y, c * z))
            }
        }
    };
}
macro_rules! impl_interpolate_for_complex {
    ($t:ty) => {
        impl Interpolate for $t {
            #[inline(always)]
            fn lerp2(a: Self, b: Self, x: f32, y: f32) -> Self {
                //a * x + b * y
                a.map2(b, |a, b| a.mul_add(x, b * y))
            }
            #[inline(always)]
            fn lerp3(a: Self, b: Self, c: Self, x: f32, y: f32, z: f32) -> Self {
                //a * x + b * y + c * z
                a.map2(b.map2(c, |b, c| b.mul_add(y, c * z)), |a, bc| {
                    a.mul_add(x, bc)
                })
            }
        }
    };
}
impl_interpolate_for_primitive!(f32);
impl_interpolate_for_complex!(vek::Vec2<f32>);
impl_interpolate_for_complex!(vek::Vec3<f32>);
impl_interpolate_for_complex!(vek::Vec4<f32>);
impl_interpolate_for_complex!(vek::Rgb<f32>);
impl_interpolate_for_complex!(vek::Rgba<f32>);

impl<T: Interpolate, U: Interpolate> Interpolate for (T, U) {
    #[inline(always)]
    fn lerp2(a: Self, b: Self, x: f32, y: f32) -> Self {
        (T::lerp2(a.0, b.0, x, y), U::lerp2(a.1, b.1, x, y))
    }

    #[inline(always)]
    fn lerp3(a: Self, b: Self, c: Self, x: f32, y: f32, z: f32) -> Self {
        (
            T::lerp3(a.0, b.0, c.0, x, y, z),
            U::lerp3(a.1, b.1, c.1, x, y, z),
        )
    }
}

impl<T: Interpolate, U: Interpolate, V: Interpolate> Interpolate for (T, U, V) {
    #[inline(always)]
    fn lerp2(a: Self, b: Self, x: f32, y: f32) -> Self {
        (
            T::lerp2(a.0, b.0, x, y),
            U::lerp2(a.1, b.1, x, y),
            V::lerp2(a.2, b.2, x, y),
        )
    }

    #[inline(always)]
    fn lerp3(a: Self, b: Self, c: Self, x: f32, y: f32, z: f32) -> Self {
        (
            T::lerp3(a.0, b.0, c.0, x, y, z),
            U::lerp3(a.1, b.1, c.1, x, y, z),
            V::lerp3(a.2, b.2, c.2, x, y, z),
        )
    }
}

impl<T: Interpolate, U: Interpolate, V: Interpolate, W: Interpolate> Interpolate for (T, U, V, W) {
    #[inline(always)]
    fn lerp2(a: Self, b: Self, x: f32, y: f32) -> Self {
        (
            T::lerp2(a.0, b.0, x, y),
            U::lerp2(a.1, b.1, x, y),
            V::lerp2(a.2, b.2, x, y),
            W::lerp2(a.3, b.3, x, y),
        )
    }

    #[inline(always)]
    fn lerp3(a: Self, b: Self, c: Self, x: f32, y: f32, z: f32) -> Self {
        (
            T::lerp3(a.0, b.0, c.0, x, y, z),
            U::lerp3(a.1, b.1, c.1, x, y, z),
            V::lerp3(a.2, b.2, c.2, x, y, z),
            W::lerp3(a.3, b.3, c.3, x, y, z),
        )
    }
}

impl Interpolate for () {
    #[inline(always)]
    fn lerp2(_: Self, _: Self, _: f32, _: f32) -> Self {
        ()
    }
    #[inline(always)]
    fn lerp3(_: Self, _: Self, _: Self, _: f32, _: f32, _: f32) -> Self {
        ()
    }
}
