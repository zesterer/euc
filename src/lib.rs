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
//!     type VsOut = ();
//!     type Pixel = [u8; 4];
//!
//!     // Vertex shader
//!     fn vert(&self, pos: &Self::Vertex) -> ([f32; 3], Self::VsOut) {
//!         ([pos[0], pos[1], 0.0], ())
//!     }
//!
//!     // Fragment shader
//!     fn frag(&self, _: &Self::VsOut) -> Self::Pixel {
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

#![feature(min_const_generics, alloc_prelude, array_map)]

extern crate alloc;

/// N-dimensional buffers that may be used as textures and render targets.
pub mod buffer;
/// Math-related functionality.
pub mod math;
/// Pipeline definitions.
pub mod pipeline;
/// Texture and target definitions.
pub mod texture;
/// Rasterization algorithms.
pub mod rasterizer;
/// Texture samplers.
pub mod sampler;

// Reexports
pub use crate::{
    buffer::{Buffer, Buffer1d, Buffer2d, Buffer3d, Buffer4d},
    pipeline::{Pipeline, DepthMode, CoordinateMode},
    texture::{Texture, Target, Empty},
    rasterizer::{Triangles, CullMode},
};
