use std::ops::{Add, Mul, Sub};

use crate::{
    math::{Complex, Matrix, One, Point, Vector, Zero},
    utils::Float,
};

pub(crate) struct Camera {
    translation: Matrix<Float>,
    scale: Matrix<Float>,
    rotation: Matrix<Float>,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            translation: Matrix::identity(),
            scale: Matrix::identity(),
            rotation: Matrix::identity(),
        }
    }
}

impl Camera {
    pub(crate) fn set_translation(&mut self, translation: Point<Float>) {
        self.translation = Matrix::translate(translation - Point::origin());
    }

    pub(crate) fn set_scale(&mut self, s: Float) {
        self.scale = Matrix::scale(s, s);
    }

    pub(crate) fn set_rotation(&mut self, rotor: Complex<Float>) {
        self.rotation = Matrix::rotate(rotor);
    }

    pub(crate) fn add_translation(&mut self, vec: Vector<Float>) {
        self.translation = Matrix::translate(self.translation().translation() + vec);
    }

    /**
     * @brief concat_scale_centered
     * @param scale_division - always > 0 (if > 1 - scale in, else if < 1 - scale out, else no scale)
     * @param center - center of scaling.
     * @param prev_center - pass the same value as center if you want only scale, different if translate is intended
     * @return absolute value of scale after concatination
     */
    pub(crate) fn concat_scale_centered(
        &mut self,
        scale_division: Float,
        center: Point<Float>,
        prev_center: Point<Float>,
    ) {
        concat_scale_centered(
            &mut self.scale,
            &mut self.translation,
            scale_division,
            center,
            prev_center,
        );
    }

    pub(crate) fn translation(&self) -> &Matrix<Float> {
        &self.translation
    }

    pub(crate) fn scale(&self) -> &Matrix<Float> {
        &self.scale
    }

    pub(crate) fn rotation(&self) -> &Matrix<Float> {
        &self.rotation
    }

    pub(crate) fn transformation(&self) -> Matrix<Float> {
        &self.translation * &self.scale * &self.rotation
    }
}

fn concat_scale_centered<T>(
    scale_output: &mut Matrix<T>,
    translation_output: &mut Matrix<T>,
    scale_division: T,
    center: Point<T>,
    prev_center: Point<T>,
) where
    T: Clone + Zero + One + Sub<Output = T> + Mul<Output = T> + Add<Output = T>,
{
    let filter_accepts_scale =
        |m: &Matrix<T>| Matrix::scale(m.scale_x().clone(), m.scale_y().clone());
    let filter_accepts_translation = |m: &Matrix<T>| Matrix::translate(m.translation());

    let scale_division_matrix: Matrix<T> = Matrix::scale(scale_division.clone(), scale_division);
    let translation = Matrix::translate(center.clone() - Point::origin());
    let inv_translation = Matrix::translate(Point::origin() - prev_center);

    let output = &translation
        * &scale_division_matrix
        * &inv_translation
        * &*translation_output
        * &*scale_output;

    *scale_output = filter_accepts_scale(&output);
    *translation_output = filter_accepts_translation(&output);
}
