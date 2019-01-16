use core::marker::PhantomData;

use vek::*;

use crate::{
    Pipeline,
    Target,
    Interpolate,
};

pub trait Rasterizer {
    type Input;
    type Supplement;

    fn draw<P: Pipeline, T: Target<Item=P::Pixel>>(
        uniform: &P::Uniform,
        vertices: &[P::Vertex],
        target: &mut T,
        supplement: &mut Self::Supplement,
    );
}

pub struct Triangles<'a, D> {
    phantom: PhantomData<&'a D>,
}

impl<'a, D: Target<Item=f32>> Rasterizer for Triangles<'a, D> {
    type Input = [f32; 3]; // Vertex coordinates
    type Supplement = &'a mut D; // Depth buffer

    fn draw<P: Pipeline, T: Target<Item=P::Pixel>>(
        uniform: &P::Uniform,
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
                let (a, a_vs_out) = P::vert(uniform, &verts[0]);
                let (b, b_vs_out) = P::vert(uniform, &verts[1]);
                let (c, c_vs_out) = P::vert(uniform, &verts[2]);

                let a = Vec3::from(a);
                let b = Vec3::from(b);
                let c = Vec3::from(c);

                // Skip back faces
                if (b - a).cross(c - a).z < 0.0 {
                    return;
                }

                // Convert to framebuffer coordinates
                let a_scr = half_scr * (Vec2::from(a) * MIRROR + 1.0);
                let b_scr = half_scr * (Vec2::from(b) * MIRROR + 1.0);
                let c_scr = half_scr * (Vec2::from(c) * MIRROR + 1.0);

                // // Find the x position of an edge given its y
                // #[inline(always)]
                // fn solve_x(a: Vec2<f32>, b: Vec2<f32>, y: f32) -> f32 {
                //     a.x + (b.x - a.x) * (y - a.y) / (b.y - a.y)
                // }

                #[inline(always)]
                fn get_tri_lerp(
                    a: Vec2<f32>,
                    b: Vec2<f32>,
                    c: Vec2<f32>,
                    p: Vec2<f32>,
                ) -> (f32, f32, f32) {
                    let wa =
                        ((b.y - c.y) * (p.x - c.x) + (c.x - b.x) * (p.y - c.y)) /
                        ((b.y - c.y) * (a.x - c.x) + (c.x - b.x) * (a.y - c.y));

                    let wb =
                        ((c.y - a.y) * (p.x - c.x) + (a.x - c.x) * (p.y - c.y)) /
                        ((b.y - c.y) * (a.x - c.x) + (c.x - b.x) * (a.y - c.y));

                    let wc = 1.0 - wa - wb;

                    (wa, wb, wc)
                }

                let a_px = a_scr.map(|e| e as usize);
                let b_px = b_scr.map(|e| e as usize);
                let c_px = c_scr.map(|e| e as usize);

                let min: Vec2<usize> = Vec2::max(Vec2::min(Vec2::min(a_px, b_px), c_px), Vec2::zero());
                let max: Vec2<usize> = Vec2::min(Vec2::max(Vec2::max(a_px, b_px), c_px) + Vec2::one(), size);

                for y in min.y..max.y {
                    for x in min.x..max.x {
                        let fpos = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                        let (wa, wb, wc) = get_tri_lerp(a_scr, b_scr, c_scr, fpos);

                        // Is the point inside the triangle?
                        if wa.min(wb).min(wc) >= 0.0 {
                            let z_lerped = f32::lerp3(a.z, b.z, c.z, wa, wb, wc);

                            // Depth test
                            if z_lerped < unsafe { *depth.get([x, y]) } {
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
                                    target.set([x, y], P::frag(uniform, &vs_out_lerped));
                                }
                            }
                        }
                    }
                }
            });
    }
}
