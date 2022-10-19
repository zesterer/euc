use super::*;
use crate::{CoordinateMode, YAxisDirection};
// use vek::*;
use ultraviolet::*;

#[cfg(feature = "micromath")]
use micromath_::F32Ext;

/// A rasterizer that produces filled triangles.
#[derive(Copy, Clone, Debug, Default)]
pub struct Triangles;

impl Rasterizer for Triangles {
    type Config = CullMode;

    unsafe fn rasterize<V, I, B>(
        &self,
        mut vertices: I,
        principal_x: bool,
        coordinate_mode: CoordinateMode,
        cull_mode: CullMode,
        mut blitter: B,
    )
    where
        V: Clone + WeightedSum,
        I: Iterator<Item = ([f32; 4], V)>,
        B: Blitter<V>,
    {
        let tgt_size = blitter.target_size();
        let tgt_min = blitter.target_min();
        let tgt_max = blitter.target_max();

        let cull_dir = match cull_mode {
            CullMode::None => None,
            CullMode::Back => Some(1.0),
            CullMode::Front => Some(-1.0),
        };

        let flip = match coordinate_mode.y_axis_direction {
            YAxisDirection::Down => Vec2::new(1.0, 1.0),
            YAxisDirection::Up => Vec2::new(1.0, -1.0),
        };

        let size = Vec2::from(tgt_size.map(|e| e as f32));

        fn from_row_arrays(arr: [[f32; 3]; 3]) -> Mat3 {
            let mut new = [
                [arr[0][0], arr[1][0], arr[2][0]],
                [arr[0][1], arr[1][1], arr[2][1]],
                [arr[0][2], arr[1][2], arr[2][2]],
            ];
            Mat3::from(new)
        }

        let to_ndc = from_row_arrays([
            [2.0 / size.x, 0.0, -1.0],
            [0.0, -2.0 / size.y, 1.0],
            [0.0, 0.0, 1.0],
        ]);

        let verts_hom_out = core::iter::from_fn(move || {
            Some([vertices.next()?, vertices.next()?, vertices.next()?])
        });

        verts_hom_out.for_each(|[vho_a, vho_b, vho_c]: [([f32; 4], V); 3]| {
            blitter.begin_primitive();

            // Calculate vertex shader outputs and vertex homogeneous coordinates
            let verts_hom = [vho_a.0, vho_b.0, vho_c.0].map(Vec4::from);
            let verts_out = [vho_a.1, vho_b.1, vho_c.1];

            let verts_hom = verts_hom.map(|v| v * Vec4::new(flip.x, flip.y, 1.0, 1.0));

            // Convert homogenous to euclidean coordinates
            let verts_euc = verts_hom.map(|v_hom| v_hom.xyz() / v_hom.w);

            // Calculate winding direction to determine culling behaviour
            let winding = (verts_euc[1] - verts_euc[0]).cross(verts_euc[2] - verts_euc[0]).z;

            // Culling and correcting for winding
            let (verts_hom, verts_euc, verts_out) = if cull_dir
                .map(|cull_dir| winding * cull_dir < 0.0)
                .unwrap_or(false)
            {
                return; // Cull the triangle
            } else if winding >= 0.0 {
                // Reverse vertex order
                let [vo_a, vo_b, vo_c] = verts_out;
                ([verts_hom[2], verts_hom[1], verts_hom[0]], [verts_euc[2], verts_euc[1], verts_euc[0]], [vo_c, vo_b, vo_a])
            } else {
                (verts_hom, verts_euc, verts_out)
            };

            // Create a matrix that allows conversion between screen coordinates and interpolation weights
            let coords_to_weights = {
                let c = Vec3::new(verts_hom[2].x, verts_hom[2].y, verts_hom[2].w);
                let ca = Vec3::new(verts_hom[0].x, verts_hom[0].y, verts_hom[0].w) - c;
                let cb = Vec3::new(verts_hom[1].x, verts_hom[1].y, verts_hom[1].w) - c;
                let n = ca.cross(cb);
                let rec_det = if n.mag_sq() > 0.0 {
                    1.0 / n.dot(c).min(-core::f32::EPSILON)
                } else {
                    1.0
                };

                from_row_arrays([cb.cross(c), c.cross(ca), n].map(|v| *v.as_array())) * rec_det * to_ndc
            };

            // Ensure we didn't accidentally end up with infinities or NaNs
            assert!(coords_to_weights.as_array().iter().all(|e| e.is_finite()));

            // Convert vertex coordinates to screen space
            let verts_screen = verts_euc.map(|euc| size * (euc.xy() * Vec2::new(0.5, -0.5) + Vec2::broadcast(0.5)));

            // Calculate the triangle bounds as a bounding box
            let screen_min = tgt_min.map(|e| e as f32);
            let screen_max = tgt_max.map(|e| e as f32);
            let tri_bounds_clamped = (
                (verts_screen.into_iter().copied().reduce(|a, b| a.min_by_component(b)).unwrap() + Vec2::broadcast(0.0)).clamped(Vec2::from(screen_min), Vec2::from(screen_max)),
                (verts_screen.into_iter().copied().reduce(|a, b| a.max_by_component(b)).unwrap() + Vec2::broadcast(1.0)).clamped(Vec2::from(screen_min), Vec2::from(screen_max)),
            );

            // Calculate change in vertex weights for each pixel
            let weights_at = |p: Vec2| coords_to_weights * Vec3::new(p.x, p.y, 1.0);
            let w_hom_origin = weights_at(Vec2::zero());
            let w_hom_dx = (weights_at(Vec2::unit_x() * 1000.0) - w_hom_origin) / 1000.0;
            let w_hom_dy = (weights_at(Vec2::unit_y() * 1000.0) - w_hom_origin) / 1000.0;

            // Iterate over fragment candidates within the triangle's bounding box
            (tri_bounds_clamped.0.y as usize..tri_bounds_clamped.1.y as usize).for_each(|y| {
                // More precisely find the required draw bounds for this row with a little maths
                // First, order vertices by height
                let verts_by_y = if verts_screen[0].y < verts_screen[1].y.min(verts_screen[2].y) {
                    if verts_screen[1].y < verts_screen[2].y {
                        [verts_screen[0], verts_screen[1], verts_screen[2]]
                    } else {
                        [verts_screen[0], verts_screen[2], verts_screen[1]]
                    }
                } else if verts_screen[1].y < verts_screen[0].y.min(verts_screen[2].y) {
                    if verts_screen[0].y < verts_screen[2].y {
                        [verts_screen[1], verts_screen[0], verts_screen[2]]
                    } else {
                        [verts_screen[1], verts_screen[2], verts_screen[0]]
                    }
                } else {
                    if verts_screen[0].y < verts_screen[1].y {
                        [verts_screen[2], verts_screen[0], verts_screen[1]]
                    } else {
                        [verts_screen[2], verts_screen[1], verts_screen[0]]
                    }
                };

                // Then, depending on the half of the triangle we're in, we need to check different lines
                let edge_lines = if (y as f32) < verts_by_y[1].y {
                    [(verts_by_y[0], verts_by_y[1]), (verts_by_y[0], verts_by_y[2])]
                } else {
                    [(verts_by_y[1], verts_by_y[2]), (verts_by_y[0], verts_by_y[2])]
                };

                // Finally, for each of the lines, calculate the point at which our row intersects it
                let row_bounds = edge_lines
                    .map(|(a, b)| {
                        // Could be more efficient
                        let x = Lerp::lerp(&a.x, b.x, (y as f32 - a.y) / (b.y - a.y));
                        let x2 = Lerp::lerp(&a.x, b.x, (y as f32 + 1.0 - a.y) / (b.y - a.y));
                        let (x_min, x_max) = (x.min(x2), x.max(x2));
                        Vec2::new(x_min, x_max)
                    })
                    .into_iter()
                    .copied()
                    .reduce(|a, b| Vec2::new(a.x.min(b.x), a.y.max(b.y)))
                    .unwrap()
                    .map(|e| e.max(0.0));

                // Now we have screen-space bounds for the row. Clean it up and clamp it to the screen bounds
                let row_range = (
                    (row_bounds.x as usize).saturating_sub(1).max(tri_bounds_clamped.0.x as usize),
                    (row_bounds.y.ceil() as usize).min(tri_bounds_clamped.1.x as usize),
                );

                // Stupid version
                //let row_range = Vec2::new(tri_bounds_clamped.min.x, tri_bounds_clamped.max.x);

                // Find the barycentric weights for the start of this row
                let mut w_hom = w_hom_origin + w_hom_dy * y as f32 + w_hom_dx * row_range.0 as f32;

                for x in row_range.0..row_range.1 {
                    // Calculate vertex weights to determine vs_out lerping and intersection
                    let w_unbalanced = Vec3::new(w_hom.x, w_hom.y, w_hom.z - w_hom.x - w_hom.y);
                    let w = w_unbalanced * w_hom.z.recip();

                    // Test the weights to determine whether the fragment is inside the triangle
                    if w.x >= 0.0 && w.y >= 0.0 && w.z >= 0.0 {
                        // Calculate the interpolated z coordinate for the depth target
                        let z: f32 = Vec3::from(verts_hom.map(|v| v.z)).dot(w_unbalanced);

                        if blitter.test_fragment([x, y], z) {
                            // Don't use `.contains(&z)`, it isn't inclusive
                            if coordinate_mode.z_clip_range.clone().map_or(true, |clip_range| z >= clip_range.start && z <= clip_range.end) {
                                let get_v_data = |[x, y]: [f32; 2]| {
                                    let w_hom = w_hom_origin + w_hom_dy * y + w_hom_dx * x;

                                    // Calculate vertex weights to determine vs_out lerping and intersection
                                    let w_unbalanced = Vec3::new(w_hom.x, w_hom.y, w_hom.z - w_hom.x - w_hom.y);
                                    let w = w_unbalanced * w_hom.z.recip();

                                    V::weighted_sum(verts_out.as_slice(), w.as_slice())
                                };

                                blitter.emit_fragment([x, y], get_v_data, z);
                            }
                        }
                    }

                    // Update barycentric weight ready for the next fragment
                    w_hom += w_hom_dx;
                }
            });
        });
    }
}
