use euc::{buffer::Buffer2d, rasterizer, Pipeline, Target};
use image::RgbImage;
use minifb::Window;
use vek::{Mat4, Vec2, Vec3, Vec4};

struct Cube<'a> {
    mvp: &'a Mat4<f32>,
    positions: &'a [Vec4<f32>],
    uvs: &'a [Vec2<f32>],
    texture: &'a RgbImage,
}

impl<'a> Pipeline for Cube<'a> {
    type Vertex = usize;
    type VsOut = Vec2<f32>;
    type Pixel = u32;

    #[inline]
    fn vert(&self, v_index: &Self::Vertex) -> ([f32; 4], Self::VsOut) {
        (
            (*self.mvp * self.positions[*v_index]).into_array(),
            self.uvs[*v_index],
        )
    }

    #[inline]
    fn frag(&self, v_uv: &Self::VsOut) -> Self::Pixel {
        // Convert interpolated uv coordinate to texture coordinate
        let (width, height) = (self.texture.width() as f32, self.texture.height() as f32);
        let x = f32::min(f32::max(0.0, v_uv.x * width), width - 1.0);
        let y = f32::min(f32::max(0.0, v_uv.y * height), height - 1.0);
        // Lookup pixel and convert to appropriate format
        let rgb = self.texture.get_pixel(x as u32, y as u32);
        255 << 24 | (rgb[0] as u32) << 16 | (rgb[1] as u32) << 8 | (rgb[2] as u32) << 0
    }
}

const W: usize = 800;
const H: usize = 600;

fn main() {
    let mut color = Buffer2d::new([W, H], 0);
    let mut depth = Buffer2d::new([W, H], 1.0);

    let mut win = Window::new("Cube", W, H, minifb::WindowOptions::default()).unwrap();
    let vp = Mat4::perspective_fov_rh_no(1.4, W as f32, H as f32, 0.01, 100.0)
        * Mat4::<f32>::translation_3d(Vec3::new(0.0, 0.0, -2.0))
        * Mat4::<f32>::scaling_3d(0.6)
        * Mat4::rotation_x(0.6);

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
    let texture = match image::open("examples/data/checkerboard.png") {
        Ok(image) => image.to_rgb(),
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };

    let mut i = 0;
    while win.is_open() {
        let mvp = vp * Mat4::rotation_y(-i as f32 * 0.006);

        color.clear(180);
        depth.clear(1.0);

        let cube = Cube {
            mvp: &mvp,
            positions: &positions,
            uvs: &uvs,
            texture: &texture,
        };
        cube.draw::<rasterizer::Triangles<_, rasterizer::BackfaceCullingEnabled>, _>(
            &[
                // z = 1
                0, 3, 1, 1, 3, 2, // z = -1
                4, 5, 7, 5, 6, 7, // y = 1
                8, 11, 9, 9, 11, 10, // y = -1,
                12, 13, 15, 13, 14, 15, // x = 1,
                16, 17, 19, 17, 18, 19, // x = -1,
                20, 23, 21, 21, 23, 22,
            ],
            &mut color,
            &mut depth,
        );

        win.update_with_buffer(color.as_ref()).unwrap();
        i += 1;
    }
}
