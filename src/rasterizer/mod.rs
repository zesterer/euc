use std::{
    ops::{Mul, Add},
    marker::PhantomData,
};

use vek::*;

use crate::Pipeline;

pub trait Rasterizer {
    type Input;
    type Supplement;

    fn draw<P: Pipeline>(
        size: [usize; 2],
        uniform: &P::Uniform,
        inputs: &[P::Input],
        target: &mut [P::Output],
        supplement: &mut Self::Supplement,
    );
}

pub struct Triangles<'a> {
    phantom: PhantomData<&'a ()>,
}

impl<'a> Rasterizer for Triangles<'a> {
    type Input = [f32; 3]; // Vertex coordinates
    type Supplement = &'a mut [f32]; // Depth buffer

    fn draw<P: Pipeline>(
        size: [usize; 2],
        uniform: &P::Uniform,
        inputs: &[P::Input],
        target: &mut [P::Output],
        depth: &mut Self::Supplement,
    ) {
        inputs
            .chunks_exact(3)
            .for_each(|verts| {
                // TODO: Use different vertex shader outputs and lerp them
                let (a, a_vs_out) = P::vert(uniform, &verts[0]);
                let (b, b_vs_out) = P::vert(uniform, &verts[1]);
                let (c, c_vs_out) = P::vert(uniform, &verts[2]);

                let a = Vec3::from(a);
                let b = Vec3::from(b);
                let c = Vec3::from(c);

                let half_scr = Vec2::from(size).map(|e: usize| e as f32 * 0.5);

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
                fn lerp_tri<T: Mul<f32, Output=T> + Add<Output=T>>(
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

                    a_val * a_fact +
                    b_val * b_fact +
                    c_val * c_fact
                }

                #[inline(always)]
                fn put<T>(surf: &mut [T], w: usize, x: i32, y: i32, out: T) {
                    surf[y as usize * w + x as usize] = out;
                };

                #[inline(always)]
                fn fetch<T: Clone>(surf: &[T], w: usize, x: i32, y: i32, default: T) -> T {
                    surf
                        .get(y as usize * w + x as usize)
                        .cloned()
                        .unwrap_or(default)
                };

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

                            if z_lerped < fetch(depth, size[0], x, y, 0.0) {
                                put(depth, size[0], x, y, z_lerped);
                                put(target, size[0], x, y, P::frag(uniform, &vs_out_lerped));
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

                            if z_lerped < fetch(depth, size[0], x, y, 0.0) {
                                put(depth, size[0], x, y, z_lerped);
                                put(target, size[0], x, y, P::frag(uniform, &vs_out_lerped));
                            }
                        }
                    }
                };
            });
    }
}
