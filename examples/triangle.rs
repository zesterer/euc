use euc::{Pipeline, rasterizer};
use mini_gl_fb;

struct Triangle;

impl Pipeline for Triangle {
    type Uniform = ();
    type Input = [f32; 3];
    type VsOut = ();
    type Output = [u8; 4];

    fn vert(_: &(), pos: &[f32; 3]) -> ([f32; 3], ()) {
        (*pos, ())
    }

    fn frag(_: &(), _: &()) -> [u8; 4] {
        [255, 0, 0, 255] // Red
    }
}

const W: usize = 640;
const H: usize = 480;

fn main() {
    let mut color = vec![[0; 4]; W * H];
    let mut depth = vec![1.0; W * H];

    Triangle::draw::<rasterizer::Triangles>(
        [W, H],
        &(),
        &[
            [-1.0, -1.0, 0.0],
            [ 1.0, -1.0, 0.0],
            [ 0.0,  1.0, 0.0],
        ],
        &mut color.as_mut(),
        &mut depth.as_mut(),
    );

    let mut win = mini_gl_fb::gotta_go_fast("Triangle", W as f64, H as f64);
    win.update_buffer(&color);
    win.persist();
}
