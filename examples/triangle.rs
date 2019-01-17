use euc::{
    Pipeline,
    rasterizer,
    buffer::Buffer2d,
};
use minifb;

struct Triangle;

impl Pipeline for Triangle {
    type Uniform = ();
    type Vertex = [f32; 3];
    type VsOut = ();
    type Pixel = u32;

    fn vert(_: &Self::Uniform, pos: &[f32; 3]) -> ([f32; 3], Self::VsOut) {
        (*pos, ())
    }

    fn frag(_: &Self::Uniform, _: &Self::VsOut) -> Self::Pixel {
        let bytes = [255, 0, 0, 255]; // Red
        (bytes[2] as u32) << 0 |
        (bytes[1] as u32) << 8 |
        (bytes[0] as u32) << 16 |
        (bytes[3] as u32) << 24
    }
}

const W: usize = 640;
const H: usize = 480;

fn main() {
    let mut color = Buffer2d::new([W, H], 0);
    let mut depth = Buffer2d::new([W, H], 1.0);

    Triangle::draw::<rasterizer::Triangles<_>, _>(
        &(),
        &[
            [-1.0, -1.0, 0.0],
            [ 1.0, -1.0, 0.0],
            [ 0.0,  1.0, 0.0],
        ],
        &mut color,
        &mut depth,
    );

    let mut win = minifb::Window::new("Triangle", W, H, minifb::WindowOptions::default()).unwrap();
    while win.is_open() {
        win.update_with_buffer(color.as_ref()).unwrap();
    }
}
