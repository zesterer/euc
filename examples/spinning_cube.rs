use vek::*;
use euc::{
    buffer2::Buffer2d,
    pipeline2::Pipeline,
    texture::{Empty, Target},
    rasterizer2,
    DepthStrategy,
};

struct Cube {
    mvp: Mat4<f32>,
}

impl Pipeline for Cube {
    type Vertex = (usize, Vec4<f32>);
    type VsOut = Vec4<f32>;
    type Fragment = u32;

    #[inline(always)]
    fn vertex_shader(&self, (v_index, v_color): &Self::Vertex) -> ([f32; 4], Self::VsOut) {
        ((self.mvp * VERTICES[*v_index]).into_array(), *v_color)
    }

    #[inline(always)]
    fn fragment_shader(&self, v_color: Self::VsOut) -> Self::Fragment {
        let bytes = v_color.map(|e| (e * 255.0) as u8).into_array();
        (bytes[2] as u32) << 0
            | (bytes[1] as u32) << 8
            | (bytes[0] as u32) << 16
            | (bytes[3] as u32) << 24
    }
}

const W: usize = 640;
const H: usize = 480;

const VERTICES: &[Vec4<f32>] = &[
    Vec4::new(-1.0, -1.0, -1.0, 1.0),
    Vec4::new(-1.0, -1.0,  1.0, 1.0),
    Vec4::new(-1.0,  1.0, -1.0, 1.0),
    Vec4::new(-1.0,  1.0,  1.0, 1.0),
    Vec4::new( 1.0, -1.0, -1.0, 1.0),
    Vec4::new( 1.0, -1.0,  1.0, 1.0),
    Vec4::new( 1.0,  1.0, -1.0, 1.0),
    Vec4::new( 1.0,  1.0,  1.0, 1.0),
];

const RED: Vec4<f32> = Vec4::new(1.0, 0.0, 0.0, 1.0);
const GREEN: Vec4<f32> = Vec4::new(0.0, 1.0, 0.0, 1.0);
const BLUE: Vec4<f32> = Vec4::new(0.0, 0.0, 1.0, 1.0);

const INDICES: &[(usize, Vec4<f32>)] = &[
    // -x
    (0, GREEN), (3, BLUE ), (2, RED  ),
    (0, GREEN), (1, RED  ), (3, BLUE ),
    // +x
    (7, BLUE ), (4, GREEN), (6, RED  ),
    (5, RED  ), (4, GREEN), (7, BLUE ),
    // -y
    (5, BLUE ), (0, RED  ), (4, GREEN),
    (1, GREEN), (0, RED  ), (5, BLUE ),
    // +y
    (2, RED  ), (7, BLUE ), (6, GREEN),
    (2, RED  ), (3, GREEN), (7, BLUE ),
    // -z
    (0, RED  ), (6, GREEN), (4, BLUE ),
    (0, RED  ), (2, BLUE ), (6, GREEN),
    // +z
    (7, GREEN), (1, RED  ), (5, BLUE ),
    (3, BLUE ), (1, RED  ), (7, GREEN),
];

fn main() {
    let mut color = Buffer2d::fill([W, H], 0);
    let mut depth = Buffer2d::fill([W, H], 1.0);

    let mut win = mini_gl_fb::gotta_go_fast("Cube", W as f64, H as f64);

    let mut i = 0;
    win.glutin_handle_basic_input(|win, input| {
        let mvp = Mat4::perspective_fov_rh_no(1.3, W as f32, H as f32, 0.01, 100.0)
            * Mat4::translation_3d(Vec3::new(0.0, 0.0, -2.0))
            * Mat4::<f32>::scaling_3d(0.6)
            * Mat4::rotation_x((i as f32 * 0.002).sin() * 8.0)
            * Mat4::rotation_y((i as f32 * 0.004).cos() * 4.0)
            * Mat4::rotation_z((i as f32 * 0.008).sin() * 2.0);

        color.clear(0);
        depth.clear(1.0);

        Cube { mvp }.render(
            rasterizer2::Triangles,
            INDICES,
            &mut color,
            &mut depth,
        );

        win.update_buffer(color.raw());
        win.redraw();

        i += 1;

        true
    });
}
