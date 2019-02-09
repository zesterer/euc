mod triangles;

// Reexports
pub use self::triangles::Triangles;

use crate::{
    Pipeline,
    Target,
};

/// This trait is for internal use only.
pub trait BackfaceMode {
    const ENABLED: bool;
}

/// Implies that reversed polygons should not be culled from the rendering pipeline.
pub struct BackfaceCullingDisabled;

impl BackfaceMode for BackfaceCullingDisabled {
    const ENABLED: bool = false;
}

/// Implies that reversed polygons should be culled from the rendering pipeline.
pub struct BackfaceCullingEnabled;

impl BackfaceMode for BackfaceCullingEnabled {
    const ENABLED: bool = true;
}

/// Represents a rasterization algorithm.
pub trait Rasterizer {
    /// The type of input required during rasterization.
    ///
    /// For most rasterization algorithms, this is the information that corresponds to a vertex
    /// position.
    type Input;

    /// The type of any supplementary data required by the rasterization algorithm.
    ///
    /// Examples of supplementary data include depth buffers, stencil buffers, etc.
    type Supplement;

    /// Rasterize the provided vertex data and write the resulting fragment information to the
    /// target.
    fn draw<P: Pipeline, T: Target<Item=P::Pixel>>(
        uniform: &P::Uniform,
        vertices: &[P::Vertex],
        target: &mut T,
        supplement: &mut Self::Supplement,
    );
}
