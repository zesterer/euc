use euc::{
    Pipeline,
    rasterizer,
    buffer::Buffer2d,
    Nothing,
};
use mini_gl_fb;

struct Triangle;

impl Pipeline for Triangle {
    type Uniform = Nothing;
    type Vertex = [f32; 3];
    type VsOut = Nothing;
    type Pixel = [u8; 4];

    fn vert(_: &Self::Uniform, pos: &[f32; 3]) -> ([f32; 3], Self::VsOut) {
        (*pos, Nothing)
    }

    fn frag(_: &Self::Uniform, _: &Self::VsOut) -> Self::Pixel {
        [255, 0, 0, 255] // Red
    }
}

const W: usize = 640;
const H: usize = 480;

fn main() {
    let mut color = Buffer2d::new([W, H], [0; 4]);
    let mut depth = Buffer2d::new([W, H], 1.0);

    Triangle::draw::<rasterizer::Triangles<_>, _>(
        &Nothing,
        &[
            [-1.0, -1.0, 0.0],
            [ 1.0, -1.0, 0.0],
            [ 0.0,  1.0, 0.0],
        ],
        &mut color,
        &mut depth,
    );

    let mut win = mini_gl_fb::gotta_go_fast("Triangle", W as f64, H as f64);
    win.update_buffer(color.as_ref());
    win.persist();
}
