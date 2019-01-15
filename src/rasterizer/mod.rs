use std::marker::PhantomData;

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
            .chunks(3)
            .for_each(|verts| {
                // TODO: Use different vertex shader outputs and lerp them
                let (a, vs_out) = P::vert(uniform, &verts[0]);
                let (b, vs_out) = P::vert(uniform, &verts[1]);
                let (c, vs_out) = P::vert(uniform, &verts[2]);

                let a = Vec3::from(a);
                let b = Vec3::from(b);
                let c = Vec3::from(c);

                let half_scr = Vec2::from(size).map(|e: usize| e as f32 * 0.5);

                // Convert to framebuffer coordinates
                let a_scr = half_scr * (Vec2::from(a) + 1.0);
                let b_scr = half_scr * (Vec2::from(b) + 1.0);
                let c_scr = half_scr * (Vec2::from(c) + 1.0);

                let tris = [a, b, c];

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
                fn solve_x(a: Vec2<f32>, b: Vec2<f32>, y: f32) -> f32 {
                    a.x + (b.x - a.x) * (y - a.y) / (b.y - a.y)
                }

                let mut put_target = |x, y, out| {
                    if
                        x >= 0 && y >= 0 &&
                        x < size[0] as i32 && y < size[1] as i32
                    {
                        target[y as usize * size[0] + x as usize] = out;
                    };
                };

                let height =
                    (top.y as i32)
                    .max(0)
                    ..
                    (bot.y as i32)
                    .min(size[1] as i32 - 1);

                if mid.x < bot.x {
                    // Left-pointing
                    for y in height {
                        let breadth =
                            (solve_x(top, mid, y as f32).max(solve_x(mid, bot, y as f32)) as i32)
                            .max(0)
                            ..
                            (solve_x(top, bot, y as f32) as i32)
                            .min(size[0] as i32 - 1);

                        for x in breadth {
                            put_target(x, y, P::frag(uniform, &vs_out));
                        }
                    }
                } else {
                    // Right-pointing
                    for y in height {
                        let breadth =
                            (solve_x(top, bot, y as f32) as i32)
                            .max(0)
                            ..
                            (solve_x(top, mid, y as f32).min(solve_x(mid, bot, y as f32)) as i32)
                            .min(size[0] as i32 - 1);

                        for x in breadth {
                            put_target(x, y, P::frag(uniform, &vs_out));
                        }
                    }
                };
            });
    }
}
