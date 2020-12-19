use euc::{
    Buffer2d,
    Pipeline,
    TriangleList,
    CullMode,
    Empty,
};
use vek::*;

struct Triangle;

impl Pipeline for Triangle {
    type Vertex = [f32; 4];
    type VsOut = Vec2<f32>;
    type Primitives = TriangleList;
    type Fragment = u32;

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
        &[
            [-1.0, -1.0, 0.0, 1.0],
            [1.0, -1.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 1.0],
        ],
        CullMode::None,
        &mut color,
        Empty::default(),
    );

    let mut win = mini_gl_fb::gotta_go_fast("Triangle", W as f64, H as f64);
    win.update_buffer(color.raw());
    win.persist();
}
