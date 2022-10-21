use euc::{Buffer2d, Pipeline, Target, TriangleList, Sampler, Nearest, Texture};
use image_::RgbaImage;
use vek::{Mat4, Vec2, Vec3, Vec4, Rgba};
use minifb::{Key, Window, WindowOptions};

struct Cube<'a> {
    mvp: Mat4<f32>,
    positions: &'a [Vec4<f32>],
    uvs: &'a [Vec2<f32>],
    sampler: &'a Nearest<RgbaImage>,
}

impl<'a> Pipeline for Cube<'a> {
    type Vertex = usize;
    type VertexData = Vec2<f32>;
    type Primitives = TriangleList;
    type Fragment = Rgba<f32>;
    type Pixel = u32;

    #[inline]
    fn vertex_shader(&self, v_index: &Self::Vertex) -> ([f32; 4], Self::VertexData) {
        (
            (self.mvp * self.positions[*v_index]).into_array(),
            self.uvs[*v_index],
        )
    }

    #[inline]
    fn fragment_shader(&self, v_uv: Self::VertexData) -> Self::Fragment {
        Rgba::from(self.sampler.sample(v_uv.into_array()).0).map(|e: u8| e as f32)
    }

    fn blend_shader(&self, _: Self::Pixel, color: Self::Fragment) -> Self::Pixel {
        u32::from_le_bytes(color.map(|e| e as u8).into_array())
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
    let sampler = texture.nearest();

    let mut win = Window::new("Texture Mapping", w, h, WindowOptions::default()).unwrap();

    let mut i = 0;
    while win.is_open() && !win.is_key_down(Key::Escape) {
        let p = Mat4::perspective_fov_rh_no(1.4, w as f32, h as f32, 0.01, 100.0);
        let v = Mat4::<f32>::translation_3d(Vec3::new(0.0, 0.0, -2.0))
            * Mat4::<f32>::scaling_3d(0.6)
            * Mat4::rotation_x(0.6);
        let m = Mat4::rotation_x((i as f32 * 0.004).sin() * 0.4)
            * Mat4::rotation_y((i as f32 * 0.0008) * 4.0)
            * Mat4::rotation_z((i as f32 * 0.006).cos() * 0.4);

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
            &mut color,
            &mut depth,
        );

        win.update_with_buffer(color.raw(), w, h).unwrap();

        i += 1;
    }
}
