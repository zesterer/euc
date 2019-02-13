use core::marker::PhantomData;
use vek::*;
use crate::{
    Interpolate,
    Pipeline,
    Target,
};
use self::super::*;

/// A rasterizer that produces straight lines from groups of 2 consecutive vertices.
pub struct Lines<'a, D> {
    phantom: PhantomData<&'a D>,
}

impl<'a, D: Target<Item=f32>> Rasterizer for Lines<'a, D> {
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
            .chunks_exact(2)
            .for_each(|verts| {
                // Compute vertex shader outputs
                let (a, a_vs_out) = pipeline.vert(&verts[0]);
                let (b, b_vs_out) = pipeline.vert(&verts[1]);

                let a = Vec3::from(a);
                let b = Vec3::from(b);

                // Convert to framebuffer coordinates
                let a_scr = half_scr * (Vec2::from(a) * MIRROR + 1.0);
                let b_scr = half_scr * (Vec2::from(b) * MIRROR + 1.0);

                let a_px = a_scr.map(|e| e as i32);
                let b_px = b_scr.map(|e| e as i32);

                let min = Vec2::<i32>::min(a_px, b_px);
                let max = Vec2::<i32>::max(a_px, b_px);

                if (max.x - min.x) > (max.y - min.y) {
                    let (l_scr, l_z, l_vs_out, r_scr, r_z, r_vs_out) =
                        if a_scr.x < b_scr.x {
                            (a_scr, a.z, a_vs_out, b_scr, b.z, b_vs_out)
                        } else {
                            (b_scr, b.z, b_vs_out, a_scr, a.z, a_vs_out)
                        };

                    let dfrac = 1.0 / (r_scr.x - l_scr.x);
                    let mut frac = 0.0;
                    for x in l_scr.x as i32..r_scr.x as i32 {
                        let y = l_scr.y + frac * (r_scr.y - l_scr.y);

                        // TODO: This is really bad bounds test code, fix this
                        if x < 0 || (x as usize) >= size.x || y < 0.0 || (y as usize) >= size.y {
                            continue;
                        }

                        // Calculate the interpolated depth of this fragment
                        let z_lerped = f32::lerp2(l_z, r_z, 1.0 - frac, frac);

                        let (x, y) = (x as usize, y as usize);

                        // Depth test
                        if z_lerped < unsafe { *depth.get([x, y]) } {
                            // Calculate the interpolated vertex attributes of this fragment
                            let vs_out_lerped = P::VsOut::lerp2(
                                l_vs_out.clone(),
                                r_vs_out.clone(),
                                1.0 - frac,
                                frac,
                            );

                            unsafe {
                                depth.set([x, y], z_lerped);
                                target.set([x, y], pipeline.frag(&vs_out_lerped));
                            }
                        }
                        frac += dfrac;
                    }
                } else {
                    let (l_scr, l_z, l_vs_out, r_scr, r_z, r_vs_out) =
                        if a_scr.y < b_scr.y {
                            (a_scr, a.z, a_vs_out, b_scr, b.z, b_vs_out)
                        } else {
                            (b_scr, b.z, b_vs_out, a_scr, a.z, a_vs_out)
                        };

                    let dfrac = 1.0 / (r_scr.y - l_scr.y);
                    let mut frac = 0.0;
                    for y in l_scr.y as i32..r_scr.y as i32 {
                        let x = l_scr.x + frac * (r_scr.x - l_scr.x);

                        // TODO: This is really bad bounds test code, fix this
                        if x < 0.0 || (x as usize) >= size.x || y < 0 || (y as usize) >= size.y {
                            continue;
                        }

                        // Calculate the interpolated depth of this fragment
                        let z_lerped = f32::lerp2(l_z, r_z, 1.0 - frac, frac);

                        let (x, y) = (x as usize, y as usize);

                        // Depth test
                        if z_lerped < unsafe { *depth.get([x, y]) } {
                            // Calculate the interpolated vertex attributes of this fragment
                            let vs_out_lerped = P::VsOut::lerp2(
                                l_vs_out.clone(),
                                r_vs_out.clone(),
                                1.0 - frac,
                                frac,
                            );

                            unsafe {
                                depth.set([x, y], z_lerped);
                                target.set([x, y], P::frag(pipeline, &vs_out_lerped));
                            }
                        }
                        frac += dfrac;
                    }
                }
            });
    }
}
