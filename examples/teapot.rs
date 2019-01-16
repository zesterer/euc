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
    type Uniform = (Mat4<f32>, &'a [f32], &'a [f32]);
    type Vertex = u32;
    type VsOut = Vec3<f32>;
    type Pixel = u32;

    #[inline(always)]
    fn vert(
        (cam_mat, positions, normals): &Self::Uniform,
        vertex: &Self::Vertex,
    ) -> ([f32; 3], Self::VsOut) {
        (
            Vec3::from(*cam_mat * Vec4::new(
                positions[*vertex as usize * 3 + 0],
                positions[*vertex as usize * 3 + 1] - 0.5,
                positions[*vertex as usize * 3 + 2],
                1.0,
            )).into_array(),
            Vec3::new(
                normals[*vertex as usize * 3 + 0],
                normals[*vertex as usize * 3 + 1],
                normals[*vertex as usize * 3 + 2],
            ).normalized(),
        )
    }

    #[inline(always)]
    fn frag((cam_mat, _, _): &Self::Uniform, normal: &Self::VsOut) -> Self::Pixel {
        let light_dir = Vec3::new(1.0, 1.0, 1.0).normalized();

        let cam_normal = *cam_mat * Vec4::from(*normal);

        let ambient = 0.2;
        let diffuse = normal.dot(light_dir).max(0.0) * 0.5;
        let specular = light_dir.reflected(Vec3::from(cam_normal).normalized()).dot(-Vec3::unit_z()).powf(20.0);
        let light = (ambient + diffuse + specular).min(1.0);

        let color = Rgba::new(1.0, 0.9, 0.7, 1.0) * light;

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

    let teapot = tobj::load_obj(&Path::new("examples/data/teapot.obj")).unwrap().0.remove(0);

    let mut win = minifb::Window::new("Teapot", W, H, minifb::WindowOptions::default()).unwrap();

    for i in 0.. {
        let cam_mat =
            Mat4::perspective_rh_no(1.3, 1.35, 0.01, 100.0) *
            Mat4::<f32>::scaling_3d(0.5) *
            Mat4::rotation_x((i as f32 * 0.002).sin() * 8.0) *
            Mat4::rotation_y((i as f32 * 0.004).cos() * 4.0);
            Mat4::rotation_z((i as f32 * 0.008).sin() * 2.0);

        color.clear(0);
        depth.clear(1.0);

        Teapot::draw::<rasterizer::Triangles<_>, _>(
            &(cam_mat, &teapot.mesh.positions, &teapot.mesh.normals),
            &teapot.mesh.indices,
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
