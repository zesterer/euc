pub mod rasterizer;

use rasterizer::Rasterizer;

pub trait Pipeline where Self: Sized {
    type Uniform;
    type Input;
    type VsOut;
    type Output;

    #[inline(always)]
    fn vert(
        uniform: &Self::Uniform,
        input: &Self::Input,
    ) -> ([f32; 3], Self::VsOut);

    #[inline(always)]
    fn frag(
        uniform: &Self::Uniform,
        input: &Self::VsOut,
    ) -> Self::Output;

    // R = rasterizer::Triangles
    fn draw<R: Rasterizer>(
        size: [usize; 2],
        uniform: &Self::Uniform,
        inputs: &[Self::Input],
        target: &mut [Self::Output],
        supplement: &mut <R as Rasterizer>::Supplement,
    ) {
        R::draw::<Self>(size, uniform, inputs, target, supplement)
    }
}
