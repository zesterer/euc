use euc::{
    buffer2::Buffer2d,
    pipeline2::{Pipeline, CullMode, CoordinateMode},
    texture::Empty,
    rasterizer2,
    DepthStrategy,
};
use vek::*;

struct Triangle;

impl Pipeline for Triangle {
    type Vertex = [f32; 4];
    type VsOut = Vec2<f32>;
    type Fragment = u32;

    fn cull_mode(&self) -> CullMode { CullMode::None }

    // Vertex shader
    // - Returns the 3D vertex location, and the VsOut value to be passed to the fragment shader
    #[inline(always)]
    fn vertex_shader(&self, pos: &[f32; 4]) -> ([f32; 4], Self::VsOut) {
        (*pos, Vec2::new(pos[0], pos[1]))
    }

    // Fragment shader
    // - Returns (in this case) a u32
    #[inline(always)]
    fn fragment_shader(&self, xy: Self::VsOut) -> Self::Fragment {
        let bytes = [(xy.x * 255.0) as u8, (xy.y * 255.0) as u8, 0, 255]; // Red

        (bytes[0] as u32) << 0
            | (bytes[1] as u32) << 8
            | (bytes[2] as u32) << 16
            | (bytes[3] as u32) << 24
    }
}

const W: usize = 640;
const H: usize = 480;

fn main() {
    let mut color = Buffer2d::fill([W, H], 0);

    Triangle.render(
        rasterizer2::Triangles,
        &[
            [-1.0, -1.0, 0.0, 1.0],
            [1.0, -1.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 1.0],
        ],
        &mut color,
        Empty::default(),
    );

    let mut win = mini_gl_fb::gotta_go_fast("Triangle", W as f64, H as f64);
    win.update_buffer(color.raw());
    win.persist();
}
