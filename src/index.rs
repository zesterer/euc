use core::{
    borrow::Borrow,
    marker::PhantomData,
};

/// A helper type that makes indexed vertex access easier.
pub struct IndexedVertices<'a, Is, Vs, I, V>(Is, Vs, PhantomData<&'a (I, V)>);

impl<'a, Is, Vs, I, V> IndexedVertices<'a, Is, Vs, I, V> {
    pub fn new(is: Is, vs: Vs) -> Self {
        Self(is, vs, PhantomData)
    }
}

impl<'a, Is, Vs, I, V> IntoIterator for IndexedVertices<'a, Is, Vs, I, V>
where
    I: Borrow<usize>,
    Is: IntoIterator<Item = I> + 'a,
    Vs: Borrow<&'a [V]> + 'a,
{
    type Item = &'a V;
    type IntoIter = impl Iterator<Item = &'a V>;

    fn into_iter(self) -> Self::IntoIter {
        let verts = self.1;
        self.0.into_iter().map(move |i| &verts.borrow()[*i.borrow()])
    }
}
