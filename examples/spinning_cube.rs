use euc::{Pipeline, rasterizer};
use mini_gl_fb;
use vek::*;

struct Cube<'a> {
    phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> Pipeline for Cube<'a> {
    type Uniform = (Mat4<f32>, &'a [Vec4<f32>]);
    type Input = (usize, Rgba<f32>);
    type VsOut = Rgba<f32>;
    type Output = [u8; 4];

    #[inline(always)]
    fn vert(
        (cam_mat, positions): &Self::Uniform,
        (v_index, v_color): &(usize, Rgba<f32>),
    ) -> ([f32; 3], Self::VsOut) {
        let screen_pos = Vec3::from(*cam_mat * positions[*v_index]).into_array();

        (screen_pos, *v_color)
    }

    #[inline(always)]
    fn frag(_uniform: &Self::Uniform, v_color: &Self::VsOut) -> [u8; 4] {
        v_color.map(|e| (e * 255.0) as u8).into_array()
    }
}

const W: usize = 640;
const H: usize = 480;

fn main() {
    let mut color;
    let mut depth;

    let mut win = mini_gl_fb::gotta_go_fast("Spinning Cube", W as f64, H as f64);

    for i in 0.. {
        let cam_mat =
            Mat4::<f32>::scaling_3d(0.4) *
            Mat4::perspective_rh_no(1.3, 1.35, 0.001, 1000.0) *
            Mat4::rotation_x(0.3) *
            Mat4::rotation_y(i as f32 * 0.01);

        color = vec![[0; 4]; W * H];
        depth = vec![1.0; W * H];

        Cube::draw::<rasterizer::Triangles>(
            [W, H],
            &(cam_mat, &[
                Vec4::new(-1.0, -1.0, -1.0, 1.0), // 0
                Vec4::new(-1.0, -1.0,  1.0, 1.0), // 1
                Vec4::new(-1.0,  1.0, -1.0, 1.0), // 2
                Vec4::new(-1.0,  1.0,  1.0, 1.0), // 3
                Vec4::new( 1.0, -1.0, -1.0, 1.0), // 4
                Vec4::new( 1.0, -1.0,  1.0, 1.0), // 5
                Vec4::new( 1.0,  1.0, -1.0, 1.0), // 6
                Vec4::new( 1.0,  1.0,  1.0, 1.0), // 7
            ]),
            &[
                // -x
                (0, Rgba::red()),
                (2, Rgba::red()),
                (3, Rgba::red()),

                (0, Rgba::red()),
                (3, Rgba::red()),
                (1, Rgba::red()),

                // +x
                (4, Rgba::red()),
                (6, Rgba::red()),
                (7, Rgba::red()),

                (4, Rgba::red()),
                (7, Rgba::red()),
                (5, Rgba::red()),

                // -y
                (0, Rgba::green()),
                (4, Rgba::green()),
                (5, Rgba::green()),

                (0, Rgba::green()),
                (5, Rgba::green()),
                (1, Rgba::green()),

                // +y
                (2, Rgba::green()),
                (6, Rgba::green()),
                (7, Rgba::green()),

                (2, Rgba::green()),
                (7, Rgba::green()),
                (3, Rgba::green()),

                // -z
                (0, Rgba::blue()),
                (4, Rgba::blue()),
                (6, Rgba::blue()),

                (0, Rgba::blue()),
                (6, Rgba::blue()),
                (2, Rgba::blue()),

                // +z
                (1, Rgba::blue()),
                (5, Rgba::blue()),
                (7, Rgba::blue()),

                (1, Rgba::blue()),
                (7, Rgba::blue()),
                (3, Rgba::blue()),
            ],
            &mut color.as_mut(),
            &mut depth.as_mut(),
        );

        win.update_buffer(&color);

        if !win.is_running() {
            break;
        }
    }
}
