use euc::{
    Pipeline,
    rasterizer,
    buffer::Buffer2d,
    Target,
};
use mini_gl_fb;
use vek::*;

struct Cube<'a> {
    phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> Pipeline for Cube<'a> {
    type Uniform = (Mat4<f32>, &'a [Vec4<f32>]);
    type Vertex = (usize, Rgba<f32>);
    type VsOut = Rgba<f32>;
    type Pixel = [u8; 4];

    #[inline(always)]
    fn vert(
        (cam_mat, positions): &Self::Uniform,
        (v_index, v_color): &Self::Vertex,
    ) -> ([f32; 3], Self::VsOut) {
        (
            Vec3::from(*cam_mat * positions[*v_index]).into_array(),
            *v_color,
        )
    }

    #[inline(always)]
    fn frag(_: &Self::Uniform, v_color: &Self::VsOut) -> Self::Pixel {
        v_color.map(|e| (e * 255.0) as u8).into_array()
    }
}

const W: usize = 640;
const H: usize = 480;

fn main() {
    let mut color = Buffer2d::new([W, H], [0; 4]);
    let mut depth = Buffer2d::new([W, H], 1.0);

    let mut win = mini_gl_fb::gotta_go_fast("Spinning Cube", W as f64, H as f64);

    for i in 0.. {
        let cam_mat =
            Mat4::perspective_rh_no(1.3, 1.35, 0.01, 100.0) *
            Mat4::<f32>::scaling_3d(0.4) *
            Mat4::rotation_x((i as f32 * 0.01).sin() * 3.0) *
            Mat4::rotation_y((i as f32 * 0.02).cos() * 2.0);
            Mat4::rotation_z((i as f32 * 0.03).sin() * 1.0);

        color.clear([0; 4]);
        depth.clear(1.0);

        Cube::draw::<rasterizer::Triangles<_>, _>(
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
                (0, Rgba::green()),
                (2, Rgba::red()),
                (3, Rgba::blue()),

                (0, Rgba::green()),
                (3, Rgba::blue()),
                (1, Rgba::red()),

                // +x
                (4, Rgba::green()),
                (6, Rgba::red()),
                (7, Rgba::blue()),

                (4, Rgba::green()),
                (7, Rgba::blue()),
                (5, Rgba::red()),

                // -y
                (0, Rgba::red()),
                (4, Rgba::green()),
                (5, Rgba::blue()),

                (0, Rgba::red()),
                (5, Rgba::blue()),
                (1, Rgba::green()),

                // +y
                (2, Rgba::red()),
                (6, Rgba::green()),
                (7, Rgba::blue()),

                (2, Rgba::red()),
                (7, Rgba::blue()),
                (3, Rgba::green()),

                // -z
                (0, Rgba::red()),
                (4, Rgba::blue()),
                (6, Rgba::green()),

                (0, Rgba::red()),
                (6, Rgba::green()),
                (2, Rgba::blue()),

                // +z
                (1, Rgba::red()),
                (5, Rgba::blue()),
                (7, Rgba::green()),

                (1, Rgba::red()),
                (7, Rgba::green()),
                (3, Rgba::blue()),
            ],
            &mut color,
            &mut depth,
        );

        win.update_buffer(color.as_ref());

        if !win.is_running() {
            break;
        }
    }
}
