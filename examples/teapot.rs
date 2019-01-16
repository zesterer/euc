use std::path::Path;
use euc::{
    Pipeline,
    rasterizer,
    buffer::Buffer2d,
    Target,
};
use mini_gl_fb;
use tobj;
use vek::*;

struct Teapot<'a> {
    phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> Pipeline for Teapot<'a> {
    type Uniform = (Mat4<f32>, &'a [f32], &'a [f32]);
    type Vertex = u32;
    type VsOut = Vec3<f32>;
    type Pixel = [u8; 4];

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
    fn frag(_: &Self::Uniform, normal: &Self::VsOut) -> Self::Pixel {
        let light_dir = Vec3::new(1.0, 1.0, 1.0).normalized();
        let light_factor = normal.dot(light_dir).max(0.0) * 0.8 + 0.2;
        (Rgba::new(1.0, 0.9, 0.7, 1.0) * light_factor).map(|e| (255.0 * e) as u8).into_array()
    }
}

const W: usize = 640;
const H: usize = 480;

fn main() {
    let mut color = Buffer2d::new([W, H], [0; 4]);
    let mut depth = Buffer2d::new([W, H], 1.0);

    let mut win = mini_gl_fb::gotta_go_fast("Teapot", W as f64, H as f64);

    let teapot = tobj::load_obj(&Path::new("examples/data/teapot.obj")).unwrap().0.remove(0);

    for i in 0.. {
        let cam_mat =
            Mat4::perspective_rh_no(1.3, 1.35, 0.01, 100.0) *
            Mat4::<f32>::scaling_3d(0.5) *
            Mat4::rotation_x((i as f32 * 0.01).sin() * 3.0) *
            Mat4::rotation_y((i as f32 * 0.02).cos() * 2.0);
            Mat4::rotation_z((i as f32 * 0.003).sin() * 10.0);

        color.clear([0; 4]);
        depth.clear(1.0);

        Teapot::draw::<rasterizer::Triangles<_>, _>(
            &(cam_mat, &teapot.mesh.positions, &teapot.mesh.normals),
            &teapot.mesh.indices,
            &mut color,
            &mut depth,
        );

        win.update_buffer(color.as_ref());

        if !win.is_running() {
            break;
        }
    }
}
