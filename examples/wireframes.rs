use euc::{Buffer2d, Empty, LineTriangleList, Pipeline, Target, Unit};
use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};
use std::marker::PhantomData;
use vek::*;

struct Teapot<'a> {
    m: Mat4<f32>,
    v: Mat4<f32>,
    p: Mat4<f32>,
    phantom: PhantomData<&'a ()>,
}

impl<'a> Pipeline for Teapot<'a> {
    type Vertex = wavefront::Vertex<'a>;
    type VertexData = Unit;
    type Primitives = LineTriangleList;
    type Fragment = Rgba<f32>;
    type Pixel = u32;

    #[inline(always)]
    fn vertex(&self, vertex: &Self::Vertex) -> ([f32; 4], Self::VertexData) {
        let wpos = self.m * Vec4::from_point(Vec3::from(vertex.position()));

        ((self.p * (self.v * wpos)).into_array(), Unit)
    }

    #[inline(always)]
    fn fragment(&self, _: Self::VertexData) -> Self::Fragment {
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

        // Update camera as the mouse moves
        let mouse_pos = win.get_mouse_pos(MouseMode::Pass).unwrap_or_default();
        if win.get_mouse_down(MouseButton::Left) {
            ori -= Vec2::new(mouse_pos.1 - old_mouse_pos.1, mouse_pos.0 - old_mouse_pos.0) * 0.003;
        }
        if win.get_mouse_down(MouseButton::Right) {
            dist = (dist + (mouse_pos.1 - old_mouse_pos.1) as f32 * 0.01)
                .max(1.0)
                .min(20.0);
        }
        old_mouse_pos = mouse_pos;

        // Position of objects in the scene
        let teapot_pos = Vec3::new(0.0, 0.0, 0.0);

        // Set up the camera matrix
        let p = Mat4::perspective_fov_lh_zo(1.3, w as f32, h as f32, 0.01, 100.0);
        let v = Mat4::<f32>::identity()
            * Mat4::translation_3d(Vec3::new(0.0, 0.0, dist))
            * Mat4::rotation_x(ori.x)
            * Mat4::rotation_y(ori.y);
        // Set up the teapot matrix
        let m = Mat4::<f32>::translation_3d(-teapot_pos) * Mat4::rotation_x(core::f32::consts::PI);

        // Colour pass
        Teapot {
            m,
            v,
            p,
            phantom: PhantomData,
        }
        .render(model.vertices(), &mut color, &mut Empty::default());

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
