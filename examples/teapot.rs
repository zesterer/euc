use std::path::Path;
use vek::*;
use euc::{Pipeline, Buffer2d, Target, DepthMode, Triangles, CullMode};

struct Teapot<'a> {
    mvp: Mat4<f32>,
    positions: &'a [Vec3<f32>],
    normals: &'a [Vec3<f32>],
    light_dir: Vec3<f32>,
}

impl<'a> Pipeline for Teapot<'a> {
    type Vertex = usize; // Vertex index
    type VsOut = Rgba<f32>; // Color
    type Fragment = u32; // BGRA

    fn depth_mode(&self) -> DepthMode { DepthMode::LESS_WRITE }

    #[inline(always)]
    fn vertex_shader(&self, index: &Self::Vertex) -> ([f32; 4], Self::VsOut) {
        let pos = self.mvp * Vec4::from_point(self.positions[*index]);
        let norm = self.normals[*index];

        let ambient = 0.2;
        let diffuse = norm.dot(self.light_dir).max(0.0) * 0.5;
        let specular = self
            .light_dir
            .reflected(Vec3::from(self.mvp * Vec4::from(norm)).normalized())
            .dot(-Vec3::unit_z())
            .powf(20.0);
        let light = ambient + diffuse + specular;

        let color = (Rgba::new(1.0, 0.7, 0.1, 1.0) * light).clamped(Rgba::zero(), Rgba::one());

        (pos.into_array(), color)
    }

    #[inline(always)]
    fn fragment_shader(&self, color: Self::VsOut) -> Self::Fragment {
        let bytes = (color * 255.0).map(|e| e as u8).into_array();
        (bytes[0] as u32) << 0
            | (bytes[1] as u32) << 8
            | (bytes[2] as u32) << 16
            | (bytes[3] as u32) << 24
    }
}

const W: usize = 800;
const H: usize = 600;

fn main() {
    let mut color = Buffer2d::fill([W, H], 0);
    let mut depth = Buffer2d::fill([W, H], 1.0);

    let obj = tobj::load_obj(&Path::new("examples/data/teapot.obj"), false).unwrap();
    let indices = obj.0[0]
        .mesh
        .indices
        .iter()
        .map(|i| *i as usize)
        .collect::<Vec<_>>();
    let positions = obj.0[0]
        .mesh
        .positions
        .chunks(3)
        .map(|sl| Vec3::from_slice(sl) - Vec3::unit_y() * 0.5) // Center model
        .collect::<Vec<_>>();
    let normals = obj.0[0]
        .mesh
        .normals
        .chunks(3)
        .map(|sl| Vec3::from_slice(sl))
        .collect::<Vec<_>>();

    let mut win = mini_gl_fb::gotta_go_fast("Teapot", W as f64, H as f64);

    let mut i = 0;
    win.glutin_handle_basic_input(|win, input| {
        let mvp = Mat4::perspective_fov_rh_no(1.3, W as f32, H as f32, 0.01, 100.0)
            * Mat4::translation_3d(Vec3::new(0.0, 0.0, -1.5))
            * Mat4::<f32>::scaling_3d(0.8)
            * Mat4::rotation_x((i as f32 * 0.002) * 8.0)
            * Mat4::rotation_y((i as f32 * 0.004) * 4.0)
            * Mat4::rotation_z((i as f32 * 0.008) * 2.0);

        color.clear(0);
        depth.clear(1.0);

        Teapot {
            mvp,
            positions: &positions,
            normals: &normals,
            light_dir: Vec3::new(1.0, 1.0, 1.0).normalized(),
        }
        .render(
            Triangles(CullMode::Back),
            indices.as_slice(),
            &mut color,
            &mut depth,
        );

        win.update_buffer(color.raw());
        win.redraw();

        i += 1;
        true
    });
}
