use crate::rasterizer::{Lines, Rasterizer, Triangles};

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

/// A list of triangles.
///
/// `0 1 2 3 4 5` produces triangles `0 1 2` and `3 4 5`.
pub struct TriangleList(());

impl<V> PrimitiveKind<V> for TriangleList {
    type Rasterizer = Triangles;
    type Primitive = [([f32; 4], V); 3];

    #[inline]
    fn collect_primitive<I>(mut iter: I) -> Option<Self::Primitive>
    where
        I: Iterator<Item = ([f32; 4], V)>,
    {
        Some([iter.next()?, iter.next()?, iter.next()?])
    }

    #[inline]
    fn primitive_vertices<O>([a, b, c]: Self::Primitive, mut output: O)
    where
        O: FnMut(([f32; 4], V)),
    {
        output(a);
        output(b);
        output(c);
    }
}

/// A list of triangles, rasterised as lines.
///
/// `0 1 2 3 4 5` produces lines `0 1`, `1 2`, `2 0`, `3 4`, `4 5`, and `5 3`.
pub struct LineTriangleList(());

impl<V: Clone> PrimitiveKind<V> for LineTriangleList {
    type Rasterizer = Lines;
    type Primitive = [([f32; 4], V); 3];

    #[inline]
    fn collect_primitive<I>(mut iter: I) -> Option<Self::Primitive>
    where
        I: Iterator<Item = ([f32; 4], V)>,
    {
        Some([iter.next()?, iter.next()?, iter.next()?])
    }

    #[inline]
    fn primitive_vertices<O>([a, b, c]: Self::Primitive, mut output: O)
    where
        O: FnMut(([f32; 4], V)),
    {
        output(a.clone());
        output(b.clone());

        output(b);
        output(c.clone());

        output(c);
        output(a);
    }
}

/// A list of lines.
///
/// `0 1 2 3 4 5` produces lines `0 1`, `2 3`, and `4 5`.
pub struct LineList(());

impl<V> PrimitiveKind<V> for LineList {
    type Rasterizer = Lines;
    type Primitive = [([f32; 4], V); 2];

    #[inline]
    fn collect_primitive<I>(mut iter: I) -> Option<Self::Primitive>
    where
        I: Iterator<Item = ([f32; 4], V)>,
    {
        Some([iter.next()?, iter.next()?])
    }

    #[inline]
    fn primitive_vertices<O>([a, b]: Self::Primitive, mut output: O)
    where
        O: FnMut(([f32; 4], V)),
    {
        output(a);
        output(b);
    }
}
