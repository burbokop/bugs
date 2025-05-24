use bugs_lib::{
    color::Color,
    math::{Matrix, Point, Size},
};
use vulkano::padded::Padded;

pub(crate) fn matrix_to_mat3<T>(m: Matrix<T>) -> [Padded<[T; 3], 4>; 3] {
    let [a, b, c, d, e, f, g, h, i] = m.into();
    [[a, b, c].into(), [d, e, f].into(), [g, h, i].into()]
}

pub(crate) fn size_to_vec2<T>(m: Size<T>) -> [T; 2] {
    m.into()
}

pub(crate) fn point_to_vec2<T>(m: Point<T>) -> [T; 2] {
    m.into()
}

pub(crate) fn color_to_vec4(c: Color) -> [f32; 4] {
    [c.r as f32, c.g as f32, c.b as f32, c.a as f32]
}
