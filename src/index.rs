use core::{borrow::Borrow, marker::PhantomData};

/// A helper type that makes indexed vertex access easier.
pub struct IndexedVertices<'a, Is, Vs, I, V> {
    indices: Is,
    verts: Vs,
    phantom: PhantomData<&'a (I, V)>,
}

impl<'a, Is, Vs, I, V> IndexedVertices<'a, Is, Vs, I, V> {
    pub fn new(indices: Is, verts: Vs) -> Self {
        Self {
            indices,
            verts,
            phantom: PhantomData,
        }
    }
}

impl<'a, Is, Vs, I, V> IntoIterator for IndexedVertices<'a, Is, Vs, I, V>
where
    I: Borrow<usize>,
    Is: IntoIterator<Item = I> + 'a,
    Vs: Borrow<&'a [V]> + 'a,
{
    type Item = &'a V;
    type IntoIter = IndexedVerticesIter<'a, Is::IntoIter, Vs, I, V>;

    fn into_iter(self) -> Self::IntoIter {
        IndexedVerticesIter {
            indices: self.indices.into_iter(),
            verts: self.verts,
            phantom: PhantomData,
        }
    }
}

pub struct IndexedVerticesIter<'a, Is: Iterator, Vs, I, V> {
    indices: Is,
    verts: Vs,
    phantom: PhantomData<&'a (I, V)>,
}

impl<'a, Is: Iterator, Vs, I, V> Iterator for IndexedVerticesIter<'a, Is, Vs, I, V>
where
    I: Borrow<usize>,
    Is: Iterator<Item = I> + 'a,
    Vs: Borrow<&'a [V]> + 'a,
{
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        Some(&self.verts.borrow()[*self.indices.next()?.borrow()])
    }
}
