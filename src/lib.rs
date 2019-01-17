#![no_std]
#![feature(alloc)]

#[macro_use]
extern crate alloc;

pub mod interpolate;
pub mod rasterizer;
pub mod buffer;

// Reexports
pub use self::rasterizer::Rasterizer;
pub use self::interpolate::Interpolate;

pub trait Pipeline where Self: Sized {
    type Uniform;
    type Vertex;
    type VsOut: Clone + Interpolate;
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
