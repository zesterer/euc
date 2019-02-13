use euc::{
    Pipeline,
    rasterizer,
    buffer::Buffer2d,
    Target,
};
use minifb;
use vek::*;

struct Cube<'a> {
    mvp: Mat4<f32>,
    positions: &'a [Vec4<f32>],
}

impl<'a> Pipeline for Cube<'a> {
    type Vertex = (usize, Rgba<f32>);
    type VsOut = Rgba<f32>;
    type Pixel = u32;

    #[inline(always)]
    fn vert(&self, (v_index, v_color): &Self::Vertex) -> ([f32; 3], Self::VsOut) {
        (
            Vec3::from(self.mvp * self.positions[*v_index]).into_array(),
            *v_color,
        )
    }

    #[inline(always)]
    fn frag(&self, v_color: &Self::VsOut) -> Self::Pixel {
        let bytes = v_color.map(|e| (e * 255.0) as u8).into_array();
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

    let mut win = minifb::Window::new("Cube", W, H, minifb::WindowOptions::default()).unwrap();

    for i in 0.. {
        let mvp =
            Mat4::perspective_rh_no(1.3, 1.35, 0.01, 100.0) *
            Mat4::<f32>::scaling_3d(0.4) *
            Mat4::rotation_x((i as f32 * 0.002).sin() * 8.0) *
            Mat4::rotation_y((i as f32 * 0.004).cos() * 4.0);
            Mat4::rotation_z((i as f32 * 0.008).sin() * 2.0);

        color.clear(0);
        depth.clear(1.0);

        Cube {
            mvp,
            positions: &[
                Vec4::new(-1.0, -1.0, -1.0, 1.0), // 0
                Vec4::new(-1.0, -1.0,  1.0, 1.0), // 1
                Vec4::new(-1.0,  1.0, -1.0, 1.0), // 2
                Vec4::new(-1.0,  1.0,  1.0, 1.0), // 3
                Vec4::new( 1.0, -1.0, -1.0, 1.0), // 4
                Vec4::new( 1.0, -1.0,  1.0, 1.0), // 5
                Vec4::new( 1.0,  1.0, -1.0, 1.0), // 6
                Vec4::new( 1.0,  1.0,  1.0, 1.0), // 7
            ],
        }
            .draw::<rasterizer::Triangles<_, rasterizer::BackfaceCullingEnabled>, _>(
                &[
                    // -x
                    (0, Rgba::green()),
                    (2, Rgba::red()),
                    (3, Rgba::blue()),

                    (0, Rgba::green()),
                    (3, Rgba::blue()),
                    (1, Rgba::red()),

                    // +x
                    (7, Rgba::blue()),
                    (6, Rgba::red()),
                    (4, Rgba::green()),

                    (5, Rgba::red()),
                    (7, Rgba::blue()),
                    (4, Rgba::green()),

                    // -y
                    (5, Rgba::blue()),
                    (4, Rgba::green()),
                    (0, Rgba::red()),

                    (1, Rgba::green()),
                    (5, Rgba::blue()),
                    (0, Rgba::red()),

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
                    (7, Rgba::green()),
                    (5, Rgba::blue()),
                    (1, Rgba::red()),

                    (3, Rgba::blue()),
                    (7, Rgba::green()),
                    (1, Rgba::red()),
                ],
                &mut color,
                &mut depth,
            );


        if win.is_open() {
            win.update_with_buffer(color.as_ref()).unwrap();
        } else {
            break;
        }
    }
}
