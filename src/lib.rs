#![cfg_attr(feature = "nightly", no_std)]
#![cfg_attr(feature = "nightly", feature(alloc))]

#[cfg(feature = "nightly")]
#[macro_use]
extern crate alloc;

pub mod interpolate;
pub mod rasterizer;
pub mod buffer;

// Reexports
pub use self::rasterizer::Rasterizer;
pub use self::interpolate::Interpolate;

/// Represents the high-level structure of a rendering pipeline.
///
/// This governs the following things:
///
/// - Vertex position and data calculation (computed by the vertex shader)
/// - Determining whether each polygon is 'backfacing', and optionally skipping it
/// - Rasterization (performed internally by `euc`)
/// - Comparing the fragment depth against the depth buffer to determine whether it is occluded,
///   and optionally skipping it
/// - Fragment output calculation (computed by the fragment shader)
///
/// In the future, `euc` may extend its capabilities to include compute, geometry, and tesselation
/// shaders.
pub trait Pipeline where Self: Sized {
    /// The type of the uniform data that is made available to all shader programs
    ///
    /// This does not change throughout the rendering process and usually consists of such things
    /// as transformation matrices, textures, or vertex lookup tables when an vertex indexing is
    /// used.
    type Uniform;

    /// The type of the vertex shader input data.
    ///
    /// This usually consists of the vertex's position, normal, colour, texture coordinates, and
    /// other such per-vertex information. When vertex indexing is used, this tends to consist of
    /// the vertex index.
    type Vertex;

    /// The type of the data that gets passed on from the vertex shader to the fragment shader.
    ///
    /// This usually consists of the fragment's normal, colour, texture coordinates and other such
    /// per-fragment information.
    type VsOut: Clone + Interpolate;

    /// The type of emitted pixels.
    ///
    /// This type is emitted by the fragment shader and usually corresponds to the colour of the
    /// pixel.
    type Pixel: Clone;

    /// The vertex shader
    #[inline(always)]
    fn vert(
        uniform: &Self::Uniform,
        vertex: &Self::Vertex,
    ) -> ([f32; 3], Self::VsOut);

    /// The fragment shader
    #[inline(always)]
    fn frag(
        uniform: &Self::Uniform,
        vs_out: &Self::VsOut,
    ) -> Self::Pixel;

    /// Perform a draw call with the given uniform data, vertex array, output target and supplement
    /// type.
    ///
    /// The supplement type is commonly used to represent additional surfaces required by the
    /// rasterizer, such as a depth buffer target.
    fn draw<R: Rasterizer, T: Target<Item=Self::Pixel>>(
        uniform: &Self::Uniform,
        vertices: &[Self::Vertex],
        target: &mut T,
        mut supplement: <R as Rasterizer>::Supplement,
    ) {
        R::draw::<Self, T>(uniform, vertices, target, &mut supplement)
    }
}

/// Represents a 2-dimensional rendering target that can have pixel data read and written to it.
pub trait Target {
    /// The type of items contained within this target.
    type Item: Clone;

    /// Get the dimensions of the target.
    #[inline(always)]
    fn size(&self) -> [usize; 2];

    /// Set the item at the specified location in the target to the given item. The validity of the
    /// location is not checked, and as such this method is marked `unsafe`.
    #[inline(always)]
    unsafe fn set(&mut self, pos: [usize; 2], item: Self::Item);

    /// Get a copy of the item at the specified location in the target. The validity of the
    /// location is not checked, and as such this method is marked `unsafe`.
    #[inline(always)]
    unsafe fn get(&self, pos: [usize; 2]) -> &Self::Item;

    /// Clear the target with copies of the specified item.
    fn clear(&mut self, fill: Self::Item);
}
