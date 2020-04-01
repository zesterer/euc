use criterion::{criterion_group, criterion_main, Bencher, Criterion};
use euc::{buffer::Buffer2d, rasterizer, Pipeline};
use std::{path::Path, time::Duration};
use vek::*;

struct Teapot<'a> {
    mvp: Mat4<f32>,
    positions: &'a [Vec3<f32>],
    normals: &'a [Vec3<f32>],
    light_dir: Vec3<f32>,
}

impl<'a> Pipeline for Teapot<'a> {
    type Vertex = u32; // Vertex index
    type VsOut = Vec3<f32>; // Normal
    type Pixel = u32; // BGRA

    #[inline(always)]
    fn vert(&self, v_index: &Self::Vertex) -> ([f32; 4], Self::VsOut) {
        let v_index = *v_index as usize;
        // Find vertex position
        let v_pos = self.positions[v_index] + Vec3::new(0.0, -0.5, 0.0); // Offset to center the teapot
        (
            // Calculate vertex position in camera space
            Vec4::from(self.mvp * Vec4::from_point(v_pos)).into_array(),
            // Find vertex normal
            self.normals[v_index],
        )
    }

    #[inline(always)]
    fn frag(&self, norm: &Self::VsOut) -> Self::Pixel {
        /*
        let ambient = 0.2;
        let diffuse = norm.dot(self.light_dir).max(0.0) * 0.5;
        let specular = self
            .light_dir
            .reflected(Vec3::from(self.mvp * Vec4::from(*norm)).normalized())
            .dot(-Vec3::unit_z())
            .powf(20.0);

        let light = ambient + diffuse + specular;
        let color = (Rgba::new(1.0, 0.7, 0.1, 1.0) * light).clamped(Rgba::zero(), Rgba::one());
        */

        let color = Rgba::broadcast(1.0);

        let bytes = (color * 255.0).map(|e| e as u8).into_array();
        (bytes[2] as u32) << 0
            | (bytes[1] as u32) << 8
            | (bytes[0] as u32) << 16
            | (bytes[3] as u32) << 24
    }
}

fn teapot_benchmark(b: &mut Bencher, &[width, height]: &[usize; 2]) {
    let mut color = Buffer2d::new([width, height], 0);
    let mut depth = Buffer2d::new([width, height], 1.0);

    let obj = tobj::load_obj(&Path::new("examples/data/teapot.obj")).unwrap();
    let indices = &obj.0[0].mesh.indices;
    let positions = obj.0[0]
        .mesh
        .positions
        .chunks(3)
        .map(|sl| Vec3::from_slice(sl))
        .collect::<Vec<_>>();
    let normals = obj.0[0]
        .mesh
        .normals
        .chunks(3)
        .map(|sl| Vec3::from_slice(sl))
        .collect::<Vec<_>>();

    let mvp = Mat4::perspective_rh_no(1.3, (width as f32) / (height as f32), 0.01, 100.0)
        * Mat4::<f32>::scaling_3d(0.8)
        * Mat4::rotation_x(0.002f32.sin() * 8.0)
        * Mat4::rotation_y(0.004f32.cos() * 4.0)
        * Mat4::rotation_z(0.008f32.sin() * 2.0);

    let shader = Teapot {
        mvp,
        positions: &positions,
        normals: &normals,
        light_dir: Vec3::new(1.0, 1.0, 1.0).normalized(),
    };

    b.iter(|| {
        shader.draw::<rasterizer::Triangles<_>, _>(indices, &mut color, &mut depth);
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "teapot",
        |b, &size| teapot_benchmark(b, size),
        &[[32, 32], [200, 200]], //, [640, 480], [800, 600], [1024, 800]],
    );
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .warm_up_time(Duration::from_millis(1000));
    targets = criterion_benchmark
}

criterion_main!(benches);
