use vek::*;
use derive_more::{Add, Mul};
use euc::{Pipeline, Buffer2d, Target, DepthMode, TriangleList, CullMode};
use std::marker::PhantomData;

struct Teapot<'a> {
    m: Mat4<f32>,
    v: Mat4<f32>,
    p: Mat4<f32>,
    phantom: PhantomData<&'a ()>,
}

#[derive(Add, Mul, Clone)]
struct VertexAttr {
    wpos: Vec3<f32>,
    wnorm: Vec3<f32>,
}

impl<'a> Pipeline for Teapot<'a> {
    type Vertex = wavefront::Vertex<'a>;
    type VertexAttr = VertexAttr;
    type Primitives = TriangleList;
    type Fragment = u32;

    fn depth_mode(&self) -> DepthMode { DepthMode::LESS_WRITE }

    #[inline(always)]
    fn vertex_shader(&self, vertex: &Self::Vertex) -> ([f32; 4], Self::VertexAttr) {
        let wpos = self.m * Vec4::from_point(Vec3::from(vertex.position()));
        let wnorm = self.m * Vec4::from_direction(-Vec3::from(vertex.normal().unwrap()));
        (
            (self.p * self.v * wpos).into_array(),
            VertexAttr { wpos: wpos.xyz(), wnorm: wnorm.xyz() },
        )
    }

    #[inline(always)]
    fn fragment_shader(&self, VertexAttr { wpos, wnorm }: Self::VertexAttr) -> Self::Fragment {
        let wnorm = wnorm.normalized();
        let light_dir = Vec3::<f32>::new(1.0, 1.0, 1.0).normalized();
        let cam_pos = Vec3::zero();
        let cam_dir = (wpos - cam_pos).normalized();
        let surf_color = Rgba::new(0.8, 1.0, 0.7, 1.0);

        // Phong reflection model
        let ambient = 0.1;
        let diffuse = wnorm.dot(-light_dir).max(0.0) * 0.5;
        let specular = light_dir.reflected(wnorm).dot(-cam_dir).max(0.0).powf(30.0) * 3.0;

        let color = surf_color * (ambient + diffuse + specular);
        u32::from_le_bytes(color.map(|e| e.clamped(0.0, 1.0) * 255.0).as_().into_array())
    }
}

fn main() {
    let [w, h] = [800, 600];

    let mut color = Buffer2d::fill([w, h], 0x0);
    let mut depth = Buffer2d::fill([w, h], 1.0);

    let model = wavefront::Obj::from_file("examples/data/teapot.obj").unwrap();

    let mut win = mini_gl_fb::gotta_go_fast("Teapot", w as f64, h as f64);

    let mut i = 0;
    win.glutin_handle_basic_input(|win, input| {
        let p = Mat4::perspective_fov_lh_zo(1.3, w as f32, h as f32, 0.01, 100.0);
        let v = Mat4::identity();
        let m = Mat4::<f32>::translation_3d(Vec3::new(0.0, 0.0, 6.0))
            * Mat4::rotation_x((i as f32 * 0.03).sin() * 0.4)
            * Mat4::rotation_y((i as f32 * 0.005) * 4.0)
            * Mat4::rotation_z((i as f32 * 0.04).cos() * 0.4);

        color.clear(0x0);
        depth.clear(1.0);

        Teapot { m, v, p, phantom: PhantomData }.render(
            model.vertices(),
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
