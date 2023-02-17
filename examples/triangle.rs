use euc::{Buffer2d, Empty, Pipeline, TriangleList};
use minifb::{Key, Window, WindowOptions};
use vek::*;

struct Triangle;

impl Pipeline for Triangle {
    type Vertex = ([f32; 2], Rgba<f32>);
    type VertexData = Rgba<f32>;
    type Primitives = TriangleList;
    type Fragment = Rgba<f32>;
    type Pixel = u32;

    fn vertex(&self, (pos, col): &Self::Vertex) -> ([f32; 4], Self::VertexData) {
        ([pos[0], pos[1], 0.0, 1.0], *col)
    }

    fn fragment(&self, col: Self::VertexData) -> Self::Fragment {
        col
    }

    fn blend(&self, _: Self::Pixel, col: Self::Fragment) -> Self::Pixel {
        u32::from_le_bytes(col.map(|e| (e * 255.0) as u8).into_array())
    }
}
fn main() {
    let [w, h] = [640, 480];
    let mut color = Buffer2d::fill([w, h], 0);
    let mut win = Window::new("Triangle", w, h, WindowOptions::default()).unwrap();

    Triangle.render(
        &[
            ([-1.0, -1.0], Rgba::red()),
            ([1.0, -1.0], Rgba::green()),
            ([0.0, 1.0], Rgba::blue()),
        ],
        &mut color,
        &mut Empty::default(),
    );

    while win.is_open() && !win.is_key_down(Key::Escape) {
        win.update_with_buffer(color.raw(), w, h).unwrap();
    }
}
