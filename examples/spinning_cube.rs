use vek::*;
use euc::{Pipeline, Buffer2d, Target, TriangleList, CullMode, IndexedVertices};

struct Cube {
    mvp: Mat4<f32>,
}

impl Pipeline for Cube {
    type Vertex = (Vec4<f32>, Rgba<f32>);
    type VertexData = Rgba<f32>;
    type Primitives = TriangleList;
    type Fragment = u32;

    #[inline(always)]
    fn vertex_shader(&self, (pos, color): &Self::Vertex) -> ([f32; 4], Self::VertexData) {
        ((self.mvp * *pos).into_array(), *color)
    }

    #[inline(always)]
    fn fragment_shader(&self, color: Self::VertexData) -> Self::Fragment {
        u32::from_le_bytes((color * 255.0).as_().into_array())
    }
}

const R: Rgba<f32> = Rgba::new(1.0, 0.0, 0.0, 1.0);
const Y: Rgba<f32> = Rgba::new(1.0, 1.0, 0.0, 1.0);
const G: Rgba<f32> = Rgba::new(0.0, 1.0, 0.0, 1.0);
const B: Rgba<f32> = Rgba::new(0.0, 0.0, 1.0, 1.0);

const VERTICES: &[(Vec4<f32>, Rgba<f32>)] = &[
    (Vec4::new(-1.0, -1.0, -1.0, 1.0), R),
    (Vec4::new(-1.0, -1.0,  1.0, 1.0), Y),
    (Vec4::new(-1.0,  1.0, -1.0, 1.0), G),
    (Vec4::new(-1.0,  1.0,  1.0, 1.0), B),
    (Vec4::new( 1.0, -1.0, -1.0, 1.0), B),
    (Vec4::new( 1.0, -1.0,  1.0, 1.0), G),
    (Vec4::new( 1.0,  1.0, -1.0, 1.0), Y),
    (Vec4::new( 1.0,  1.0,  1.0, 1.0), R),
];

const INDICES: &[usize] = &[
    0, 3, 2, 0, 1, 3, // -x
    7, 4, 6, 5, 4, 7, // +x
    5, 0, 4, 1, 0, 5, // -y
    2, 7, 6, 2, 3, 7, // +y
    0, 6, 4, 0, 2, 6, // -z
    7, 1, 5, 3, 1, 7, // +z
];

fn main() {
    let [w, h] = [800, 600];

    let mut color = Buffer2d::fill([w, h], 0);
    let mut depth = Buffer2d::fill([w, h], 1.0);

    let mut win = mini_gl_fb::gotta_go_fast("Cube", w as f64, h as f64);

    let mut i = 0;
    win.glutin_handle_basic_input(|win, input| {
        let mvp = Mat4::perspective_fov_lh_zo(1.3, w as f32, h as f32, 0.01, 100.0)
            * Mat4::translation_3d(Vec3::new(0.0, 0.0, 3.0))
            * Mat4::rotation_x((i as f32 * 0.002).sin() * 8.0)
            * Mat4::rotation_y((i as f32 * 0.004).cos() * 4.0)
            * Mat4::rotation_z((i as f32 * 0.008).sin() * 2.0)
            * Mat4::scaling_3d(Vec3::new(1.0, -1.0, 1.0));

        color.clear(0);
        depth.clear(1.0);

        Cube { mvp }.render(
            IndexedVertices::new(INDICES, VERTICES),
            CullMode::Back,
            &mut color,
            &mut depth,
        );

        win.update_buffer(color.raw());
        win.redraw();

        i += 1;
        true
    });
}
