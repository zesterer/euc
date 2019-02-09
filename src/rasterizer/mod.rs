mod triangles;

// Reexports
pub use self::triangles::Triangles;

use crate::{
    Pipeline,
    Target,
};

pub trait BackfaceMode {
    const ENABLED: bool;
}

pub struct BackfaceCullingDisabled;
impl BackfaceMode for BackfaceCullingDisabled {
    const ENABLED: bool = false;
}

pub struct BackfaceCullingEnabled;
impl BackfaceMode for BackfaceCullingEnabled {
    const ENABLED: bool = true;
}

pub trait Rasterizer {
    type Input;
    type Supplement;

    fn draw<P: Pipeline, T: Target<Item=P::Pixel>>(
        uniform: &P::Uniform,
        vertices: &[P::Vertex],
        target: &mut T,
        supplement: &mut Self::Supplement,
    );
}
