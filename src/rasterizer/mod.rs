pub mod triangles;

pub use self::triangles::Triangles;

use crate::{CoordinateMode, math::WeightedSum};
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
    unsafe fn rasterize<V, I, F, G>(
        &self,
        vertices: I,
        target_area: ([usize; 2], [usize; 2]),
        target_size: [usize; 2],
        principal_x: bool,
        coordinate_mode: CoordinateMode,
        config: Self::Config,
        test_depth: F,
        emit_fragment: G,
    )
    where
        V: Clone + WeightedSum,
        I: Iterator<Item = ([f32; 4], V)>,
        F: FnMut([usize; 2], f32) -> bool,
        G: FnMut([usize; 2], V, f32);
}
