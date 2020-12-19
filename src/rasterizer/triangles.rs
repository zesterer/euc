use super::*;
use crate::{CullMode, CoordinateMode};
use vek::*;

/// A rasterizer that produces filled triangles.
pub struct Triangles;

impl Rasterizer for Triangles {
    unsafe fn rasterize<P, I, F>(
        &self,
        pipeline: &P,
        mut vertices: I,
        target_size: [usize; 2],
        principal_x: bool,
        mut emit_fragment: F,
    )
    where
        P: Pipeline,
        I: Iterator<Item = ([f32; 4], P::VsOut)>,
        F: FnMut([usize; 2], &[f32], &[P::VsOut], f32),
    {
        let cull_dir = match pipeline.cull_mode() {
            CullMode::None => None,
            CullMode::Back => Some(1.0),
            CullMode::Front => Some(-1.0),
        };

        let flip = match pipeline.coordinate_mode() {
            CoordinateMode::Left => Vec2::new(1.0, 1.0),
            CoordinateMode::Right => Vec2::new(1.0, -1.0),
        };

        let size = Vec2::from(target_size).map(|e: usize| e as f32);

        let to_ndc = Mat3::from_row_arrays([
            [2.0 / size.x, 0.0, -1.0],
            [0.0, -2.0 / size.y, 1.0],
            [0.0, 0.0, 1.0],
        ]);

        loop {
            let verts_hom_out = Vec3::new(
                if let Some(v) = vertices.next() { v } else { break },
                if let Some(v) = vertices.next() { v } else { break },
                if let Some(v) = vertices.next() { v } else { break },
            );

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
                .map(|cull_dir| winding * cull_dir > 0.0)
                .unwrap_or(false)
            {
                continue; // Cull the triangle
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

            // Choose an iteration order based on the principal axis
            let (xs, ys) = (
                (tri_bounds.min.x, tri_bounds.max.x),
                (tri_bounds.min.y, tri_bounds.max.y),
            );
            let coords = (if principal_x { ys.0..ys.1 } else { xs.0..xs.1 })
                .map(|j| (if principal_x { xs.0..xs.1 } else { ys.0..ys.1 })
                    .map(move |i| if principal_x { (i, j) } else { (j, i) }))
                .flatten();

            // Iterate over fragment candidates within the triangle's bounding box
            for (x, y) in coords {
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

                emit_fragment([x, y], w.as_slice(), verts_out.as_slice(), z);
            }
        }
    }
}
