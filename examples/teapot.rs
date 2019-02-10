use std::path::Path;
use euc::{
    Pipeline,
    rasterizer,
    buffer::Buffer2d,
    Target,
};
use minifb;
use tobj;
use vek::*;

struct Teapot<'a> {
    phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> Pipeline for Teapot<'a> {
    type Uniform = (
        Mat4<f32>, // Camera matrix
        &'a [f32], // Positions
        &'a [f32], // Normals
    );
    type Vertex = u32; // Vertex index
    type VsOut = Vec3<f32>; // Normal
    type Pixel = u32; // BGRA

    #[inline(always)]
    fn vert(
        (cam_mat, pos, norms): &Self::Uniform,
        v_index: &Self::Vertex,
    ) -> ([f32; 3], Self::VsOut) {
        let v_index = *v_index as usize;
        // Find vertex position
        let v_pos = Vec3::from_slice(&pos[v_index * 3..v_index * 3 + 3])
            + Vec3::new(0.0, -0.5, 0.0); // Offset to center the teapot
        (
            // Calculate vertex position in camera space
            Vec3::from(*cam_mat * Vec4::from_point(v_pos)).into_array(),
            // Find vertex normal
            Vec3::from_slice(&norms[v_index * 3..v_index * 3 + 3]),
        )
    }

    #[inline(always)]
    fn frag((cam_mat, _, _): &Self::Uniform, norm: &Self::VsOut) -> Self::Pixel {
        let light_dir = Vec3::new(1.0, 1.0, 1.0).normalized();

        let ambient = 0.2;
        let diffuse = norm.dot(light_dir).max(0.0) * 0.5;
        let specular = light_dir.reflected(Vec3::from(*cam_mat * Vec4::from(*norm)).normalized()).dot(-Vec3::unit_z()).powf(20.0);

        let light = ambient + diffuse + specular;
        let color = (Rgba::new(1.0, 0.7, 0.1, 1.0) * light).clamped(Rgba::zero(), Rgba::one());

        let bytes = (color * 255.0).map(|e| e as u8).into_array();
        (bytes[2] as u32) << 0 |
        (bytes[1] as u32) << 8 |
        (bytes[0] as u32) << 16 |
        (bytes[3] as u32) << 24
    }
}

const W: usize = 800;
const H: usize = 600;

fn main() {
    let mut color = Buffer2d::new([W, H], 0);
    let mut depth = Buffer2d::new([W, H], 1.0);

    let teapot = tobj::load_obj(&Path::new("examples/data/teapot.obj")).unwrap();

    let mut win = minifb::Window::new("Teapot", W, H, minifb::WindowOptions::default()).unwrap();

    for i in 0.. {
        let cam_mat =
            Mat4::perspective_rh_no(1.3, 1.35, 0.01, 100.0) *
            Mat4::<f32>::scaling_3d(0.8) *
            Mat4::rotation_x((i as f32 * 0.002).sin() * 8.0) *
            Mat4::rotation_y((i as f32 * 0.004).cos() * 4.0);
            Mat4::rotation_z((i as f32 * 0.008).sin() * 2.0);

        color.clear(0);
        depth.clear(1.0);

        for model in &teapot.0 {
            Teapot::draw::<rasterizer::Lines<_>, _>(
                &(
                    cam_mat,
                    &model.mesh.positions,
                    &model.mesh.normals,
                ),
                &model.mesh.indices,
                &mut color,
                &mut depth,
            );
        }

        if win.is_open() {
            win.update_with_buffer(color.as_ref()).unwrap();
        } else {
            break;
        }
    }
}
