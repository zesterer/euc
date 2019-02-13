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

        vertices
            .chunks_exact(3)
            .for_each(|verts| {
                // Compute vertex shader outputs
                let (a, a_vs_out) = pipeline.vert(&verts[0]);
                let (b, b_vs_out) = pipeline.vert(&verts[1]);
                let (c, c_vs_out) = pipeline.vert(&verts[2]);

                let a = Vec3::from(a);
                let b = Vec3::from(b);
                let c = Vec3::from(c);

                // Backface culling
                let ((a, a_vs_out), (c, c_vs_out)) =
                    // Back face?
                    if (b - a).cross(c - a).z < 0.0 {
                        // If backface culling is enabled, just return: we're done with this tri.
                        if B::ENABLED {
                            return;
                        } else {
                            // Reverse the vertex order
                            ((c, c_vs_out), (a, a_vs_out))
                        }
                    } else {
                        // Maintain vertex order
                        ((a, a_vs_out), (c, c_vs_out))
                    };

                // Convert to framebuffer coordinates
                let a_scr = half_scr * (Vec2::from(a) * MIRROR + 1.0);
                let b_scr = half_scr * (Vec2::from(b) * MIRROR + 1.0);
                let c_scr = half_scr * (Vec2::from(c) * MIRROR + 1.0);

                #[inline(always)]
                fn edge(a: Vec2<f32>, b: Vec2<f32>, c: Vec2<f32>) -> f32 {
                    (c.x - a.x) * (b.y - a.y) - (c.y - a.y) * (b.x - a.x)
                    //(b.x-a.x)*(c.y-a.y) - (b.y-a.y)*(c.x-a.x)
                }

                // // Find the x position of an edge given its y
                // #[inline(always)]
                // fn solve_x(a: Vec2<f32>, b: Vec2<f32>, y: f32) -> f32 {
                //     a.x + (b.x - a.x) * (y - a.y) / (b.y - a.y)
                // }

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

                let area = edge(a_scr, b_scr, c_scr);

                for y in min.y..max.y {
                    for x in min.x..max.x {
                        // Where is the centre of the fragment?
                        let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);

                        // Calculate edge values
                        let ea = edge(b_scr, c_scr, p);
                        let eb = edge(c_scr, a_scr, p);
                        let ec = edge(a_scr, b_scr, p);

                        // If the point falls outside the triangle, skip this fragment
                        if ea < 0.0 || eb < 0.0 || ec < 0.0 {
                            continue;
                        }

                        // Calculate vertex weights
                        let wa = ea / area;
                        let wb = eb / area;
                        let wc = ec / area;

                        // Calculate the interpolated depth of this fragment
                        let z_lerped = f32::lerp3(a.z, b.z, c.z, wa, wb, wc);

                        // Depth test
                        if z_lerped < unsafe { *depth.get([x, y]) } {
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
                                depth.set([x, y], z_lerped);
                                target.set([x, y], pipeline.frag(&vs_out_lerped));
                            }
                        }
                    }
                }
            });
    }
}
