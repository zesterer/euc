use crate::{
    math::*,
    texture::Target,
    rasterizer::Rasterizer,
};
use alloc::collections::VecDeque;
use core::{
    cmp::Ordering,
    ops::{Add, Mul},
};

/// Defines how a [`Pipeline`] will interact with the depth target.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct DepthMode {
    /// The test, if any, that occurs when comparing the depth of the new fragment with that of the current depth.
    pub test: Option<Ordering>,
    /// Whether the fragment's depth should be written to the depth target if the test was passed.
    pub write: bool,
}

impl DepthMode {
    pub const NONE: Self = Self {
        test: None,
        write: false,
    };

    pub const LESS_WRITE: Self = Self {
        test: Some(Ordering::Less),
        write: true,
    };

    pub const GREATER_WRITE: Self = Self {
        test: Some(Ordering::Greater),
        write: true,
    };

    pub const LESS_PASS: Self = Self {
        test: Some(Ordering::Less),
        write: false,
    };

    pub const GREATER_PASS: Self = Self {
        test: Some(Ordering::Greater),
        write: false,
    };
}

impl Default for DepthMode {
    fn default() -> Self {
        Self::LESS_WRITE
    }
}

impl DepthMode {
    /// Determine whether the depth mode needs to interact with the depth target at all.
    pub fn uses_depth(&self) -> bool {
        self.test.is_some() || self.write
    }
}

/// The coordinate space used during rasterization.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum CoordinateMode {
    /// right = +x, up = +y, out = -z (used by OpenGL and DirectX), default
    Right,
    /// right = +x, up = -y, out = -z (used by Vulkan)
    Left,
}

impl Default for CoordinateMode {
    fn default() -> Self {
        CoordinateMode::Right
    }
}

/// Represents the high-level structure of a rendering pipeline.
///
/// Conventionally, uniform data is stores as state within the pipeline itself.
///
/// Additional methods such as [`Pipeline::depth_mode`], [Pipeline::`cull_mode`], etc. may be implemented to customize
/// the behaviour of the pipeline even further.
pub trait Pipeline: Sized {
    type Vertex;
    type VsOut: Clone + Mul<f32, Output=Self::VsOut> + Add<Output=Self::VsOut>;
    type Fragment;

    /// Returns the [`DepthMode`] of this pipeline.
    fn depth_mode(&self) -> DepthMode { DepthMode::NONE }

    /// Returns the [`CoordinateMode`] of this pipeline.
    fn coordinate_mode(&self) -> CoordinateMode { CoordinateMode::default() }

    /// Transforms a [`Pipeline::Vertex`] into homogeneous NDCs (Normalised Device Coordinates) for the vertex and a
    /// [`Pipeline::VsOut`] to be interpolated and passed to the fragment shader.
    fn vertex_shader(&self, vertex: &Self::Vertex) -> ([f32; 4], Self::VsOut);

    /// Intercepts the [Pipeline::vertex_shader] and emit additional vertices into the pipeline on the fly.
    ///
    /// This function will be repeatedly called until there are no more vertex inputs to receive. For this reason, it
    /// should never fail to pull at least one vertex from the `input` iterator.
    fn geometry_shader<I, O>(&self, input: I, output: O)
    where
        I: Iterator<Item = ([f32; 4], Self::VsOut)>,
        O: FnMut(([f32; 4], Self::VsOut)),
    {
        input.for_each(output);
    }

    /// Transforms a [`Pipeline::VsOut`] into a fragment to be rendered to a pixel target.
    fn fragment_shader(&self, vs_out: Self::VsOut) -> Self::Fragment;

    /// Blend an old fragment with a new fragment.
    ///
    /// The default implementation simply returns the new fragment and ignores the old one. However, this may be used
    /// to implement techniques such as alpha blending.
    fn blend_shader(&self, a: Self::Fragment, b: Self::Fragment) -> Self::Fragment { b }

    /// Render a stream of vertices to given provided pixel target and depth target using the rasterizer.
    ///
    /// **Do not implement this method**
    fn render<'a, R, V, T, D>(
        &'a self,
        rasterizer: R,
        vertex_stream: V,
        mut pixels: T,
        mut depth: D,
    )
    where
        R: Rasterizer + 'a,
        V: IntoIterator<Item = &'a Self::Vertex>,
        T: Target<Texel = Self::Fragment> + 'a,
        D: Target<Texel = f32> + 'a,
    {
        let depth_mode = self.depth_mode();
        let target_size = pixels.size();
        let principal_x = depth.principal_axis() == 0;

        // Ensure that the pixel target and depth target are compatible (but only if we need to actually use the depth
        // target).
        if depth_mode.uses_depth() {
            assert_eq!(target_size, depth.size(), "Depth target size is compatible with the size of other target(s)");
        }

        let mut vertices = vertex_stream.into_iter().map(|v| self.vertex_shader(v)).peekable();
        let mut vert_queue = VecDeque::new();
        let fetch_vertex = move || {
            loop {
                match vert_queue.pop_front() {
                    Some(v) => break Some(v),
                    None if vertices.peek().is_none() => break None,
                    None => self.geometry_shader(&mut vertices, |v| vert_queue.push_back(v)),
                }
            }
        };

        let emit_fragment = move |pos, w: &[f32], vs_out: &[Self::VsOut], z: f32| {
            // Should we attempt to render the fragment at all?
            let should_render = if let Some(test) = depth_mode.test {
                let old_z = unsafe { depth.read_unchecked(pos) };
                z.partial_cmp(&old_z) == Some(test)
            } else {
                true
            };

            if should_render {
                let vs_out_lerped = w[1..]
                    .iter()
                    .zip(vs_out[1..].iter())
                    .fold(vs_out[0].clone() * w[0], |acc, (w, vs_out)| acc + vs_out.clone() * *w);

                let frag = self.fragment_shader(vs_out_lerped);
                let old_px = unsafe { pixels.read_unchecked(pos) };
                let blended_px = self.blend_shader(old_px, frag);
                unsafe { pixels.write_unchecked(pos, blended_px); }

                if depth_mode.write {
                    unsafe { depth.write_unchecked(pos, z); }
                }
            }
        };

        unsafe {
            rasterizer.rasterize(
                self,
                core::iter::from_fn(fetch_vertex),
                target_size,
                principal_x,
                emit_fragment,
            );
        }
    }
}
