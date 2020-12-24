use euc::{Buffer2d, Pipeline, TriangleList, CullMode, Empty};
use vek::*;

struct Triangle;

impl Pipeline for Triangle {
    type Vertex = [f32; 2];
    type VertexData = Vec2<f32>;
    type Primitives = TriangleList;
    type Pixel = u32;

    fn vertex_shader(&self, pos: &[f32; 2]) -> ([f32; 4], Self::VertexData) {
        ([pos[0], pos[1], 0.0, 1.0], Vec2::new(pos[0], pos[1]))
    }

    fn fragment_shader(&self, xy: Self::VertexData) -> Self::Pixel {
        u32::from_le_bytes([(xy.x * 255.0) as u8, (xy.y * 255.0) as u8, 0, 255]) // Red
    }
}
fn main() {
    let [w, h] = [640, 480];
    let mut color = Buffer2d::fill([w, h], 0);
    let mut win = mini_gl_fb::gotta_go_fast("Triangle", w as f64, h as f64);

    Triangle.render(
        &[[-1.0, -1.0], [1.0, -1.0], [0.0, 1.0]],
        CullMode::None,
        &mut color,
        &mut Empty::default(),
    );

    win.update_buffer(color.raw());
    win.persist();
}
