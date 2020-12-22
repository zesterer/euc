use euc::{Buffer2d, Pipeline, Target, TriangleList, CullMode, Sampler, Nearest};
use image_::RgbaImage;
use vek::{Mat4, Vec2, Vec3, Vec4};

struct Cube<'a> {
    mvp: Mat4<f32>,
    positions: &'a [Vec4<f32>],
    uvs: &'a [Vec2<f32>],
    sampler: &'a Nearest<RgbaImage, f32>,
}

impl<'a> Pipeline for Cube<'a> {
    type Vertex = usize;
    type VertexData = Vec2<f32>;
    type Primitives = TriangleList;
    type Fragment = u32;

    #[inline]
    fn vertex_shader(&self, v_index: &Self::Vertex) -> ([f32; 4], Self::VertexData) {
        (
            (self.mvp * self.positions[*v_index]).into_array(),
            self.uvs[*v_index],
        )
    }

    #[inline]
    fn fragment_shader(&self, v_uv: Self::VertexData) -> Self::Fragment {
        u32::from_le_bytes(self.sampler.sample(v_uv.into_array()).0)
    }
}

fn main() {
    let [w, h] = [800, 600];

    let mut color = Buffer2d::fill([w, h], 0);
    let mut depth = Buffer2d::fill([w, h], 1.0);

    let positions = [
        // z = 1
        Vec4::new(-1.0, -1.0, 1.0, 1.0),
        Vec4::new(-1.0, 1.0, 1.0, 1.0),
        Vec4::new(1.0, 1.0, 1.0, 1.0),
        Vec4::new(1.0, -1.0, 1.0, 1.0),
        // z == -1
        Vec4::new(-1.0, -1.0, -1.0, 1.0),
        Vec4::new(-1.0, 1.0, -1.0, 1.0),
        Vec4::new(1.0, 1.0, -1.0, 1.0),
        Vec4::new(1.0, -1.0, -1.0, 1.0),
        // y = 1
        Vec4::new(-1.0, 1.0, 1.0, 1.0),
        Vec4::new(-1.0, 1.0, -1.0, 1.0),
        Vec4::new(1.0, 1.0, -1.0, 1.0),
        Vec4::new(1.0, 1.0, 1.0, 1.0),
        // y = -1
        Vec4::new(-1.0, -1.0, 1.0, 1.0),
        Vec4::new(-1.0, -1.0, -1.0, 1.0),
        Vec4::new(1.0, -1.0, -1.0, 1.0),
        Vec4::new(1.0, -1.0, 1.0, 1.0),
        // x = 1
        Vec4::new(1.0, -1.0, 1.0, 1.0),
        Vec4::new(1.0, -1.0, -1.0, 1.0),
        Vec4::new(1.0, 1.0, -1.0, 1.0),
        Vec4::new(1.0, 1.0, 1.0, 1.0),
        // x = -1
        Vec4::new(-1.0, -1.0, 1.0, 1.0),
        Vec4::new(-1.0, -1.0, -1.0, 1.0),
        Vec4::new(-1.0, 1.0, -1.0, 1.0),
        Vec4::new(-1.0, 1.0, 1.0, 1.0),
    ];
    let uvs = [
        // z = 1
        Vec2::new(0.0, 1.0),
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(1.0, 1.0),
        // z = -1
        Vec2::new(0.0, 0.0),
        Vec2::new(0.0, 1.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(1.0, 0.0),
        // y = 1
        Vec2::new(0.0, 0.0),
        Vec2::new(0.0, 1.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(1.0, 0.0),
        // y = -1
        Vec2::new(0.0, 0.0),
        Vec2::new(0.0, 1.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(1.0, 0.0),
        // x = 1
        Vec2::new(1.0, 1.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(0.0, 0.0),
        Vec2::new(0.0, 1.0),
        // x = -1
        Vec2::new(0.0, 1.0),
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(1.0, 1.0),
    ];

    let texture = match image_::open("examples/data/rust.png") {
        Ok(image) => image.to_rgba8(),
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };
    let sampler = Nearest::new(texture);

    let mut win = mini_gl_fb::gotta_go_fast("Cube", w as f64, h as f64);

    let mut i = 0;
    win.glutin_handle_basic_input(|win, input| {
        let p = Mat4::perspective_fov_rh_no(1.4, w as f32, h as f32, 0.01, 100.0);
        let v = Mat4::<f32>::translation_3d(Vec3::new(0.0, 0.0, -2.0))
            * Mat4::<f32>::scaling_3d(0.6)
            * Mat4::rotation_x(0.6);
        let m = Mat4::rotation_x((i as f32 * 0.04).sin() * 0.4)
            * Mat4::rotation_y((i as f32 * 0.008) * 4.0)
            * Mat4::rotation_z((i as f32 * 0.06).cos() * 0.4);

        color.clear(180);
        depth.clear(1.0);

        let cube = Cube {
            mvp: p * v * m,
            positions: &positions,
            uvs: &uvs,
            sampler: &sampler,
        };
        cube.render(
            &[
                0, 3, 1, 1, 3, 2,
                4, 5, 7, 5, 6, 7,
                8, 11, 9, 9, 11, 10,
                12, 13, 15, 13, 14, 15,
                16, 17, 19, 17, 18, 19,
                20, 23, 21, 21, 23, 22,
            ],
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
