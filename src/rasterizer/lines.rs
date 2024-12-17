use super::*;
use crate::{CoordinateMode, YAxisDirection};

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
            YAxisDirection::Down => [1.0f32, 1.0],
            YAxisDirection::Up => [1.0f32, -1.0],
        };

        let size = tgt_size.map(|e| e as f32);

        let verts_hom_out = core::iter::from_fn(move || Some([vertices.next()?, vertices.next()?]));

        verts_hom_out.for_each(|verts_hom_out: [([f32; 4], V); 2]| {
            blitter.begin_primitive();

            // Calculate vertex shader outputs and vertex homogeneous coordinates
            let verts_hom = [verts_hom_out[0].0, verts_hom_out[1].0];
            let verts_out = verts_hom_out.map(|(_, v)| v);

            let verts_hom = verts_hom.map(|[a0, a1, a2, a3]| [a0 * flip[0], a1 * flip[1], a2, a3]);

            // Convert homogenous to euclidean coordinates
            let verts_euc = verts_hom.map(|[a0, a1, a2, a3]| {
                let w = a3.max(0.0001);
                [a0 / w, a1 / w, a2 / w]
            });

            // Convert vertex coordinates to screen space
            let verts_screen = verts_euc
                .map(|[a0, a1, _a2]| [size[0] * (a0 * 0.5 + 0.5), size[1] * (a1 * -0.5 + 0.5)]);

            // Calculate the triangle bounds as a bounding box
            let screen_min = tgt_min.map(|e| e as f32);
            let screen_max = tgt_max.map(|e| e as f32);

            let [x1, y1] = [verts_screen[0][0] as isize, verts_screen[0][1] as isize];
            let [x2, y2] = [verts_screen[1][0] as isize, verts_screen[1][1] as isize];

            let [wx1, wy1] = [
                (verts_screen[0][0].min(verts_screen[1][0]) + 0.)
                    .clamp(screen_min[0], screen_max[0]) as isize,
                (verts_screen[0][1].min(verts_screen[1][1]) + 0.)
                    .clamp(screen_min[1], screen_max[1]) as isize,
            ];
            let [wx2, wy2] = [
                (verts_screen[0][0].max(verts_screen[1][0]) + 1.)
                    .clamp(screen_min[0], screen_max[0]) as isize,
                (verts_screen[0][1].max(verts_screen[1][1]) + 1.)
                    .clamp(screen_min[1], screen_max[1]) as isize,
            ];

            let use_x = (x1 - x2).abs() > (y1 - y2).abs();
            let norm = 1.0
                / if use_x {
                    verts_screen[1][0] - verts_screen[0][0]
                } else {
                    verts_screen[1][1] - verts_screen[0][1]
                };

            clipline::clipline(
                ((x1, y1), (x2, y2)),
                ((wx1, wy1), (wx2 - 1, wy2 - 1)),
                |x, y| {
                    let (x, y) = (x as usize, y as usize);

                    let frac = if use_x {
                        x as f32 - verts_screen[0][0]
                    } else {
                        y as f32 - verts_screen[0][1]
                    } * norm;

                    // Calculate the interpolated z coordinate for the depth target
                    let z = verts_euc[0][2] + frac * (verts_euc[1][2] - verts_euc[0][2]);

                    if coords.passes_z_clip(z) && blitter.test_fragment(x, y, z) {
                        let get_v_data = |x: f32, y: f32| {
                            let frac = if use_x {
                                x - verts_screen[0][0]
                            } else {
                                y - verts_screen[0][1]
                            } * norm;

                            V::weighted_sum2(
                                verts_out[0].clone(),
                                verts_out[1].clone(),
                                1.0 - frac,
                                frac,
                            )
                        };

                        blitter.emit_fragment(x, y, get_v_data, z);
                    }
                },
            );
        });
    }
}
