use crate::rasterizer::{Rasterizer, Triangles};

pub trait PrimitiveKind<V> {
    type Rasterizer: Rasterizer;
    type Primitive;

    /// Collect a single primitive from an iterator of vertices.
    fn collect_primitive<I>(iter: I) -> Option<Self::Primitive>
    where
        I: Iterator<Item = ([f32; 4], V)>;

    /// Emit a primitive as a series of vertices 'as-is'.
    fn primitive_vertices<O>(primitive: Self::Primitive, output: O)
    where
        O: FnMut(([f32; 4], V));
}

pub struct TriangleList;

impl<V> PrimitiveKind<V> for TriangleList {
    type Rasterizer = Triangles;
    type Primitive = [([f32; 4], V); 3];

    fn collect_primitive<I>(mut iter: I) -> Option<Self::Primitive>
    where
        I: Iterator<Item = ([f32; 4], V)>
    {
        Some([iter.next()?, iter.next()?, iter.next()?])
    }

    fn primitive_vertices<O>([a, b, c]: Self::Primitive, mut output: O)
    where
        O: FnMut(([f32; 4], V))
    {
        output(a);
        output(b);
        output(c);
    }
}
