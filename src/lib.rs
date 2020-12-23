//! [![crates.io](https://img.shields.io/crates/v/euc.svg)](https://crates.io/crates/euc)
//! [![crates.io](https://docs.rs/euc/badge.svg)](https://docs.rs/euc)
//!
//! <img src="misc/example.png" alt="Utah teapot, rendered with Euc" width="100%"/>
//!
//! # Example
//! ```ignore
//! struct Example;
//!
//! impl Pipeline for Example {
//!     type Vertex = [f32; 2];
//!     type VertexData = ();
//!     type Fragment = [u8; 4];
//!
//!     // Vertex shader
//!     fn vert(&self, pos: &Self::Vertex) -> ([f32; 3], Self::VertexData) {
//!         ([pos[0], pos[1], 0.0], ())
//!     }
//!
//!     // Fragment shader
//!     fn frag(&self, _: Self::VertexData) -> Self::Fragment {
//!         [255, 0, 0, 255] // Red
//!     }
//! }
//!
//! fn main() {
//!     let mut color = Buffer2d::new([640, 480], [0; 4]);

//!     Example.draw::<Triangles<(f32,)>, _>(
//!         &[
//!             [-1.0, -1.0],
//!             [ 1.0, -1.0],
//!             [ 0.0,  1.0],
//!         ],
//!         &mut color,
//!         None,
//!     );
//! }
//! ```

#![no_std]

#![feature(min_const_generics, array_map, type_alias_impl_trait)]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

/// N-dimensional buffers that may be used as textures and render targets.
pub mod buffer;
/// Index buffer features.
pub mod index;
/// Math-related functionality.
pub mod math;
/// Pipeline definitions.
pub mod pipeline;
/// Primitive definitions.
pub mod primitives;
/// Rasterization algorithms.
pub mod rasterizer;
/// Texture samplers.
pub mod sampler;
/// Texture and target definitions.
pub mod texture;

// Reexports
pub use crate::{
    buffer::{Buffer, Buffer1d, Buffer2d, Buffer3d, Buffer4d},
    pipeline::{Pipeline, DepthMode, PixelMode, CoordinateMode, Handedness, YAxisDirection},
    primitives::TriangleList,
    texture::{Texture, Target, Empty},
    rasterizer::CullMode,
    sampler::{Sampler, Nearest},
    index::IndexedVertices,
};
