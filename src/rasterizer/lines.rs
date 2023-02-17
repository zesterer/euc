use super::*;
use crate::{CoordinateMode, YAxisDirection};
use vek::*;

#[cfg(feature = "micromath")]
use micromath_::F32Ext;

/// A rasterizer that produces filled triangles.
#[derive(Copy, Clone, Debug, Default)]
pub struct Lines;

impl Rasterizer for Lines {
    type Config = CullMode;

    #[inline]
    unsafe fn rasterize<V, I, B>(
        &self,
        mut vertices: I,
        _principal_x: bool,
        coordinate_mode: CoordinateMode,
        cull_mode: CullMode,
        mut blitter: B,
    ) where
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

        let size = Vec2::<usize>::from(tgt_size).map(|e| e as f32);

        let to_ndc = Mat3::from_row_arrays([
            [2.0 / size.x, 0.0, -1.0],
            [0.0, -2.0 / size.y, 1.0],
            [0.0, 0.0, 1.0],
        ]);

        let verts_hom_out =
            core::iter::from_fn(move || Some(Vec2::new(vertices.next()?, vertices.next()?)));

        verts_hom_out.for_each(|verts_hom_out: Vec2<([f32; 4], V)>| {
            blitter.begin_primitive();

            // Calculate vertex shader outputs and vertex homogeneous coordinates
            let verts_hom = Vec3::new(verts_hom_out.x.0, verts_hom_out.y.0, verts_hom_out.x.0)
                .map(Vec4::<f32>::from)
                + Vec3::new(
                    Vec4::zero(),
                    Vec4::zero(),
                    Vec4::new(0.001, 0.001, 0.0, 0.0),
                );
            let verts_out = Vec3::new(
                verts_hom_out.x.1.clone(),
                verts_hom_out.y.1,
                verts_hom_out.x.1,
            );

            let verts_hom = verts_hom.map(|v| v * Vec4::new(flip.x, flip.y, 1.0, 1.0));

            // Convert homogenous to euclidean coordinates
            let verts_euc = verts_hom.map(|v_hom| v_hom.xyz() / v_hom.w);

            // Create a matrix that allows conversion between screen coordinates and interpolation weights
            let coords_to_weights = {
                let c = Vec3::new(verts_hom.z.x, verts_hom.z.y, verts_hom.z.w);
                let ca = Vec3::new(verts_hom.x.x, verts_hom.x.y, verts_hom.x.w) - c;
                let cb = Vec3::new(verts_hom.y.x, verts_hom.y.y, verts_hom.y.w) - c;
                let n = ca.cross(cb);
                let rec_det = if n.magnitude_squared() > 0.0 {
                    1.0 / n.dot(c).min(-core::f32::EPSILON)
                } else {
                    1.0
                };

                Mat3::from_row_arrays([cb.cross(c), c.cross(ca), n].map(|v| v.into_array()))
                    * rec_det
                    * to_ndc
            };

            // Ensure we didn't accidentally end up with infinities or NaNs
            assert!(coords_to_weights
                .into_row_array()
                .iter()
                .all(|e| e.is_finite()));

            // Convert vertex coordinates to screen space
            let verts_screen = verts_euc.map(|euc| size * (euc.xy() * Vec2::new(0.5, -0.5) + 0.5));

            // Calculate the triangle bounds as a bounding box
            let screen_min = Vec2::<usize>::from(tgt_min).map(|e| e as f32);
            let screen_max = Vec2::<usize>::from(tgt_max).map(|e| e as f32);
            let tri_bounds_clamped = Aabr::<usize> {
                min: (verts_screen.reduce(|a, b| Vec2::partial_min(a, b)) + 0.0)
                    .clamped(screen_min, screen_max)
                    .as_(),
                max: (verts_screen.reduce(|a, b| Vec2::partial_max(a, b)) + 1.0)
                    .clamped(screen_min, screen_max)
                    .as_(),
            };

            // Calculate change in vertex weights for each pixel
            let weights_at = |p: Vec2<f32>| coords_to_weights * Vec3::new(p.x, p.y, 1.0);
            let w_hom_origin = weights_at(Vec2::zero());
            let w_hom_dx = (weights_at(Vec2::unit_x() * 1000.0) - w_hom_origin) / 1000.0;
            let w_hom_dy = (weights_at(Vec2::unit_y() * 1000.0) - w_hom_origin) / 1000.0;

            let verts_clamped = verts_screen/*verts_screen.xy().map2(verts_screen.xy().yx(), |mut a, b| {
                let dir = b - a;
                if a.y < screen_min.y { a += Vec2::new(dir.x / dir.y, 1.0) * -(a.y - screen_min.y); }
                if a.x < screen_min.x { a += Vec2::new(1.0, dir.y / dir.x) * -(a.x - screen_min.x); }

                if a.y > screen_max.y { a += Vec2::new(dir.x / dir.y, 1.0) * (a.y - screen_max.y); }
                if a.x > screen_max.x { a += Vec2::new(1.0, dir.y / dir.x) * (a.x - screen_max.x); }

                a
            })*/;

            // TODO: This sucks. A lot. It uses 3-vertex homogeneous coordinates with the last vertex being very close
            // to the first, it does loads of unnecessary work for stuff outside the viewport, and it's not even fast.
            for (x, y) in
                // [
                //     verts_screen.x.as_().into_tuple(),
                //     verts_screen.y.as_().into_tuple(),
                // ]
                bresenham::Bresenham::new(
                    verts_clamped.x.as_().into_tuple(),
                    verts_clamped.y.as_().into_tuple(),
                )
            {
                if (tri_bounds_clamped.min.x as isize..tri_bounds_clamped.max.x as isize)
                    .contains(&x)
                    && (tri_bounds_clamped.min.y as isize..tri_bounds_clamped.max.y as isize)
                        .contains(&y)
                {
                    let (x, y) = (x as usize, y as usize);
                    // Find the barycentric weights for the start of this row
                    let w_hom = w_hom_origin + w_hom_dy * y as f32 + w_hom_dx * x as f32;
                    // Calculate vertex weights to determine vs_out lerping and intersection
                    let w_unbalanced = Vec3::new(w_hom.x, w_hom.y, w_hom.z - w_hom.x - w_hom.y);

                    // Calculate the interpolated z coordinate for the depth target
                    let z: f32 = verts_hom.map(|v| v.z).dot(w_unbalanced);

                    if blitter.test_fragment([x, y], z) {
                        // Don't use `.contains(&z)`, it isn't inclusive
                        if coordinate_mode
                            .z_clip_range
                            .clone()
                            .map_or(true, |clip_range| {
                                z >= clip_range.start && z <= clip_range.end
                            })
                        {
                            let get_v_data = |[x, y]: [f32; 2]| {
                                let w_hom = w_hom_origin + w_hom_dy * y + w_hom_dx * x;

                                // Calculate vertex weights to determine vs_out lerping and intersection
                                let w_unbalanced =
                                    Vec3::new(w_hom.x, w_hom.y, w_hom.z - w_hom.x - w_hom.y);
                                let w = w_unbalanced * w_hom.z.recip();
                                let w = Vec2::new(w.x.max(w.z), w.y);

                                V::weighted_sum(verts_out.as_slice(), w.as_slice())
                            };

                            blitter.emit_fragment([x, y], get_v_data, z);
                        }
                    }
                }
            }
        });
    }
}
