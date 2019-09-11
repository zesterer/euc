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
                let (a_hom, a_vs_out) = pipeline.vert(&verts[0]);
                let (b_hom, b_vs_out) = pipeline.vert(&verts[1]);

                // Convert homogenous to euclidean coordinates
                let a = Vec3::new(a_hom[0], a_hom[1], a_hom[2]) / a_hom[3];
                let b = Vec3::new(b_hom[0], b_hom[1], b_hom[2]) / b_hom[3];

                // Convert to framebuffer coordinates
                let a_scr = half_scr * (Vec2::from(a) * MIRROR + 1.0);
                let b_scr = half_scr * (Vec2::from(b) * MIRROR + 1.0);

                let pa: Vec4<f32> = Vec4::from(a_hom);
                let pb: Vec4<f32> = Vec4::from(b_hom);
                let ab: Vec4<f32> = pb - pa;

                let a_px = a_scr.map(|e| e as i32);
                let b_px = b_scr.map(|e| e as i32);
                
                let min = Vec2::<i32>::min(a_px, b_px);
                let max = Vec2::<i32>::max(a_px, b_px);

                if (max.x - min.x) > (max.y - min.y) {
                    let (l_scr, r_scr) =
                        if a_scr.x < b_scr.x {
                            (a_scr, b_scr)
                        } else {
                            (b_scr, a_scr)
                        };
                    let factor = 1.0 / (r_scr.x - l_scr.x);
                    let m = Mat2::new(pa.w, -pa.x, -ab.w, ab.x) / (ab.x * pa.w - pa.x * ab.w);

                    for x in l_scr.x as i32..r_scr.x as i32 {
                        let y = l_scr.y + (x as f32 - l_scr.x) * factor * (r_scr.y - l_scr.y);

                        // TODO: This is really bad bounds test code, fix this
                        if x < 0 || (x as usize) >= size.x || y < 0.0 || (y as usize) >= size.y {
                            continue;
                        }

                        // Calculate the interpolated depth of this fragment
                        let s_hom = m * Vec2::new(2.0 * x as f32 / size.x as f32 - 1.0, 1.0);
                        let s = s_hom.x / s_hom.y;
                        let z_lerped = (pa.z + s * ab.z) * s_hom.y;

                        let (x, y) = (x as usize, y as usize);

                        // Depth test
                        if z_lerped < unsafe { depth.get([x, y]) } {
                            // Calculate the interpolated vertex attributes of this fragment
                            let vs_out_lerped = P::VsOut::lerp2(
                                a_vs_out.clone(),
                                b_vs_out.clone(),
                                1.0 - s,
                                s,
                            );

                            unsafe {
                                depth.set([x, y], z_lerped);
                                target.set([x, y], pipeline.frag(&vs_out_lerped));
                            }
                        }
                    }
                } else {
                    let (l_scr, r_scr) =
                        if a_scr.y < b_scr.y {
                            (a_scr, b_scr)
                        } else {
                            (b_scr, a_scr)
                        };

                    let factor = 1.0 / (r_scr.y - l_scr.y);
                    let m = Mat2::new(pa.w, -pa.y, -ab.w, ab.y) / (ab.y * pa.w - pa.y * ab.w);

                    for y in l_scr.y as i32..r_scr.y as i32 {
                        let x = l_scr.x + (y as f32 - l_scr.y) * factor * (r_scr.x - l_scr.x);

                        // TODO: This is really bad bounds test code, fix this
                        if x < 0.0 || (x as usize) >= size.x || y < 0 || (y as usize) >= size.y {
                            continue;
                        }

                        // Calculate the interpolated depth of this fragment
                        let s_hom = m * Vec2::new(-2.0 * y as f32 / size.y as f32 + 1.0, 1.0);
                        let s = s_hom.x / s_hom.y;
                        let z_lerped = (pa.z + s * ab.z) * s_hom.y;

                        let (x, y) = (x as usize, y as usize);

                        // Depth test
                        if z_lerped < unsafe { depth.get([x, y]) } {
                            // Calculate the interpolated vertex attributes of this fragment
                            let vs_out_lerped = P::VsOut::lerp2(
                                a_vs_out.clone(),
                                b_vs_out.clone(),
                                1.0 - s,
                                s,
                            );

                            unsafe {
                                depth.set([x, y], z_lerped);
                                target.set([x, y], P::frag(pipeline, &vs_out_lerped));
                            }
                        }
                    }
                }
            });
    }
}
