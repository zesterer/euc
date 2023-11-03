use super::*;
use crate::{CoordinateMode, YAxisDirection};
use vek::*;

#[cfg(feature = "micromath")]
use micromath::F32Ext;

/// A rasterizer that produces filled triangles.
#[derive(Copy, Clone, Debug, Default)]
pub struct Lines;

impl Rasterizer for Lines {
    type Config = ();

    #[inline]
    unsafe fn rasterize<V, I, B>(
        &self,
        mut vertices: I,
        _principal_x: bool,
        coords: CoordinateMode,
        _config: (),
        mut blitter: B,
    ) where
        V: Clone + WeightedSum,
        I: Iterator<Item = ([f32; 4], V)>,
        B: Blitter<V>,
    {
        let tgt_size = blitter.target_size();
        let tgt_min = blitter.target_min();
        let tgt_max = blitter.target_max();

        let flip = match coords.y_axis_direction {
            YAxisDirection::Down => Vec2::new(1.0, 1.0),
            YAxisDirection::Up => Vec2::new(1.0, -1.0),
        };

        let size = Vec2::<usize>::from(tgt_size).map(|e| e as f32);

        let verts_hom_out =
            core::iter::from_fn(move || Some(Vec2::new(vertices.next()?, vertices.next()?)));

        verts_hom_out.for_each(|verts_hom_out: Vec2<([f32; 4], V)>| {
            blitter.begin_primitive();

            // Calculate vertex shader outputs and vertex homogeneous coordinates
            let verts_hom = Vec2::new(verts_hom_out.x.0, verts_hom_out.y.0).map(Vec4::<f32>::from);
            let verts_out = verts_hom_out.map(|e| e.1);

            let verts_hom = verts_hom.map(|v| v * Vec4::new(flip.x, flip.y, 1.0, 1.0));

            // Convert homogenous to euclidean coordinates
            let verts_euc = verts_hom.map(|v_hom| v_hom.xyz() / v_hom.w.max(0.0001));

            // Convert vertex coordinates to screen space
            let verts_screen = verts_euc.map(|euc| size * (euc.xy() * Vec2::new(0.5, -0.5) + 0.5));

            // Calculate the triangle bounds as a bounding box
            let screen_min = Vec2::<usize>::from(tgt_min).map(|e| e as f32);
            let screen_max = Vec2::<usize>::from(tgt_max).map(|e| e as f32);
            let bounds_clamped = Aabr::<usize> {
                min: (verts_screen.reduce(|a, b| Vec2::partial_min(a, b)) + 0.0)
                    .clamped(screen_min, screen_max)
                    .as_(),
                max: (verts_screen.reduce(|a, b| Vec2::partial_max(a, b)) + 1.0)
                    .clamped(screen_min, screen_max)
                    .as_(),
            };

            let (x1, y1) = verts_screen.x.as_::<isize>().into_tuple();
            let (x2, y2) = verts_screen.y.as_::<isize>().into_tuple();

            let (wx1, wy1) = bounds_clamped.min.as_::<isize>().into_tuple();
            let (wx2, wy2) = bounds_clamped.max.as_::<isize>().into_tuple();

            let use_x = (x1 - x2).abs() > (y1 - y2).abs();
            let norm = 1.0
                / if use_x {
                    verts_screen.y.x - verts_screen.x.x
                } else {
                    verts_screen.y.y - verts_screen.x.y
                };

            clipline::clipline(
                ((x1, y1), (x2, y2)),
                ((wx1, wy1), (wx2 - 1, wy2 - 1)),
                |x, y| {
                    let (x, y) = (x as usize, y as usize);

                    let frac = if use_x {
                        x as f32 - verts_screen.x.x
                    } else {
                        y as f32 - verts_screen.x.y
                    } * norm;

                    // Calculate the interpolated z coordinate for the depth target
                    let z = Lerp::lerp(verts_euc.x.z, verts_euc.y.z, frac);

                    if coords.passes_z_clip(z) {
                        if blitter.test_fragment(x, y, z) {
                            let get_v_data = |x: f32, y: f32| {
                                let frac = if use_x {
                                    x - verts_screen.x.x
                                } else {
                                    y - verts_screen.x.y
                                } * norm;

                                V::weighted_sum2(
                                    verts_out.x.clone(),
                                    verts_out.y.clone(),
                                    1.0 - frac,
                                    frac,
                                )
                            };

                            blitter.emit_fragment(x, y, get_v_data, z);
                        }
                    }
                },
            );
        });
    }
}
