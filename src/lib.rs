pub mod rasterizer;
pub mod buffer;

use std::ops::{Mul, Add};

use rasterizer::Rasterizer;

pub trait Pipeline where Self: Sized {
    type Uniform;
    type Vertex;
    type VsOut: Clone + VsOut;
    type Pixel: Clone;

    #[inline(always)]
    fn vert(
        uniform: &Self::Uniform,
        vertex: &Self::Vertex,
    ) -> ([f32; 3], Self::VsOut);

    #[inline(always)]
    fn frag(
        uniform: &Self::Uniform,
        vs_out: &Self::VsOut,
    ) -> Self::Pixel;

    // R = rasterizer::Triangles
    fn draw<R: Rasterizer, T: Target<Item=Self::Pixel>>(
        uniform: &Self::Uniform,
        vertices: &[Self::Vertex],
        target: &mut T,
        mut supplement: <R as Rasterizer>::Supplement,
    ) {
        R::draw::<Self, T>(uniform, vertices, target, &mut supplement)
    }
}

pub trait VsOut {
    fn lerp2(a: Self, b: Self, x: f32, y: f32) -> Self;
    fn lerp3(a: Self, b: Self, c: Self, x: f32, y: f32, z: f32) -> Self;
}

// Default impl for certain types
impl<T: Mul<f32, Output=T> + Add<Output=T>> VsOut for T {
    fn lerp2(a: Self, b: Self, x: f32, y: f32) -> Self {
        a * x + b * y
    }
    fn lerp3(a: Self, b: Self, c: Self, x: f32, y: f32, z: f32) -> Self {
        a * x + b * y + c * z
    }
}

#[derive(Clone)]
pub struct Nothing;
impl VsOut for Nothing {
    fn lerp2(_: Self, _: Self, _: f32, _: f32) -> Self { Self }
    fn lerp3(_: Self, _: Self, _: Self, _: f32, _: f32, _: f32) -> Self { Self }
}

pub trait Target {
    type Item: Clone;

    #[inline(always)]
    fn size(&self) -> [usize; 2];

    #[inline(always)]
    unsafe fn set(&mut self, pos: [usize; 2], item: Self::Item);

    #[inline(always)]
    unsafe fn get(&self, pos: [usize; 2]) -> &Self::Item;

    fn clear(&mut self, fill: Self::Item);
}
