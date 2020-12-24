pub mod triangles;

pub use self::triangles::Triangles;

use crate::{CoordinateMode, Pipeline, math::WeightedSum};
use core::ops::{Mul, Add};

/// The face culling strategy used during rendering.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum CullMode {
    /// Do not cull triangles regardless of their winding order
    None,
    /// Cull clockwise triangles
    Back,
    /// Cull counter-clockwise triangles
    Front,
}

impl Default for CullMode {
    fn default() -> Self {
        CullMode::Back
    }
}

/// A trait for types that define an interface for blitting fragments to surfaces
pub trait Blitter<V>: Sized {
    fn target_size(&self) -> [usize; 2];
    fn target_min(&self) -> [usize; 2];
    fn target_max(&self) -> [usize; 2];

    // Indicate to the blitter that a new primitive is now being rasterized.
    fn begin_primitive(&mut self);

    /// Test whether a fragment should be emitted with the given attributes.
    ///
    /// # Safety
    ///
    /// This function *must* be called with a position that is valid for size and bounds that this type provides.
    unsafe fn test_fragment(&mut self, pos: [usize; 2], z: f32) -> bool;

    /// Emit a fragment with the given attributes.
    ///
    /// # Safety
    ///
    /// This function *must* be called with a position that is valid for size and bounds that this type provides.
    unsafe fn emit_fragment(&mut self, pos: [usize; 2], v_data: V, z: f32);
}

/// A trait that represents types that turn vertex streams into fragment coordinates.
///
/// Rasterizers take an iterator of vertices and emit fragment positions. They do not, by themselves, perform shader
/// execution, depth testing, etc.
pub trait Rasterizer: Default {
    type Config: Clone + Send + Sync;

    /// Rasterize the given vertices into fragments.
    ///
    /// - `target_size`: The size of the render target(s) in pixels
    /// - `principal_x`: Whether the rasterizer should prefer the x axis as the principal iteration access (see
    ///   [`Texture::principle_axes`])
    /// - `emit_fragment`: The function that should be called with the target coordinate (in pixels), weights for each
    ///   vertex as a contribution to the final interpolated vertex output, the vertex outputs, and the depth of each
    ///   rasterized fragment.
    ///
    /// # Safety
    ///
    /// `emit_fragment` must only be called with fragment positions that are valid for the `target_size` parameter
    /// provided. Undefined behaviour can be assumed to occur if this is not upheld.
    unsafe fn rasterize<V, I, B>(
        &self,
        vertices: I,
        principal_x: bool,
        coordinate_mode: CoordinateMode,
        config: Self::Config,
        blitter: B,
    )
    where
        V: Clone + WeightedSum,
        I: Iterator<Item = ([f32; 4], V)>,
        B: Blitter<V>;
}
