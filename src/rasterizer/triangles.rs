use super::*;
use crate::{CoordinateMode, YAxisDirection};
use core::ops::{Mul, Add};
use vek::*;

/// A rasterizer that produces filled triangles.
#[derive(Copy, Clone, Debug, Default)]
pub struct Triangles;

impl Rasterizer for Triangles {
    type Config = CullMode;

    unsafe fn rasterize<V, I, F, G>(
        &self,
        mut vertices: I,
        target_size: [usize; 2],
        principal_x: bool,
        coordinate_mode: CoordinateMode,
        cull_mode: CullMode,
        test_depth: F,
        emit_fragment: G,
    )
    where
        V: Clone + Mul<f32, Output = V> + Add<Output = V> + Send + Sync,
        I: Iterator<Item = ([f32; 4], V)>,
        F: Fn([usize; 2], f32) -> bool + Send + Sync,
        G: Fn([usize; 2], V, f32) + Send + Sync,
    {
        let cull_dir = match cull_mode {
            CullMode::None => None,
            CullMode::Back => Some(1.0),
            CullMode::Front => Some(-1.0),
        };

        let flip = match coordinate_mode.y_axis_direction {
            YAxisDirection::Down => Vec2::new(1.0, 1.0),
            YAxisDirection::Up => Vec2::new(1.0, -1.0),
        };

        let size = Vec2::from(target_size).map(|e: usize| e as f32);

        let to_ndc = Mat3::from_row_arrays([
            [2.0 / size.x, 0.0, -1.0],
            [0.0, -2.0 / size.y, 1.0],
            [0.0, 0.0, 1.0],
        ]);

        let verts_hom_outs = core::iter::from_fn(move || {
            Some(Vec3::new(vertices.next()?, vertices.next()?, vertices.next()?))
        });

        verts_hom_outs.for_each(|verts_hom_out: Vec3<([f32; 4], V)>| {
            // Calculate vertex shader outputs and vertex homogeneous coordinates
            let verts_hom = Vec3::new(verts_hom_out.x.0, verts_hom_out.y.0, verts_hom_out.z.0).map(Vec4::<f32>::from);
            let verts_out = Vec3::new(verts_hom_out.x.1, verts_hom_out.y.1, verts_hom_out.z.1);

            let verts_hom = verts_hom.map(|v| v * Vec4::new(flip.x, flip.y, 1.0, 1.0));

            // Convert homogenous to euclidean coordinates
            let verts_euc = verts_hom.map(|v_hom| v_hom.xyz() / v_hom.w);

            // Calculate winding direction to determine culling behaviour
            let winding = (verts_euc.y - verts_euc.x).cross(verts_euc.z - verts_euc.x).z;

            // Culling and correcting for winding
            let (verts_hom, verts_euc, verts_out) = if cull_dir
                .map(|cull_dir| winding * cull_dir < 0.0)
                .unwrap_or(false)
            {
                return; // Cull the triangle
            } else if winding < 0.0 {
                // Reverse vertex order
                (verts_hom.zyx(), verts_euc.zyx(), verts_out.zyx())
            } else {
                (verts_hom, verts_euc, verts_out)
            };

            // Create a matrix that allows conversion between screen coordinates and interpolation weights
            let coords_to_weights = {
                let c = Vec3::new(verts_hom.z.x, verts_hom.z.y, verts_hom.z.w);
                let ca = Vec3::new(verts_hom.x.x, verts_hom.x.y, verts_hom.x.w) - c;
                let cb = Vec3::new(verts_hom.y.x, verts_hom.y.y, verts_hom.y.w) - c;
                let n = ca.cross(cb);
                let rec_det = if n.magnitude_squared() > 0.0 {
                    1.0 / n.dot(c)
                } else {
                    1.0
                };

                Mat3::from_row_arrays([cb.cross(c), c.cross(ca), n].map(|v| v.into_array())) * rec_det * to_ndc
            };

            // Ensure we didn't accidentally end up with infinities or NaNs
            debug_assert!(coords_to_weights.into_row_array().iter().all(|e| e.is_finite()));

            // Convert vertex coordinates to screen space
            let verts_screen = verts_euc.map(|euc| size * (euc.xy() * Vec2::new(0.5, -0.5) + 0.5));

            // Calculate the triangle bounds as a bounding box
            let tri_bounds = Aabr::<usize> {
                min: Vec2::max(verts_screen.reduce(|a, b| Vec2::partial_min(a, b)).as_(), Vec2::zero()),
                max: Vec2::min(verts_screen.reduce(|a, b| Vec2::partial_max(a, b)).as_() + 1, Vec2::from(target_size) - 1),
            };

            // Iterate over fragment candidates within the triangle's bounding box
            (tri_bounds.min.y..tri_bounds.max.y).for_each(|y| {
                // Only perform this optimisation if it looks to be worth it.
                let row_range = if tri_bounds.max.x - tri_bounds.min.x > 8 {
                    // Calculate a precise for which the pixels of the triangle appear on this row
                    let edges = verts_screen.zip(Vec3::new(verts_screen.y, verts_screen.z, verts_screen.x));
                    let row_range = edges
                        .map(|(a, b)| {
                            let x = Lerp::lerp(a.x, b.x, (y as f32 - a.y) / (b.y - a.y));
                            let x2 = Lerp::lerp(a.x, b.x, (y as f32 + 1.0 - a.y) / (b.y - a.y));
                            let (x_min, x_max) = (x.min(x2), x.max(x2));
                            Vec2::new(
                                if x < tri_bounds.min.x as f32 { tri_bounds.max.x as f32 } else { x_min },
                                if x > tri_bounds.max.x as f32 { tri_bounds.min.x as f32 } else { x_max },
                            )
                        })
                        .reduce(|a, b| Vec2::new(a.x.min(b.x), a.y.max(b.y)))
                        .map(|e| e as usize);
                    Vec2::new(
                        row_range.x.saturating_sub(1),
                        (row_range.y + 1).min(target_size[0]),
                    )
                } else {
                    Vec2::new(tri_bounds.min.x, tri_bounds.max.x)
                };

                for x in row_range.x..row_range.y {
                    // Calculate fragment center
                    let p = Vec3::new(x as f32 + 0.5, y as f32 + 0.5, 1.0);

                    // Calculate vertex weights to determine vs_out lerping and intersection
                    let w_hom = coords_to_weights * p;
                    let w = Vec2::new(w_hom.x / w_hom.z, w_hom.y / w_hom.z);
                    let w = Vec3::new(w.x, w.y, 1.0 - w.x - w.y);

                    // Test the weights to determine whether the fragment is outside the triangle
                    if w.map(|e| e < 0.0).reduce_or() {
                        continue;
                    }

                    // Calculate the interpolated z coordinate for the depth target
                    let z: f32 = verts_hom.map2(w, |v, w| v.z * w).sum() * w_hom.z;

                    // Don't use `.contains(&z)`, it isn't inclusive
                    if z >= coordinate_mode.z_clip_range.start && z <= coordinate_mode.z_clip_range.end {
                        if test_depth([x, y], z) {
                            let vert_out_lerped = verts_out.clone().map2(w, |vo, w| vo * w).sum();

                            emit_fragment([x, y], vert_out_lerped, z);
                        }
                    }
                }
            });
        });
    }
}
