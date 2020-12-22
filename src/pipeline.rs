use crate::{
    texture::Target,
    rasterizer::Rasterizer,
    primitives::PrimitiveKind,
};
use alloc::collections::VecDeque;
use core::{
    cmp::Ordering,
    ops::{Add, Mul, Range},
    borrow::Borrow,
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

impl DepthMode {
    /// Determine whether the depth mode needs to interact with the depth target at all.
    pub fn uses_depth(&self) -> bool {
        self.test.is_some() || self.write
    }
}

/// The handedness of the coordinate space used by a pipeline.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Handedness {
    /// Left-handed coordinate space (used by Vulkan and DirectX)
    Left,
    /// Right-handed coordinate space (used by OpenGL and Metal)
    Right,
}

/// The direction represented by +y in screen space.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum YAxisDirection {
    // +y points down towards the bottom of the screen (i.e: -y is up).
    Down,
    // +y points up towards the top of the screen (i.e: -y is down).
    Up,
}

/// The configuration of the coordinate system used by a pipeline.
pub struct CoordinateMode {
    pub handedness: Handedness,
    pub y_axis_direction: YAxisDirection,
    pub z_clip_range: Option<Range<f32>>,
}

impl CoordinateMode {
    /// OpenGL-like coordinates (right-handed, y = up, -1 to 1 z clip range).
    pub const OPENGL: Self = Self {
        handedness: Handedness::Right,
        y_axis_direction: YAxisDirection::Up,
        z_clip_range: Some(-1.0..1.0),
    };

    /// Vulkan-like coordinates (left-handed, y = down, 0 to 1 z clip range).
    pub const VULKAN: Self = Self {
        handedness: Handedness::Left,
        y_axis_direction: YAxisDirection::Down,
        z_clip_range: Some(0.0..1.0),
    };

    /// Metal-like coordinates (right-handed, y = down, 0 to 1 z clip range).
    pub const METAL: Self = Self {
        handedness: Handedness::Right,
        y_axis_direction: YAxisDirection::Down,
        z_clip_range: Some(0.0..1.0),
    };

    /// DirectX-like coordinates (left-handed, y = up, 0 to 1 z clip range).
    pub const DIRECTX: Self = Self {
        handedness: Handedness::Left,
        y_axis_direction: YAxisDirection::Up,
        z_clip_range: Some(0.0..1.0),
    };

    pub fn without_z_clip(self) -> Self {
        Self {
            z_clip_range: None,
            ..self
        }
    }
}

impl Default for CoordinateMode {
    fn default() -> Self {
        Self::VULKAN
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
    type VertexData: Clone + Mul<f32, Output=Self::VertexData> + Add<Output=Self::VertexData> + Send + Sync;
    type Primitives: PrimitiveKind<Self::VertexData>;
    type Fragment;

    /// Returns whether the pixel buffer should be written to by this pipeline..
    #[inline(always)]
    fn pixel_mode(&self) -> bool { true }

    /// Returns the [`DepthMode`] of this pipeline.
    #[inline(always)]
    fn depth_mode(&self) -> DepthMode { DepthMode::NONE }

    /// Returns the [`CoordinateMode`] of this pipeline.
    #[inline(always)]
    fn coordinate_mode(&self) -> CoordinateMode { CoordinateMode::default() }

    /// Transforms a [`Pipeline::Vertex`] into homogeneous NDCs (Normalised Device Coordinates) for the vertex and a
    /// [`Pipeline::VertexData`] to be interpolated and passed to the fragment shader.
    ///
    /// This stage is executed at the beginning of pipeline execution.
    fn vertex_shader(&self, vertex: &Self::Vertex) -> ([f32; 4], Self::VertexData);

    /// Turn a primitive into many primitives.
    ///
    /// This stage sits between the vertex shader and the fragment shader.
    #[inline(always)]
    fn geometry_shader<O>(&self, primitive: <Self::Primitives as PrimitiveKind<Self::VertexData>>::Primitive, mut output: O)
    where
        O: FnMut(<Self::Primitives as PrimitiveKind<Self::VertexData>>::Primitive),
    {
        output(primitive);
    }

    /// Transforms a [`Pipeline::VertexData`] into a fragment to be rendered to a pixel target.
    ///
    /// This stage is executed for every fragment generated by the rasterizer.
    fn fragment_shader(&self, vs_out: Self::VertexData) -> Self::Fragment;

    /// Blend an old fragment with a new fragment.
    ///
    /// This stage is executed after rasterization and defines how a fragment may be blended into an existing fragment
    /// from the pixel target.
    ///
    /// The default implementation simply returns the new fragment and ignores the old one. However, this may be used
    /// to implement techniques such as alpha blending.
    fn blend_shader(&self, a: Self::Fragment, b: Self::Fragment) -> Self::Fragment { b }

    /// Render a stream of vertices to given provided pixel target and depth target using the rasterizer.
    ///
    /// **Do not implement this method**
    fn render<'a, S, V, P, D>(
        &'a self,
        vertices: S,
        rasterizer_config: <<Self::Primitives as PrimitiveKind<Self::VertexData>>::Rasterizer as Rasterizer>::Config,
        pixels: &mut P,
        depth: &mut D,
    )
    where
        Self: Send + Sync,
        S: IntoIterator<Item = V>,
        V: Borrow<Self::Vertex>,
        P: Target<Texel = Self::Fragment> + Send + Sync + 'a,
        D: Target<Texel = f32> + Send + Sync + 'a,
    {
        let pixel_write = self.pixel_mode();
        let depth_mode = self.depth_mode();
        let principal_x = depth.principal_axis() == 0;
        let target_size = if pixel_write {
            // Ensure that the pixel target and depth target are compatible (but only if we need to actually use the
            // depth target).
            if depth_mode.uses_depth() {
                assert_eq!(pixels.size(), depth.size(), "Pixel target size is compatible with depth target size");
            }
            pixels.size()
        } else {
            depth.size()
        };

        let mut vert_outs = vertices.into_iter().map(|v| self.vertex_shader(v.borrow())).peekable();
        let mut vert_out_queue = VecDeque::new();
        let fetch_vertex = move || {
            loop {
                match vert_out_queue.pop_front() {
                    Some(v) => break Some(v),
                    None if vert_outs.peek().is_none() => break None,
                    None => {
                        let prim = Self::Primitives::collect_primitive(&mut vert_outs)?;
                        self.geometry_shader(
                            prim,
                            |prim| Self::Primitives::primitive_vertices(prim, |v| vert_out_queue.push_back(v)),
                        );
                    },
                }
            }
        };

        let pixels = &*pixels;
        let depth = &*depth;

        let test_depth = move |pos, z: f32| {
            if let Some(test) = depth_mode.test {
                let old_z = unsafe { depth.read_exclusive_unchecked(pos) };
                z.partial_cmp(&old_z) == Some(test)
            } else {
                true
            }
        };

        let emit_fragment = move |pos, vs_out_lerped: Self::VertexData, z: f32| {
            if depth_mode.write {
                unsafe { depth.write_exclusive_unchecked(pos, z); }
            }

            if pixel_write {
                let frag = self.fragment_shader(vs_out_lerped);
                let old_px = unsafe { pixels.read_exclusive_unchecked(pos) };
                let blended_px = self.blend_shader(old_px, frag);
                unsafe { pixels.write_exclusive_unchecked(pos, blended_px); }
            }
        };

        unsafe {
            <Self::Primitives as PrimitiveKind<Self::VertexData>>::Rasterizer::default().rasterize(
                core::iter::from_fn(fetch_vertex),
                target_size,
                principal_x,
                self.coordinate_mode(),
                rasterizer_config,
                test_depth,
                emit_fragment,
            );
        }
    }
}
