use crate::{
    texture::Target,
    rasterizer::Rasterizer,
    primitives::PrimitiveKind,
    math::WeightedSum,
};
use alloc::{vec::Vec, collections::VecDeque};
use core::{
    cmp::Ordering,
    ops::{Add, Mul, Range},
    borrow::Borrow,
    marker::PhantomData,
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

/// Defines how a [`Pipeline`] will interact with the pixel target.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PixelMode {
    /// Whether the fragment's pixel should be written to the pixel target.
    pub write: bool,
}

impl PixelMode {
    pub const WRITE: Self = Self {
        write: true,
    };

    pub const PASS: Self = Self {
        write: false,
    };
}

impl Default for PixelMode {
    fn default() -> Self {
        Self::WRITE
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
    type VertexData: Clone + WeightedSum + Send + Sync;
    type Primitives: PrimitiveKind<Self::VertexData>;
    type Pixel: Clone;

    /// Returns the [`PixelMode`] of this pipeline.
    #[inline(always)]
    fn pixel_mode(&self) -> PixelMode { PixelMode::default() }

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
    fn fragment_shader(&self, vs_out: Self::VertexData) -> Self::Pixel;

    /// Blend an old fragment with a new fragment.
    ///
    /// This stage is executed after rasterization and defines how a fragment may be blended into an existing fragment
    /// from the pixel target.
    ///
    /// The default implementation simply returns the new fragment and ignores the old one. However, this may be used
    /// to implement techniques such as alpha blending.
    fn blend_shader(&self, a: Self::Pixel, b: Self::Pixel) -> Self::Pixel { b }

    /// Render a stream of vertices to given provided pixel target and depth target using the rasterizer.
    ///
    /// **Do not implement this method**
    fn render<S, V, P, D>(
        &self,
        vertices: S,
        rasterizer_config: <<Self::Primitives as PrimitiveKind<Self::VertexData>>::Rasterizer as Rasterizer>::Config,
        pixel: &mut P,
        depth: &mut D,
    )
    where
        Self: Send + Sync,
        S: IntoIterator<Item = V>,
        V: Borrow<Self::Vertex>,
        P: Target<Texel = Self::Pixel> + Send + Sync,
        D: Target<Texel = f32> + Send + Sync,
    {
        let target_size = match (self.pixel_mode().write, self.depth_mode().uses_depth()) {
            (false, false) => return, // No targets actually get written to, don't bother doing anything
            (true, false) => pixel.size(),
            (false, true) => depth.size(),
            (true, true) => {
                // Ensure that the pixel target and depth target are compatible
                assert_eq!(pixel.size(), depth.size(), "Pixel target size is compatible with depth target size");
                // Prefer
                pixel.size()
            },
        };

        // Produce an iterator over vertices (using the vertex shader and geometry shader to product them)
        let mut vert_outs = vertices.into_iter().map(|v| self.vertex_shader(v.borrow())).peekable();
        let mut vert_out_queue = VecDeque::new();
        let fetch_vertex = core::iter::from_fn(move || {
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
        });

        #[cfg(not(feature = "par"))]
        let r = render_seq(self, fetch_vertex, rasterizer_config, target_size, pixel, depth);
        #[cfg(feature = "par")]
        let r = render_par(self, fetch_vertex, rasterizer_config, target_size, pixel, depth);
        r
    }
}

#[cfg(feature = "par")]
fn render_par<Pipe, S, P, D>(
    pipeline: &Pipe,
    fetch_vertex: S,
    rasterizer_config: <<Pipe::Primitives as PrimitiveKind<Pipe::VertexData>>::Rasterizer as Rasterizer>::Config,
    tgt_size: [usize; 2],
    pixel: &mut P,
    depth: &mut D,
)
where
    Pipe: Pipeline + Send + Sync,
    S: Iterator<Item = ([f32; 4], Pipe::VertexData)>,
    P: Target<Texel = Pipe::Pixel> + Send + Sync,
    D: Target<Texel = f32> + Send + Sync,
{
    use std::thread;
    use core::sync::atomic::{AtomicUsize, Ordering};

    // TODO: Don't pull all vertices at once
    let vertices = fetch_vertex.collect::<Vec<_>>();
    let threads = num_cpus::get();
    assert!(tgt_size[1] >= threads); // TODO: Remove this limitation
    let groups = threads * 8;
    let rows_each = tgt_size[1] / groups;
    let group_index = AtomicUsize::new(0);

    let vertices = &vertices;
    let rasterizer_config = &rasterizer_config;
    let group_index = &group_index;
    let pixel = &*pixel;
    let depth = &*depth;

    crossbeam_utils::thread::scope(|s| {
        for i in 0..threads {
            // TODO: Respawning them each time is dumb
            s.spawn(move |_| {
                loop {
                    let i = group_index.fetch_add(1, Ordering::Relaxed);
                    if i >= groups {
                        break;
                    }

                    let (row_start, rows) = if i == groups - 1 {
                        (i * rows_each, tgt_size[1] - (groups - 1) * rows_each)
                    } else {
                        (i * rows_each, rows_each)
                    };
                    let tgt_min = [0, row_start];
                    let tgt_max = [tgt_size[0], row_start + rows];
                    // Safety: we have exclusive access to our specific regions of `pixel` and `depth`
                    unsafe { render_inner(pipeline, vertices.iter().cloned(), rasterizer_config.clone(), (tgt_min, tgt_max), tgt_size, pixel, depth) }
                }
            });
        }
    }).unwrap();
}

fn render_seq<Pipe, S, P, D>(
    pipeline: &Pipe,
    fetch_vertex: S,
    rasterizer_config: <<Pipe::Primitives as PrimitiveKind<Pipe::VertexData>>::Rasterizer as Rasterizer>::Config,
    tgt_size: [usize; 2],
    pixel: &mut P,
    depth: &mut D,
)
where
    Pipe: Pipeline + Send + Sync,
    S: Iterator<Item = ([f32; 4], Pipe::VertexData)>,
    P: Target<Texel = Pipe::Pixel> + Send + Sync,
    D: Target<Texel = f32> + Send + Sync,
{
    // Safety: we have exclusive access to `pixel` and `depth`
    unsafe { render_inner(pipeline, fetch_vertex, rasterizer_config, ([0; 2], tgt_size), tgt_size, pixel, depth) }
}

unsafe fn render_inner<Pipe, S, P, D>(
    pipeline: &Pipe,
    mut fetch_vertex: S,
    rasterizer_config: <<Pipe::Primitives as PrimitiveKind<Pipe::VertexData>>::Rasterizer as Rasterizer>::Config,
    (tgt_min, tgt_max): ([usize; 2], [usize; 2]),
    tgt_size: [usize; 2],
    pixel: &P,
    depth: &D,
)
where
    Pipe: Pipeline + Send + Sync,
    S: Iterator<Item = ([f32; 4], Pipe::VertexData)>,
    P: Target<Texel = Pipe::Pixel> + Send + Sync,
    D: Target<Texel = f32> + Send + Sync,
{
    let pixel_write = pipeline.pixel_mode().write;
    let depth_mode = pipeline.depth_mode();
    for i in 0..2 {
        // Safety check
        if pixel_write {
            assert!(tgt_min[i] <= pixel.size()[i], "{}, {}, {}", i, tgt_min[i], pixel.size()[i]);
            assert!(tgt_max[i] <= pixel.size()[i], "{}, {}, {}", i, tgt_min[i], pixel.size()[i]);
        }
        if depth_mode.uses_depth() {
            assert!(tgt_min[i] <= depth.size()[i], "{}, {}, {}", i, tgt_min[i], depth.size()[i]);
            assert!(tgt_max[i] <= depth.size()[i], "{}, {}, {}", i, tgt_min[i], depth.size()[i]);
        }
    }

    let principal_x = depth.principal_axis() == 0;

    let pixel = &*pixel;
    let depth = &*depth;

    let test_depth = move |pos, z: f32| {
        if let Some(test) = depth_mode.test {
            let old_z = depth.read_exclusive_unchecked(pos);
            z.partial_cmp(&old_z) == Some(test)
        } else {
            true
        }
    };

    let msaa_level = 0;
    let mut near = core::cell::RefCell::new(fxhash::FxHashMap::default());
    let near = &near;
    let emit_fragment = move |pos, vs_out_lerped: Pipe::VertexData, z: f32| {
        if depth_mode.write {
            depth.write_exclusive_unchecked(pos, z);
        }

        if pixel_write {
            let frag = if msaa_level == 0 {
                pipeline.fragment_shader(vs_out_lerped)
            } else {
                near
                    .borrow_mut()
                    .entry([pos[0] >> msaa_level, pos[1] >> msaa_level])
                    .or_insert_with(|| pipeline.fragment_shader(vs_out_lerped))
                    .clone()
            };
            let old_px = pixel.read_exclusive_unchecked(pos);
            let blended_px = pipeline.blend_shader(old_px, frag);
            pixel.write_exclusive_unchecked(pos, blended_px);
        }
    };

    <Pipe::Primitives as PrimitiveKind<Pipe::VertexData>>::Rasterizer::default().rasterize(
        core::iter::from_fn(move || {
            near.borrow_mut().clear();
            fetch_vertex.next()
        }),
        (tgt_min, tgt_max),
        tgt_size,
        principal_x,
        pipeline.coordinate_mode(),
        rasterizer_config,
        test_depth,
        emit_fragment,
    );
}
