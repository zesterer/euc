use core::marker::PhantomData;
use vek::*;
use crate::{
    Interpolate,
    Pipeline,
    Target,
};
use self::super::*;

/// A rasterizer that produces filled triangles from groups of 3 consecutive vertices.
///
/// Use the BackfaceCullingEnabled type parameter to enable backface culling.
/// Use the BackfaceCullingDisabled type parameter to disable backface culling.
pub struct Triangles<'a, D, B: BackfaceMode=BackfaceCullingEnabled> {
    phantom: PhantomData<(&'a D, B)>,
}

impl<'a, D: Target<Item=f32>, B: BackfaceMode> Rasterizer for Triangles<'a, D, B> {
    type Input = [f32; 3]; // Vertex coordinates
    type Supplement = &'a mut D; // Depth buffer

    fn draw<P: Pipeline, T: Target<Item=P::Pixel>>(
        pipeline: &P,
        vertices: &[P::Vertex],
        target: &mut T,
        depth: &mut Self::Supplement,
    ) {
        assert_eq!(target.size(), depth.size(), "Target and depth buffers are not similarly sized!");

        let size = Vec2::from(target.size());
        let half_scr = size.map(|e: usize| e as f32 * 0.5);
        const MIRROR: Vec2<f32> = Vec2 { x: 1.0, y: -1.0 };

        let to_ndc = Mat3::from_row_arrays([
            [2.0 / size.x as f32, 0.0, -1.0],
            [0.0, -2.0 / size.y as f32, 1.0],
            [0.0, 0.0, 1.0],
        ]);

        vertices
            .chunks_exact(3)
            .for_each(|verts| {
                // Compute vertex shader outputs
                let (a_hom, a_vs_out) = pipeline.vert(&verts[0]);
                let (b_hom, b_vs_out) = pipeline.vert(&verts[1]);
                let (c_hom, c_vs_out) = pipeline.vert(&verts[2]);

                // Convert homogenous to euclidean coordinates
                let a = Vec3::new(a_hom[0], a_hom[1], a_hom[2]) / a_hom[3];
                let b = Vec3::new(b_hom[0], b_hom[1], b_hom[2]) / b_hom[3];
                let c = Vec3::new(c_hom[0], c_hom[1], c_hom[2]) / c_hom[3];

                // Backface culling
                let ((a, a_hom, a_vs_out), (c, c_hom, c_vs_out)) =
                    // Back face?
                    if (b - a).cross(c - a).z < 0.0 {
                        // If backface culling is enabled, just return: we're done with this tri.
                        if B::ENABLED {
                            return;
                        } else {
                            // Reverse the vertex order
                            ((c, c_hom, c_vs_out), (a, a_hom, a_vs_out))
                        }
                    } else {
                        // Maintain vertex order
                        ((a, a_hom, a_vs_out), (c, c_hom, c_vs_out))
                    };

                let fb_to_weights = {
                    let c = Vec3::new(c_hom[0], c_hom[1], c_hom[3]);
                    let ca = Vec3::new(a_hom[0], a_hom[1], a_hom[3]) - c;
                    let cb = Vec3::new(b_hom[0], b_hom[1], b_hom[3]) - c;
                    let n = ca.cross(cb);
                    let rec_det = if n.magnitude_squared() > 0.0 {
                        1.0 / n.dot(c)
                    } else {
                        1.0
                    };
                    // Compute matrix inverse
                    Mat3::from_row_arrays([
                        cb.cross(c).into_array(),
                        c.cross(ca).into_array(),
                        n.into_array(),
                    ]) * rec_det * to_ndc
                };

                debug_assert!(fb_to_weights.into_row_array().iter().all(|e| e.is_finite()));

                // Convert to framebuffer coordinates
                let a_scr = half_scr * (Vec2::from(a) * MIRROR + 1.0);
                let b_scr = half_scr * (Vec2::from(b) * MIRROR + 1.0);
                let c_scr = half_scr * (Vec2::from(c) * MIRROR + 1.0);

                let a_px = a_scr.map(|e| e as i32);
                let b_px = b_scr.map(|e| e as i32);
                let c_px = c_scr.map(|e| e as i32);

                let min = a_px
                    .map2(b_px, |e, b| e.min(b))
                    .map2(c_px, |e, c| e.min(c))
                    .map(|e| e.max(0))
                    .map2(size, |e, sz| (e).min(sz as i32) as usize);
                let max = a_px
                    .map2(b_px, |e, b| e.max(b))
                    .map2(c_px, |e, c| e.max(c))
                    .map(|e| e.max(0))
                    .map2(size, |e, sz| (e + 1).min(sz as i32) as usize);

                for y in min.y..max.y {
                    for x in min.x..max.x {
                        // Where is the centre of the fragment?
                        let p = Vec3::new(x as f32 + 0.5, y as f32 + 0.5, 1.0);

                        // Calculate vertex weights
                        let weights_hom = fb_to_weights * p;
                        let wa = weights_hom.x / weights_hom.z;
                        let wb = weights_hom.y / weights_hom.z;
                        let wc = 1.0 - wa - wb;

                        // If the point falls outside the triangle, skip this fragment
                        if wa < 0.0 || wa > 1.0 || wb < 0.0 || wb > 1.0 || wc < 0.0 || wc > 1.0 {
                            continue;
                        }

                        // Calculate the interpolated depth of this fragment
                        let z_lerped = f32::lerp3(a_hom[2], b_hom[2], c_hom[2], wa, wb, wc) *
                            weights_hom.z;

                        // Depth test
                        let should_draw = match pipeline.get_depth_strategy() {
                            DepthStrategy::IfLessWrite | DepthStrategy::IfLessNoWrite =>
                                z_lerped < unsafe { depth.get([x, y]) },
                            DepthStrategy::IfMoreWrite | DepthStrategy::IfMoreNoWrite =>
                                z_lerped > unsafe { depth.get([x, y]) },
                            DepthStrategy::None => true,
                        };

                        if should_draw {
                            // Calculate the interpolated vertex attributes of this fragment
                            let vs_out_lerped = P::VsOut::lerp3(
                                a_vs_out.clone(),
                                b_vs_out.clone(),
                                c_vs_out.clone(),
                                wa,
                                wb,
                                wc,
                            );

                            unsafe {
                                // Write depth
                                match pipeline.get_depth_strategy() {
                                    DepthStrategy::IfLessWrite | DepthStrategy::IfMoreWrite =>
                                        depth.set([x, y], z_lerped),
                                    _ => {},
                                }

                                target.set([x, y], pipeline.frag(&vs_out_lerped));
                            }
                        }
                    }
                }
            });
    }
}
