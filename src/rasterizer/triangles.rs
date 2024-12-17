use super::*;
use crate::{CoordinateMode, YAxisDirection};

#[cfg(feature = "micromath")]
use micromath::F32Ext;

/// A rasterizer that produces filled triangles.
#[derive(Copy, Clone, Debug, Default)]
pub struct Triangles;

impl Rasterizer for Triangles {
    type Config = CullMode;

    #[inline]
    unsafe fn rasterize<V, I, B>(
        &self,
        mut vertices: I,
        _principal_x: bool,
        coords: CoordinateMode,
        cull_mode: CullMode,
        mut blitter: B,
    ) where
        V: Clone + WeightedSum,
        I: Iterator<Item = ([f32; 4], V)>,
        B: Blitter<V>,
    {
        let tgt_size = blitter.target_size();
        let tgt_min = blitter.target_min();
        let tgt_max = blitter.target_max();

        let cull_dir = match cull_mode {
            CullMode::None => None,
            CullMode::Back => Some(1.0),
            CullMode::Front => Some(-1.0),
        };

        let flip = match coords.y_axis_direction {
            YAxisDirection::Down => [1.0f32, 1.0],
            YAxisDirection::Up => [1.0f32, -1.0],
        };

        let [size_x, size_y] = tgt_size.map(|e| e as f32);

        let to_ndc = [
            [2.0 / size_x, 0.0, -1.0],
            [0.0, -2.0 / size_y, 1.0],
            [0.0, 0.0, 1.0],
        ];

        let verts_hom_out = core::iter::from_fn(move || {
            Some([vertices.next()?, vertices.next()?, vertices.next()?])
        });

        verts_hom_out.for_each(|verts_hom_out: [([f32; 4], V); 3]| {
            blitter.begin_primitive();

            // Calculate vertex shader outputs and vertex homogeneous coordinates
            let verts_hom = [verts_hom_out[0].0, verts_hom_out[1].0, verts_hom_out[2].0];
            let verts_out = verts_hom_out.map(|(_, v)| v);

            let verts_hom = verts_hom.map(|[a0, a1, a2, a3]| [a0 * flip[0], a1 * flip[1], a2, a3]);

            // Convert homogenous to euclidean coordinates
            let verts_euc = verts_hom.map(|[a0, a1, a2, a3]| [a0 / a3, a1 / a3, a2 / a3]);

            // Calculate winding direction to determine culling behaviour
            let winding = cross(
                sub(verts_euc[1], verts_euc[0]),
                sub(verts_euc[2], verts_euc[0]),
            )[2];

            // Culling and correcting for winding
            let (verts_hom, verts_euc, verts_out) = if cull_dir
                .map(|cull_dir| winding * cull_dir < 0.0)
                .unwrap_or(false)
            {
                return; // Cull the triangle
            } else if winding >= 0.0 {
                // Reverse vertex order
                (rev(verts_hom), rev(verts_euc), rev(verts_out))
            } else {
                (verts_hom, verts_euc, verts_out)
            };

            // Create a matrix that allows conversion between screen coordinates and interpolation weights
            let coords_to_weights = {
                let [a, b, c] = [verts_hom[0], verts_hom[1], verts_hom[2]];
                let c = [c[0], c[1], c[3]];
                let ca = sub([a[0], a[1], a[3]], c);
                let cb = sub([b[0], b[1], b[3]], c);
                let n = cross(ca, cb);
                let rec_det = if magnitude_squared(n) > 0.0 {
                    1.0 / dot(n, c).min(-core::f32::EPSILON)
                } else {
                    1.0
                };

                matmul(
                    [cross(cb, c), cross(c, ca), n].map(|v| v.map(|e| e * rec_det)),
                    to_ndc,
                )
            };

            // Ensure we didn't accidentally end up with infinities or NaNs
            debug_assert!(coords_to_weights
                .iter()
                .all(|v| v.iter().all(|e| e.is_finite())));

            // Convert vertex coordinates to screen space
            let verts_screen = verts_euc
                .map(|[a0, a1, _a2]| [size_x * (a0 * 0.5 + 0.5), size_y * (a1 * -0.5 + 0.5)]);

            // Calculate the triangle bounds as a bounding box
            let screen_min = tgt_min.map(|e| e as usize);
            let screen_max = tgt_max.map(|e| e as usize);
            let bounds_clamped_min = [
                ((verts_screen[0][0]
                    .min(verts_screen[1][0])
                    .min(verts_screen[2][0])
                    + 0.) as usize)
                    .clamp(screen_min[0], screen_max[0]),
                ((verts_screen[0][1]
                    .min(verts_screen[1][1])
                    .min(verts_screen[2][1])
                    + 0.) as usize)
                    .clamp(screen_min[1], screen_max[1]),
            ];
            let bounds_clamped_max = [
                ((verts_screen[0][0]
                    .max(verts_screen[1][0])
                    .max(verts_screen[2][0])
                    + 1.) as usize)
                    .clamp(screen_min[0], screen_max[0]),
                ((verts_screen[0][1]
                    .max(verts_screen[1][1])
                    .max(verts_screen[2][1])
                    + 1.) as usize)
                    .clamp(screen_min[1], screen_max[1]),
            ];

            // Calculate change in vertex weights for each pixel
            let weights_at = |[p0, p1]: [f32; 2]| mat3_mul_vec3(coords_to_weights, [p0, p1, 1.0]);
            let w_hom_origin = weights_at([0., 0.]);
            let w_hom_dx = sub(weights_at([1000.0, 0.]), w_hom_origin).map(|e| e * (1.0 / 1000.0));
            let w_hom_dy = sub(weights_at([0., 1000.0]), w_hom_origin).map(|e| e * (1.0 / 1000.0));

            // First, order vertices by height
            let min_y = {
                let y = verts_screen.map(|v| v[1]);
                y[0].min(y[1]).min(y[2])
            };
            let verts_by_y = if verts_screen[0][1] == min_y {
                if verts_screen[1][1] < verts_screen[2][1] {
                    [verts_screen[0], verts_screen[1], verts_screen[2]]
                } else {
                    [verts_screen[0], verts_screen[2], verts_screen[1]]
                }
            } else if verts_screen[1][1] == min_y {
                if verts_screen[0][1] < verts_screen[2][1] {
                    [verts_screen[1], verts_screen[0], verts_screen[2]]
                } else {
                    [verts_screen[1], verts_screen[2], verts_screen[0]]
                }
            } else {
                #[allow(clippy::collapsible_else_if)]
                if verts_screen[0][1] < verts_screen[1][1] {
                    [verts_screen[2], verts_screen[0], verts_screen[1]]
                } else {
                    [verts_screen[2], verts_screen[1], verts_screen[0]]
                }
            };

            if let [true, true, true] = verts_euc.map(|v| coords.passes_z_clip(v[2])) {
                rasterize::<_, _, true>(
                    coords.clone(),
                    bounds_clamped_min,
                    bounds_clamped_max,
                    verts_by_y,
                    verts_hom,
                    w_hom_origin,
                    w_hom_dx,
                    w_hom_dy,
                    verts_out,
                    &mut blitter,
                );
            } else {
                rasterize::<_, _, false>(
                    coords.clone(),
                    bounds_clamped_min,
                    bounds_clamped_max,
                    verts_by_y,
                    verts_hom,
                    w_hom_origin,
                    w_hom_dx,
                    w_hom_dy,
                    verts_out,
                    &mut blitter,
                );
            }

            // Iterate over fragment candidates within the triangle's bounding box
            #[inline]
            unsafe fn rasterize<
                V: Clone + WeightedSum,
                B: Blitter<V>,
                const NO_VERTS_CLIPPED: bool,
            >(
                coords: CoordinateMode,
                bounds_clamped_min: [usize; 2],
                bounds_clamped_max: [usize; 2],
                verts_by_y: [[f32; 2]; 3],
                verts_hom: [[f32; 4]; 3],
                w_hom_origin: [f32; 3],
                w_hom_dx: [f32; 3],
                w_hom_dy: [f32; 3],
                verts_out: [V; 3],
                blitter: &mut B,
            ) {
                (bounds_clamped_min[1]..bounds_clamped_max[1]).for_each(|y| {
                    let extent = [
                        bounds_clamped_max[0] - bounds_clamped_min[0],
                        bounds_clamped_max[1] - bounds_clamped_min[1],
                    ];
                    let row_range = if extent.iter().product::<usize>() < 128 {
                        // Stupid version
                        [bounds_clamped_min[0], bounds_clamped_max[0]]
                    } else {
                        let [a, b, c] = verts_by_y;

                        // For each of the lines, calculate the point at which our row intersects it
                        let ac = lerp(a[0], c[0], (y as f32 - a[1]) / (c[1] - a[1])); // Longest side
                                                                                      // Then, depending on the half of the triangle we're in, we need to check different lines
                        let row_bounds = if (y as f32) < b[1] {
                            let ab = lerp(a[0], b[0], (y as f32 - a[1]) / (b[1] - a[1]));
                            [ab.min(ac), ab.max(ac)]
                        } else {
                            let bc = lerp(b[0], c[0], (y as f32 - b[1]) / (c[1] - b[1]));
                            [bc.min(ac), bc.max(ac)]
                        };

                        // Now we have screen-space bounds for the row. Clean it up and clamp it to the screen bounds
                        let screen_clamp = |e, b| {
                            if e >= bounds_clamped_min[0] as f32 && e < bounds_clamped_max[0] as f32
                            {
                                e as usize
                            } else {
                                b
                            }
                        };
                        [
                            screen_clamp(row_bounds[0].floor(), bounds_clamped_min[0]),
                            screen_clamp(row_bounds[1].ceil(), bounds_clamped_max[0]),
                        ]
                    };

                    // Find the barycentric weights for the start of this row
                    let mut w_hom = add(
                        add(w_hom_origin, w_hom_dy.map(|e| e * y as f32)),
                        w_hom_dx.map(|e| e * row_range[0] as f32),
                    );

                    (row_range[0]..row_range[1]).for_each(|x| {
                        // Calculate vertex weights to determine vs_out lerping and intersection
                        let w_unbalanced = [w_hom[0], w_hom[1], w_hom[2] - w_hom[0] - w_hom[1]];

                        // Test the weights to determine whether the fragment is inside the triangle
                        if let [true, true, true] = w_unbalanced.map(|e| e >= 0.0) {
                            // Calculate the interpolated z coordinate for the depth target
                            let z = dot(verts_hom.map(|v| v[2]), w_unbalanced);

                            if (NO_VERTS_CLIPPED || coords.passes_z_clip(z))
                                && blitter.test_fragment(x, y, z)
                            {
                                let get_v_data = |x: f32, y: f32| {
                                    let w_hom = add(
                                        add(w_hom_origin, w_hom_dy.map(|e| e * y)),
                                        w_hom_dx.map(|e| e * x),
                                    );

                                    // Calculate vertex weights to determine vs_out lerping and intersection
                                    let w_unbalanced =
                                        [w_hom[0], w_hom[1], w_hom[2] - w_hom[0] - w_hom[1]];
                                    let r = w_hom[2].recip();
                                    let w = w_unbalanced.map(|e| e * r);

                                    V::weighted_sum3(
                                        verts_out[0].clone(),
                                        verts_out[1].clone(),
                                        verts_out[2].clone(),
                                        w[0],
                                        w[1],
                                        w[2],
                                    )
                                };

                                blitter.emit_fragment(x, y, get_v_data, z);
                            }
                        }

                        // Update barycentric weight ready for the next fragment
                        w_hom = add(w_hom, w_hom_dx);
                    });
                });
            }
        });
    }
}

fn cross([a0, a1, a2]: [f32; 3], [b0, b1, b2]: [f32; 3]) -> [f32; 3] {
    [
        a1 * b2 - a2 * b1, // x-component
        a2 * b0 - a0 * b2, // y-component
        a0 * b1 - a1 * b0, // z-component
    ]
}

fn sub([a0, a1, a2]: [f32; 3], [b0, b1, b2]: [f32; 3]) -> [f32; 3] {
    [
        a0 - b0, // x-component
        a1 - b1, // y-component
        a2 - b2, // z-component
    ]
}

fn add([a0, a1, a2]: [f32; 3], [b0, b1, b2]: [f32; 3]) -> [f32; 3] {
    [
        a0 + b0, // x-component
        a1 + b1, // y-component
        a2 + b2, // z-component
    ]
}

fn dot([a0, a1, a2]: [f32; 3], [b0, b1, b2]: [f32; 3]) -> f32 {
    a0 * b0 + a1 * b1 + a2 * b2
}

fn rev<T>([a0, a1, a2]: [T; 3]) -> [T; 3] {
    [a2, a1, a0]
}

fn magnitude_squared([v0, v1, v2]: [f32; 3]) -> f32 {
    v0 * v0 + v1 * v1 + v2 * v2
}

fn matmul(a: [[f32; 3]; 3], b: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
    let mut result = [[0.0; 3]; 3]; // Initialize a 3x3 matrix to store the result

    for i in 0..3 {
        for j in 0..3 {
            result[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
        }
    }

    result
}

fn mat3_mul_vec3(m: [[f32; 3]; 3], v: [f32; 3]) -> [f32; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2], // x-component
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2], // y-component
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2], // z-component
    ]
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}
