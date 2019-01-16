use std::marker::PhantomData;

use vek::*;

use crate::{
    Pipeline,
    Target,
    VsOut,
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

        vertices
            .chunks_exact(3)
            .for_each(|verts| {
                // TODO: Use different vertex shader outputs and lerp them
                let (a, a_vs_out) = P::vert(uniform, &verts[0]);
                let (b, b_vs_out) = P::vert(uniform, &verts[1]);
                let (c, c_vs_out) = P::vert(uniform, &verts[2]);

                let a = Vec3::from(a);
                let b = Vec3::from(b);
                let c = Vec3::from(c);

                // Convert to framebuffer coordinates
                let a_scr = half_scr * (Vec2::from(a) + 1.0);
                let b_scr = half_scr * (Vec2::from(b) + 1.0);
                let c_scr = half_scr * (Vec2::from(c) + 1.0);

                // Find the top, middle and bottom vertices
                let (top, mid, bot) = if a_scr.y < b_scr.y {
                    if a_scr.y < c_scr.y {
                        if b_scr.y < c_scr.y { (a_scr, b_scr, c_scr) } else { (a_scr, c_scr, b_scr) }
                    } else {
                        if a_scr.y < b_scr.y { (c_scr, a_scr, b_scr) } else { (c_scr, b_scr, a_scr) }
                    }
                } else {
                    if b_scr.y < c_scr.y {
                        if a_scr.y < c_scr.y { (b_scr, a_scr, c_scr) } else { (b_scr, c_scr, a_scr) }
                    } else {
                        if a_scr.y < b_scr.y { (c_scr, a_scr, b_scr) } else { (c_scr, b_scr, a_scr) }
                    }
                };

                // Find the x position of an edge given its y
                #[inline(always)]
                fn solve_x(a: Vec2<f32>, b: Vec2<f32>, y: f32) -> f32 {
                    a.x + (b.x - a.x) * (y - a.y) / (b.y - a.y)
                }

                #[inline(always)]
                fn lerp_tri<T: VsOut>(
                    a: Vec3<f32>,
                    b: Vec3<f32>,
                    c: Vec3<f32>,
                    p: Vec3<f32>,
                    a_val: T,
                    b_val: T,
                    c_val: T,
                ) -> T {
                    let total = (a - b).cross(a - c).magnitude();
                    let a_fact = (b - p).cross(c - p).magnitude() / total;
                    let b_fact = (c - p).cross(a - p).magnitude() / total;
                    let c_fact = (a - p).cross(b - p).magnitude() / total;

                    <T as VsOut>::lerp3(
                        a_val,
                        b_val,
                        c_val,
                        a_fact,
                        b_fact,
                        c_fact,
                    )
                }

                let height =
                    (top.y as i32)
                    .max(0)
                    ..
                    (bot.y as i32 + 1)
                    .min(size[1] as i32);

                if mid.x < solve_x(top, bot, mid.y) {
                    // Left-pointing
                    for y in height {
                        let breadth =
                            (solve_x(top, mid, y as f32).max(solve_x(mid, bot, y as f32)) as i32)
                            .max(0)
                            ..
                            (solve_x(top, bot, y as f32) as i32 + 1)
                            .min(size[0] as i32);

                        for x in breadth {
                            let vs_out_lerped = lerp_tri(
                                Vec3::from(a_scr),
                                Vec3::from(b_scr),
                                Vec3::from(c_scr),
                                Vec3::new(x as f32, y as f32, 0.0),
                                a_vs_out.clone(),
                                b_vs_out.clone(),
                                c_vs_out.clone(),
                            );

                            let z_lerped = lerp_tri(
                                Vec3::from(a_scr),
                                Vec3::from(b_scr),
                                Vec3::from(c_scr),
                                Vec3::new(x as f32, y as f32, 0.0),
                                a.z,
                                b.z,
                                c.z,
                            );

                            unsafe {
                                let pos = [x as usize, y as usize];
                                if z_lerped < *depth.get(pos) {
                                    depth.set(pos, z_lerped);
                                    target.set(pos, P::frag(uniform, &vs_out_lerped));
                                }
                            }
                        }
                    }
                } else {
                    // Right-pointing
                    for y in height {
                        let breadth =
                            (solve_x(top, bot, y as f32) as i32)
                            .max(0)
                            ..
                            (solve_x(top, mid, y as f32).min(solve_x(mid, bot, y as f32)) as i32 + 1)
                            .min(size[0] as i32);

                        for x in breadth {
                            let vs_out_lerped = lerp_tri(
                                Vec3::from(a_scr),
                                Vec3::from(b_scr),
                                Vec3::from(c_scr),
                                Vec3::new(x as f32, y as f32, 0.0),
                                a_vs_out.clone(),
                                b_vs_out.clone(),
                                c_vs_out.clone(),
                            );

                            let z_lerped = lerp_tri(
                                Vec3::from(a_scr),
                                Vec3::from(b_scr),
                                Vec3::from(c_scr),
                                Vec3::new(x as f32, y as f32, 0.0),
                                a.z,
                                b.z,
                                c.z,
                            );

                            unsafe {
                                let pos = [x as usize, y as usize];
                                if z_lerped < *depth.get(pos) {
                                    depth.set(pos, z_lerped);
                                    target.set(pos, P::frag(uniform, &vs_out_lerped));
                                }
                            }
                        }
                    }
                };
            });
    }
}
