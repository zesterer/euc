use derive_more::{Add, Mul};
use euc::{
    AaMode, Buffer2d, Clamped, DepthMode, Empty, LineTriangleList, Linear, Pipeline, PixelMode,
    Sampler, Target, Texture, Unit,
};
use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};
use std::marker::PhantomData;
use vek::*;

struct Teapot<'a> {
    m: Mat4<f32>,
    v: Mat4<f32>,
    p: Mat4<f32>,
    light_pos: Vec3<f32>,
    phantom: PhantomData<&'a ()>,
}

#[derive(Add, Mul, Clone)]
struct VertexData {
    wpos: Vec3<f32>,
    wnorm: Vec3<f32>,
}

impl<'a> Pipeline for Teapot<'a> {
    type Vertex = wavefront::Vertex<'a>;
    type VertexData = VertexData;
    type Primitives = LineTriangleList;
    type Fragment = Rgba<f32>;
    type Pixel = u32;

    fn depth_mode(&self) -> DepthMode {
        DepthMode::LESS_WRITE
    }
    fn aa_mode(&self) -> AaMode {
        AaMode::Msaa { level: 1 }
    }

    #[inline(always)]
    fn vertex(&self, vertex: &Self::Vertex) -> ([f32; 4], Self::VertexData) {
        let wpos = self.m * Vec4::from_point(Vec3::from(vertex.position()));
        let wnorm = self.m * Vec4::from_direction(-Vec3::from(vertex.normal().unwrap()));

        (
            (self.p * self.v * wpos).into_array(),
            VertexData {
                wpos: wpos.xyz(),
                wnorm: wnorm.xyz(),
            },
        )
    }

    #[inline(always)]
    fn fragment(&self, VertexData { wpos, wnorm }: Self::VertexData) -> Self::Fragment {
        Rgba::red()
    }

    #[inline(always)]
    fn blend(&self, _old: Self::Pixel, rgba: Self::Fragment) -> Self::Pixel {
        let rgba = rgba.map(|e| e.clamped(0.0, 1.0) * 255.0).as_();
        // The window's framebuffer uses BGRA format
        let bgra = Rgba::new(rgba.b, rgba.g, rgba.r, rgba.a);
        u32::from_le_bytes(bgra.into_array())
    }
}

fn main() {
    let [w, h] = [1280, 960];

    let mut color = Buffer2d::fill([w, h], 0x0);
    let mut depth = Buffer2d::fill([w, h], 1.0);

    let model = wavefront::Obj::from_file("examples/data/teapot.obj").unwrap();

    let mut win = Window::new("Teapot", w, h, WindowOptions::default()).unwrap();

    let mut ori = Vec2::new(0.0, 0.0);
    let mut dist = 6.0;
    let mut old_mouse_pos = (0.0, 0.0);

    let mut i = 0;
    while win.is_open() && !win.is_key_down(Key::Escape) {
        let start_time = std::time::Instant::now();

        // Clear the render targets ready for the next frame
        color.clear(0x0);
        depth.clear(1.0);

        // Update camera as the mouse moves
        let mouse_pos = win.get_mouse_pos(MouseMode::Pass).unwrap_or_default();
        if win.get_mouse_down(MouseButton::Left) {
            ori.x -= (mouse_pos.1 - old_mouse_pos.1) * 0.003;
            ori.y += (mouse_pos.0 - old_mouse_pos.0) * 0.003;
        }
        if win.get_mouse_down(MouseButton::Right) {
            dist = (dist + (mouse_pos.1 - old_mouse_pos.1) as f32 * 0.01)
                .max(1.0)
                .min(20.0);
        }
        old_mouse_pos = mouse_pos;

        // Position of objects in the scene
        let teapot_pos = Vec3::new(0.0, 0.0, 0.0);
        let light_pos = Vec3::<f32>::new(-8.0, 5.0, -5.0);

        // Set up the camera matrix
        let p = Mat4::perspective_fov_lh_zo(1.3, w as f32, h as f32, 0.01, 100.0);
        let v = Mat4::<f32>::identity() * Mat4::translation_3d(Vec3::new(0.0, 0.0, dist));
        // Set up the teapot matrix
        let m = Mat4::<f32>::translation_3d(-teapot_pos)
            * Mat4::rotation_x(core::f32::consts::PI)
            * Mat4::rotation_x(ori.x)
            * Mat4::rotation_y(ori.y);

        // Colour pass
        Teapot {
            m,
            v,
            p,
            light_pos,
            phantom: PhantomData,
        }
        .render(model.vertices(), &mut color, &mut depth);

        win.update_with_buffer(color.raw(), w, h).unwrap();

        if i % 60 == 0 {
            let elapsed = start_time.elapsed();
            win.set_title(&format!(
                "Teapot (Time = {:?}, FPS = {})",
                elapsed,
                1.0 / elapsed.as_secs_f32()
            ));
        }
        i += 1;
    }
}

/*
use euc::{buffer::Buffer2d, rasterizer, Pipeline, Target};
use std::path::Path;
use vek::*;

struct Teapot<'a> {
    mvp: Mat4<f32>,
    positions: &'a [Vec4<f32>],
    normals: &'a [Vec3<f32>],
    light_dir: Vec3<f32>,
}

impl<'a> Pipeline for Teapot<'a> {
    type Vertex = u32; // Vertex index
    type VertexData = Vec3<f32>; // Normal
    type Fragment = u32; // BGRA
    type Pixel = u32; // BGRA

    fn vertex_shader(&self, v_index: &Self::Vertex) -> ([f32; 4], Self::VsOut) {
        let v_index = *v_index as usize;
        // Find vertex position
        (
            // Calculate vertex position in camera space
            (self.mvp * self.positions[v_index]).into_array(),
            // Find vertex normal
            self.normals[v_index],
        )
    }

    fn fragment_shader(&self, norm: Self::VertexData) -> Self::Pixel {
        let ambient = 0.2;
        let diffuse = norm.dot(self.light_dir).max(0.0) * 0.5;
        let specular = self
            .light_dir
            .reflected(Vec3::from(self.mvp * Vec4::from(*norm)).normalized())
            .dot(-Vec3::unit_z())
            .powf(20.0);

        let light = ambient + diffuse + specular;
        let color = (Rgba::new(1.0, 0.7, 0.1, 1.0) * light).clamped(Rgba::zero(), Rgba::one());

        let bytes = (color * 255.0).map(|e| e as u8).into_array();
        (bytes[2] as u32) << 0
            | (bytes[1] as u32) << 8
            | (bytes[0] as u32) << 16
            | (bytes[3] as u32) << 24
    }
}

struct Wireframe<'a> {
    mvp: &'a Mat4<f32>,
    positions: &'a [Vec4<f32>],
    normals: &'a [Vec3<f32>],
}

impl<'a> Pipeline for Wireframe<'a> {
    type Vertex = u32; // Vertex index
    type VertexData = ();
    type Fragment = u32; // BGRA
    type Pixel = u32; // BGRA

    #[inline]
    fn vertex_shader(&self, v_index: &Self::Vertex) -> ([f32; 4], Self::VsOut) {
        let v_index = *v_index as usize;
        // Offset position to avoid z fighting
        let offset = 0.002 * self.normals[v_index];
        let v_pos = self.positions[v_index] + offset;
        ((*self.mvp * Vec4::from_point(v_pos)).into_array(), ())
    }

    #[inline]
    fn fragment_shader(&self, _: Self::VertexData) -> Self::Pixel {
        120
    }
}

const W: usize = 800;
const H: usize = 600;

fn main() {
    let mut color = Buffer2d::new([W, H], 0);
    let mut depth = Buffer2d::new([W, H], 1.0);

    let mut win = minifb::Window::new("Teapot", W, H, minifb::WindowOptions::default()).unwrap();

    let model = wavefront::Obj::from_file("examples/data/teapot.obj").unwrap();
    let wf_indices: Vec<_> = model
        .triangles()
        .chunks(3)
        .flat_map(|sl| [sl[0], sl[1], sl[1], sl[2], sl[2], sl[0]]
            .map(|v| v.position_index())
            .into_iter())
        .collect();
    let positions = model
        .positions()
        .chunks(3)
        .map(|sl| Vec4::from_point(Vec3::from_slice(sl) + Vec3::new(0.0, -0.5, 0.0)))
        .collect::<Vec<_>>();
    let normals = model
        .normals()
        .chunks(3)
        .map(|sl| Vec3::from_slice(sl))
        .collect::<Vec<_>>();

    for i in 0.. {
        let mvp = Mat4::perspective_fov_rh_no(1.3, W as f32, H as f32, 0.01, 100.0)
            * Mat4::translation_3d(Vec3::new(0.0, 0.0, -1.5))
            * Mat4::<f32>::scaling_3d(0.8)
            * Mat4::rotation_x((i as f32 * 0.002).sin() * 8.0)
            * Mat4::rotation_y((i as f32 * 0.004).cos() * 4.0)
            * Mat4::rotation_z((i as f32 * 0.008).sin() * 2.0);

        color.clear(0);
        depth.clear(1.0);

        Teapot {
            mvp: mvp,
            positions: &positions,
            normals: &normals,
            light_dir: Vec3::new(1.0, 1.0, 1.0).normalized(),
        }
        .draw::<rasterizer::Triangles<_>, _>(indices, &mut color, Some(&mut depth));
        Wireframe {
            mvp: &mvp,
            positions: &positions,
            normals: &normals,
        }
        .draw::<rasterizer::Lines<_>, _>(&wf_indices, &mut color, Some(&mut depth));

        if win.is_open() {
            win.update_with_buffer(color.as_ref(), W, H).unwrap();
        } else {
            break;
        }
    }
}
*/
