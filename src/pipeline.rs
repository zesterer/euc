use crate::{
    buffer::Buffer2d, math::WeightedSum, primitives::PrimitiveKind, rasterizer::Rasterizer,
    texture::Target,
};
use alloc::{collections::VecDeque, vec::Vec};
use core::{borrow::Borrow, cmp::Ordering, ops::Range};

#[cfg(feature = "micromath")]
use micromath::F32Ext;

/// Defines how a [`Pipeline`] will interact with the depth target.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
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
#[non_exhaustive]
pub struct PixelMode {
    /// Whether the fragment's pixel should be written to the pixel target.
    pub write: bool,
}

impl PixelMode {
    pub const WRITE: Self = Self { write: true };

    pub const PASS: Self = Self { write: false };
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
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CoordinateMode {
    pub handedness: Handedness,
    pub y_axis_direction: YAxisDirection,
    pub z_clip_range: Option<Range<f32>>,
}

/// The anti-aliasing mode used by a pipeline.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AaMode {
    /// No anti-aliasing.
    None,
    /// Multi-sampling anti-aliasing.
    ///
    /// This form of anti-aliasing skips evaluating fragments in the middle of primitives while maintaining detail
    /// along edges. The `level` should be within the range 1 to 6 (inclusive).
    Msaa { level: u32 },
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

    pub(crate) fn passes_z_clip(&self, z: f32) -> bool {
        // Don't use `.contains(&z)`, it isn't inclusive
        self.z_clip_range
            .as_ref()
            .map_or(true, |clip| clip.start <= z && z <= clip.end)
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
pub trait Pipeline<'r>: Sized {
    type Vertex;
    type VertexData: Clone + WeightedSum + Send + Sync;
    type Primitives: PrimitiveKind<Self::VertexData>;
    type Fragment: Clone + WeightedSum;
    type Pixel: Clone;

    /// Returns the [`PixelMode`] of this pipeline.
    #[inline]
    fn pixel_mode(&self) -> PixelMode {
        PixelMode::default()
    }

    /// Returns the [`DepthMode`] of this pipeline.
    #[inline]
    fn depth_mode(&self) -> DepthMode {
        DepthMode::NONE
    }

    /// Returns the [`CoordinateMode`] of this pipeline.
    #[inline]
    fn coordinate_mode(&self) -> CoordinateMode {
        CoordinateMode::default()
    }

    /// Returns the [`AaMode`] of this pipeline.
    #[inline]
    fn aa_mode(&self) -> AaMode {
        AaMode::None
    }

    /// Returns the rasterizer configuration (usually [`CullMode`], when using [`Triangles`]) of this pipeline.
    #[inline]
    fn rasterizer_config(
        &self,
    ) -> <<Self::Primitives as PrimitiveKind<Self::VertexData>>::Rasterizer as Rasterizer>::Config
    {
        Default::default()
    }

    /// Transforms a [`Pipeline::Vertex`] into homogeneous NDCs (Normalised Device Coordinates) for the vertex and a
    /// [`Pipeline::VertexData`] to be interpolated and passed to the fragment shader.
    ///
    /// This stage is executed at the beginning of pipeline execution.
    fn vertex(&self, vertex: &Self::Vertex) -> ([f32; 4], Self::VertexData);

    /// Turn a primitive into many primitives.
    ///
    /// This stage sits between the vertex shader and the fragment shader.
    #[inline]
    fn geometry<O>(
        &self,
        primitive: <Self::Primitives as PrimitiveKind<Self::VertexData>>::Primitive,
        mut output: O,
    ) where
        O: FnMut(<Self::Primitives as PrimitiveKind<Self::VertexData>>::Primitive),
    {
        output(primitive);
    }

    /// Transforms a [`Pipeline::VertexData`] into a fragment to be rendered to a pixel target.
    ///
    /// This stage is executed for every fragment generated by the rasterizer.
    fn fragment(&self, vs_out: Self::VertexData) -> Self::Fragment;

    /// Blend an old fragment with a new fragment.
    ///
    /// This stage is executed after rasterization and defines how a fragment may be blended into an existing fragment
    /// from the pixel target.
    ///
    /// The default implementation simply returns the new fragment and ignores the old one. However, this may be used
    /// to implement techniques such as alpha blending.
    fn blend(&self, old: Self::Pixel, new: Self::Fragment) -> Self::Pixel;

    /// Render a stream of vertices to given provided pixel target and depth target using the rasterizer.
    ///
    /// **Do not implement this method**
    fn render<S, V, P, D>(&self, vertices: S, pixel: &mut P, depth: &mut D)
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
                assert_eq!(
                    pixel.size(),
                    depth.size(),
                    "Pixel target size is compatible with depth target size"
                );
                // Prefer
                pixel.size()
            }
        };

        // Produce an iterator over vertices (using the vertex shader and geometry shader to produce them)
        let mut vert_outs = vertices
            .into_iter()
            .map(|v| self.vertex(v.borrow()))
            .peekable();
        let mut vert_out_queue = VecDeque::new();
        let fetch_vertex = core::iter::from_fn(move || loop {
            match vert_out_queue.pop_front() {
                Some(v) => break Some(v),
                None if vert_outs.peek().is_none() => break None,
                None => {
                    let prim = Self::Primitives::collect_primitive(&mut vert_outs)?;
                    self.geometry(prim, |prim| {
                        Self::Primitives::primitive_vertices(prim, |v| vert_out_queue.push_back(v))
                    });
                }
            }
        });

        let msaa_level = match self.aa_mode() {
            AaMode::None => 0,
            AaMode::Msaa { level } => level.max(0).min(6) as usize,
        };

        #[cfg(not(feature = "par"))]
        let r = render_seq(self, fetch_vertex, target_size, pixel, depth, msaa_level);
        #[cfg(feature = "par")]
        let r = render_par(self, fetch_vertex, target_size, pixel, depth, msaa_level);
        r
    }
}

#[cfg(feature = "par")]
fn render_par<'r, Pipe, S, P, D>(
    pipeline: &Pipe,
    fetch_vertex: S,
    tgt_size: [usize; 2],
    pixel: &mut P,
    depth: &mut D,
    msaa_level: usize,
) where
    Pipe: Pipeline<'r> + Send + Sync,
    S: Iterator<Item = ([f32; 4], Pipe::VertexData)>,
    P: Target<Texel = Pipe::Pixel> + Send + Sync,
    D: Target<Texel = f32> + Send + Sync,
{
    use core::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;

    // TODO: Don't pull all vertices at once
    let vertices = fetch_vertex.collect::<Vec<_>>();
    let threads = num_cpus::get();
    let row = AtomicUsize::new(0);

    const FRAGMENTS_PER_GROUP: usize = 20_000; // Magic number, maybe make this configurable?
    let group_rows = FRAGMENTS_PER_GROUP * (1 << msaa_level) / tgt_size[0].max(1);
    let needed_threads = (tgt_size[1] / group_rows).min(threads);

    let vertices = &vertices;
    let pixel = &*pixel;
    let depth = &*depth;

    thread::scope(|s| {
        for _ in 0..needed_threads {
            // TODO: Respawning them each time is dumb
            s.spawn(|| {
                loop {
                    let row_start = row.fetch_add(group_rows, Ordering::Relaxed);
                    let row_end = if row_start >= tgt_size[1] {
                        break;
                    } else {
                        (row_start + group_rows).min(tgt_size[1])
                    };

                    let tgt_min = [0, row_start];
                    let tgt_max = [tgt_size[0], row_end];
                    // Safety: we have exclusive access to our specific regions of `pixel` and `depth`
                    unsafe {
                        render_inner(
                            pipeline,
                            vertices.iter().cloned(),
                            (tgt_min, tgt_max),
                            tgt_size,
                            pixel,
                            depth,
                            msaa_level,
                        )
                    }
                }
            });
        }
    });
}

#[cfg(not(feature = "par"))]
fn render_seq<'r, Pipe, S, P, D>(
    pipeline: &Pipe,
    fetch_vertex: S,
    tgt_size: [usize; 2],
    pixel: &mut P,
    depth: &mut D,
    msaa_level: usize,
) where
    Pipe: Pipeline<'r> + Send + Sync,
    S: Iterator<Item = ([f32; 4], Pipe::VertexData)>,
    P: Target<Texel = Pipe::Pixel> + Send + Sync,
    D: Target<Texel = f32> + Send + Sync,
{
    // Safety: we have exclusive access to `pixel` and `depth`
    unsafe {
        render_inner(
            pipeline,
            fetch_vertex,
            ([0; 2], tgt_size),
            tgt_size,
            pixel,
            depth,
            msaa_level,
        )
    }
}

unsafe fn render_inner<'r, Pipe, S, P, D>(
    pipeline: &Pipe,
    fetch_vertex: S,
    (tgt_min, tgt_max): ([usize; 2], [usize; 2]),
    tgt_size: [usize; 2],
    pixel: &P,
    depth: &D,
    msaa_level: usize,
) where
    Pipe: Pipeline<'r> + Send + Sync,
    S: Iterator<Item = ([f32; 4], Pipe::VertexData)>,
    P: Target<Texel = Pipe::Pixel> + Send + Sync,
    D: Target<Texel = f32> + Send + Sync,
{
    let write_pixels = pipeline.pixel_mode().write;
    let depth_mode = pipeline.depth_mode();
    for i in 0..2 {
        // Safety check
        if write_pixels {
            assert!(
                tgt_min[i] <= pixel.size()[i],
                "{}, {}, {}",
                i,
                tgt_min[i],
                pixel.size()[i]
            );
            assert!(
                tgt_max[i] <= pixel.size()[i],
                "{}, {}, {}",
                i,
                tgt_min[i],
                pixel.size()[i]
            );
        }
        if depth_mode.uses_depth() {
            assert!(
                tgt_min[i] <= depth.size()[i],
                "{}, {}, {}",
                i,
                tgt_min[i],
                depth.size()[i]
            );
            assert!(
                tgt_max[i] <= depth.size()[i],
                "{}, {}, {}",
                i,
                tgt_min[i],
                depth.size()[i]
            );
        }
    }

    let principal_x = depth.preferred_axes().map_or(true, |[a, _]| a == 0);

    use crate::rasterizer::Blitter;

    struct BlitterImpl<'a, 'r, Pipe: Pipeline<'r>, P, D> {
        write_pixels: bool,
        depth_mode: DepthMode,

        tgt_min: [usize; 2],
        tgt_max: [usize; 2],
        tgt_size: [usize; 2],

        pipeline: &'a Pipe,
        pixel: &'a P,
        depth: &'a D,
        primitive_count: u64,

        msaa_level: usize,
        msaa_buf: Option<Buffer2d<(u64, Option<Pipe::Fragment>)>>,
        msaa_div: f32,
    }

    impl<'a, 'r, Pipe, P, D> BlitterImpl<'a, 'r, Pipe, P, D>
    where
        Pipe: Pipeline<'r> + Send + Sync,
        P: Target<Texel = Pipe::Pixel> + Send + Sync,
        D: Target<Texel = f32> + Send + Sync,
    {
        #[inline]
        unsafe fn msaa_fragment<F: FnMut(usize, usize) -> Pipe::VertexData>(
            &mut self,
            x: usize,
            y: usize,
            mut get_v_data: F,
        ) -> Pipe::Fragment {
            // Safety: MSAA buffer will always be large enough
            let texel = self.msaa_buf.as_mut().unwrap().get_mut([x + 1, y + 1]);
            if texel.0 != self.primitive_count {
                texel.0 = self.primitive_count;
                texel.1 = Some(self.pipeline.fragment(get_v_data(x, y)));
            }
            // Safety: We know this entry will always be occupied due to the code above
            texel
                .1
                .clone()
                .unwrap_or_else(|| core::hint::unreachable_unchecked())
        }
    }

    impl<'a, 'r, Pipe, P, D> Blitter<Pipe::VertexData> for BlitterImpl<'a, 'r, Pipe, P, D>
    where
        Pipe: Pipeline<'r> + Send + Sync,
        P: Target<Texel = Pipe::Pixel> + Send + Sync,
        D: Target<Texel = f32> + Send + Sync,
    {
        fn target_size(&self) -> [usize; 2] {
            self.tgt_size
        }
        fn target_min(&self) -> [usize; 2] {
            self.tgt_min
        }
        fn target_max(&self) -> [usize; 2] {
            self.tgt_max
        }

        #[inline]
        fn begin_primitive(&mut self) {
            self.primitive_count = self.primitive_count.wrapping_add(1);
        }

        #[inline]
        unsafe fn test_fragment(&mut self, x: usize, y: usize, z: f32) -> bool {
            if let Some(test) = self.depth_mode.test {
                let old_z = self.depth.read_exclusive_unchecked(x, y);
                z.partial_cmp(&old_z) == Some(test)
            } else {
                true
            }
        }

        #[inline]
        unsafe fn emit_fragment<F: FnMut(f32, f32) -> Pipe::VertexData>(
            &mut self,
            x: usize,
            y: usize,
            mut get_v_data: F,
            z: f32,
        ) {
            if self.depth_mode.write {
                self.depth.write_exclusive_unchecked(x, y, z);
            }

            if self.write_pixels {
                let frag = if self.msaa_level == 0 {
                    self.pipeline.fragment(get_v_data(x as f32, y as f32))
                } else {
                    let (fractx, fracty) = (
                        ((x - self.tgt_min[0]) as f32 * self.msaa_div).fract(),
                        ((y - self.tgt_min[1]) as f32 * self.msaa_div).fract(),
                    );

                    let posix = (x - self.tgt_min[0]) >> self.msaa_level;
                    let posiy = (y - self.tgt_min[1]) >> self.msaa_level;

                    let tgt_min = self.tgt_min;
                    let msaa_level = self.msaa_level;
                    let mut get_v_data = |x: usize, y: usize| {
                        get_v_data(
                            (tgt_min[0] + (x << msaa_level)) as f32,
                            (tgt_min[1] + (y << msaa_level)) as f32,
                        )
                    };

                    let t00 = self.msaa_fragment(posix + 0, posiy + 0, &mut get_v_data);
                    let t10 = self.msaa_fragment(posix + 1, posiy + 0, &mut get_v_data);
                    let t01 = self.msaa_fragment(posix + 0, posiy + 1, &mut get_v_data);
                    let t11 = self.msaa_fragment(posix + 1, posiy + 1, &mut get_v_data);

                    let t0 = Pipe::Fragment::weighted_sum2(t00, t01, 1.0 - fracty, fracty);
                    let t1 = Pipe::Fragment::weighted_sum2(t10, t11, 1.0 - fracty, fracty);

                    let t = Pipe::Fragment::weighted_sum2(t0, t1, 1.0 - fractx, fractx);
                    t

                    //self.fetch_pixel([posi[0] + 0, posi[1] + 0], v_data.clone())
                };
                let old_px = self.pixel.read_exclusive_unchecked(x, y);
                let blended_px = self.pipeline.blend(old_px, frag);
                self.pixel.write_exclusive_unchecked(x, y, blended_px);
            }
        }
    }

    <Pipe::Primitives as PrimitiveKind<Pipe::VertexData>>::Rasterizer::default().rasterize(
        fetch_vertex,
        principal_x,
        pipeline.coordinate_mode(),
        pipeline.rasterizer_config(),
        BlitterImpl {
            write_pixels,
            depth_mode,

            tgt_size,
            tgt_min,
            tgt_max,

            pipeline,
            pixel,
            depth,
            primitive_count: 0,

            msaa_level,
            msaa_buf: if msaa_level > 0 {
                Some(Buffer2d::fill_with(
                    [
                        ((tgt_max[0] - tgt_min[0]) >> msaa_level) + 3,
                        ((tgt_max[1] - tgt_min[1]) >> msaa_level) + 3,
                    ],
                    || (u64::MAX, None),
                ))
            } else {
                None
            },
            msaa_div: 1.0 / (1 << msaa_level) as f32,
        },
    );
}
