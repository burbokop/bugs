use std::{
    fmt::Debug,
    ops::{Add, Div, Mul, Neg, Not, Sub},
};

use super::{Complex, One, Point, Rect, Size, Two, Vector, Zero};

#[derive(Debug, Clone, Copy)]
pub struct Matrix<T>([T; 9]);

mod indices {
    /// horizontal scale factor
    pub const SCALE_X: usize = 0;
    /// horizontal skew factor
    pub const SKEW_X: usize = 1;
    /// horizontal translation
    pub const TRANS_X: usize = 2;
    /// vertical skew factor
    pub const SKEW_Y: usize = 3;
    /// vertical scale factor
    pub const SCALE_Y: usize = 4;
    /// vertical translation
    pub const TRANS_Y: usize = 5;
    /// input x perspective factor
    pub const PERSP0: usize = 6;
    /// input y perspective factor
    pub const PERSP1: usize = 7;
    /// perspective bias
    pub const PERSP2: usize = 8;
}

impl<T> Matrix<T> {
    /**
     * @brief identity - create identity matrix
     *  | 1  0  0 |
     *  | 0  1  0 |
     *  | 0  0  1 |
     * @return identity matrix
     */
    pub fn identity() -> Self
    where
        T: Zero + One,
    {
        Self([
            T::one(),
            T::zero(),
            T::zero(),
            T::zero(),
            T::one(),
            T::zero(),
            T::zero(),
            T::zero(),
            T::one(),
        ])
    }

    /**
     * @brief scale - create scale matrix
     *  | x  0  0 |
     *  | 0  y  0 |
     *  | 0  0  1 |
     * @param x - horizontal scale factor
     * @param y - vertical scale factor
     * @return Matrix with scale
     */
    pub fn scale(x: T, y: T) -> Self
    where
        T: Zero + One,
    {
        Self([
            x,
            T::zero(),
            T::zero(),
            T::zero(),
            y,
            T::zero(),
            T::zero(),
            T::zero(),
            T::one(),
        ])
    }

    /**
     * @brief translate - create translate matrix
     *  | 1  0  x |
     *  | 0  1  y |
     *  | 0  0  1 |
     * @param x - horizontal translation
     * @param y - vertical translation
     * @return Matrix with translation
     */
    pub fn translate(offset: Vector<T>) -> Self
    where
        T: Zero + One + Clone,
    {
        Self([
            T::one(),
            T::zero(),
            offset.x().clone(),
            T::zero(),
            T::one(),
            offset.y().clone(),
            T::zero(),
            T::zero(),
            T::one(),
        ])
    }

    /**
     * @brief rotate - create translate matrix from radians
     *  | cos(θ) -sin(θ)   0 |
     *  | sin(θ)  cos(θ)   0 |
     *  |   0       0      1 |
     * @param rad
     * @return Matrix with rotation
     */
    pub fn rotate(rotor: Complex<T>) -> Self
    where
        T: Zero + One + Clone + Neg<Output = T>,
    {
        Self([
            rotor.real().clone(),
            -rotor.imag().clone(),
            T::zero(),
            rotor.imag().clone(),
            rotor.real().clone(),
            T::zero(),
            T::zero(),
            T::zero(),
            T::one(),
        ])
    }

    pub fn scale_x(&self) -> &T {
        &self.0[indices::SCALE_X]
    }

    pub fn scale_y(&self) -> &T {
        &self.0[indices::SCALE_Y]
    }

    pub fn average_scale(&self) -> T
    where
        T: Two + Clone + Add<Output = T> + Div<Output = T>,
    {
        (self.0[indices::SCALE_X].clone() + self.0[indices::SCALE_Y].clone()) / T::two()
    }

    pub fn translation(&self) -> Vector<T>
    where
        T: Clone,
    {
        (
            self.0[indices::TRANS_X].clone(),
            self.0[indices::TRANS_Y].clone(),
        )
            .into()
    }

    fn rotation(&self) -> Complex<T>
    where
        T: Div<Output = T> + Neg<Output = T> + Clone,
    {
        (
            self.0[indices::SCALE_X].clone() / self.0[indices::SCALE_Y].clone(),
            -self.0[indices::SKEW_X].clone() / self.0[indices::SKEW_Y].clone(),
        )
            .into()
    }

    /*
     * @brief apply_affine_to_point
     * @return vector (x, y) multiplied by matrix
     *               | A B C |
     * @param self - | D E F |
     *               | G H I |
     *              | x |
     * @param rhs - | y |
     *              | 1 |
     *
     *                       |A B C| |x|                               Ax+By+C   Dx+Ey+F
     * @return Matrix * pt = |D E F| |y| = |Ax+By+C Dx+Ey+F Gx+Hy+I| = ------- , -------
     *                       |G H I| |1|                               Gx+Hy+I   Gx+Hy+I
     */
    fn apply_affine_to_point<'a>(&'a self, rhs: &Point<T>) -> Point<T>
    where
        T: One + Clone + Div<Output = T>,
        &'a Matrix<T>: Mul<[T; 3], Output = [T; 3]>,
    {
        let result = self * [rhs.x().clone(), rhs.y().clone(), T::one()];
        (
            result[0].clone() / result[2].clone(),
            result[1].clone() / result[2].clone(),
        )
            .into()
    }

    fn apply_affine_to_rect<'a>(&'a self, rhs: &Rect<T>) -> Rect<T>
    where
        T: One + Add<Output = T> + Sub<Output = T> + Div<Output = T> + Clone + PartialOrd,
        &'a Matrix<T>: Mul<[T; 3], Output = [T; 3]>,
    {
        return Rect::aabb_from_points(
            [
                self.apply_affine_to_point(&rhs.left_top()),
                self.apply_affine_to_point(&rhs.right_top()),
                self.apply_affine_to_point(&rhs.right_bottom()),
                self.apply_affine_to_point(&rhs.left_bottom()),
            ]
            .into_iter(),
        )
        .unwrap();
    }

    /*
     * @brief apply_affine_without_translation
     * @return vector (x, y) multiplied by matrix, treating matrix translation as zero.
     *               | A B 0 |
     * @param self - | D E 0 |
     *               | G H I |
     *              | x |
     * @param rhs - | y |
     *              | 1 |
     *
     *            |A B 0| |x|                            Ax+By     Dx+Ey
     * @return -  |D E 0| |y| = |Ax+By Dx+Ey Gx+Hy+I| = ------- , -------
     *            |G H I| |1|                           Gx+Hy+I   Gx+Hy+I
     */
    fn apply_affine_without_translation<'a>(&'a self, rhs: &Point<T>) -> Point<T>
    where
        T: Zero + One + Div<Output = T> + Clone + Add<Output = T> + Mul<Output = T>,
        &'a Matrix<T>: Mul<[T; 3], Output = [T; 3]>,
    {
        let result = Self([
            self.a().clone(),
            self.b().clone(),
            T::zero(),
            self.d().clone(),
            self.e().clone(),
            T::zero(),
            self.g().clone(),
            self.h().clone(),
            self.i().clone(),
        ]) * &[rhs.x().clone(), rhs.y().clone(), T::one()];
        (
            result[0].clone() / result[2].clone(),
            result[1].clone() / result[2].clone(),
        )
            .into()
    }

    /**
     * @brief apply_only_scale - apply only affine scale to size object
     */
    fn apply_only_scale(&self, rhs: &Size<T>) -> Size<T>
    where
        T: Clone + Mul<Output = T>,
    {
        (
            rhs.w().clone() * self.a().clone(),
            rhs.h().clone() * self.e().clone(),
        )
            .into()
    }

    /**
     * @brief transposed
     * @return returns matrix reflected about the main diagonal
     */
    fn transposed(&self) -> Self
    where
        T: Clone,
    {
        return Self([
            self.a().clone(),
            self.d().clone(),
            self.g().clone(),
            self.b().clone(),
            self.e().clone(),
            self.h().clone(),
            self.c().clone(),
            self.f().clone(),
            self.i().clone(),
        ]);
    }

    fn minor<const I: usize, const J: usize>(&self) -> [T; 4]
    where
        T: Default + Clone,
    {
        let side_len = 3;

        assert!(I < side_len && J < side_len);
        let mut pos: usize = 0;
        let mut result: [T; 4] = Default::default();
        for y in 0..side_len {
            for x in 0..side_len {
                if x != I && y != J {
                    result[pos] = self.0[x + y * side_len].clone();
                    pos += 1;
                }
            }
        }
        return result;
    }

    /**
     * @brief det2x2 - determinant of 2x2 matrix
     * @return
     */
    fn det2x2(data: [T; 4]) -> T
    where
        T: Clone + Mul<Output = T> + Sub<Output = T>,
    {
        return data[0].clone() * data[3].clone() - data[1].clone() * data[2].clone();
    }

    /**
     * @brief det - determinant
     * @return
     */
    fn det(&self) -> T
    where
        T: Clone + Mul<Output = T> + Add<Output = T> + Sub<Output = T>,
    {
        return self.a().clone() * self.e().clone() * self.i().clone()
            + self.b().clone() * self.f().clone() * self.g().clone()
            + self.c().clone() * self.d().clone() * self.h().clone()
            - self.c().clone() * self.e().clone() * self.g().clone()
            - self.b().clone() * self.d().clone() * self.i().clone()
            - self.a().clone() * self.f().clone() * self.h().clone();
    }

    fn a(&self) -> &T {
        return &self.0[0];
    }
    fn b(&self) -> &T {
        return &self.0[1];
    }
    fn c(&self) -> &T {
        return &self.0[2];
    }
    fn d(&self) -> &T {
        return &self.0[3];
    }
    fn e(&self) -> &T {
        return &self.0[4];
    }
    fn f(&self) -> &T {
        return &self.0[5];
    }
    fn g(&self) -> &T {
        return &self.0[6];
    }
    fn h(&self) -> &T {
        return &self.0[7];
    }
    fn i(&self) -> &T {
        return &self.0[8];
    }
}

/// https://d138zd1ktt9iqe.cloudfront.net/media/seo_landing_files/multiplication-of-matrices-of-order-3-x-3-1627879219.png
impl<T> Mul for &Matrix<T>
where
    T: Clone + Mul<Output = T> + Add<Output = T>,
{
    type Output = Matrix<T>;

    fn mul(self, rhs: Self) -> Self::Output {
        let l = &self.0;
        let r = &rhs.0;

        Matrix([
            (l[0].clone() * r[0].clone()
                + l[1].clone() * r[3].clone()
                + l[2].clone() * r[6].clone()),
            (l[0].clone() * r[1].clone()
                + l[1].clone() * r[4].clone()
                + l[2].clone() * r[7].clone()),
            (l[0].clone() * r[2].clone()
                + l[1].clone() * r[5].clone()
                + l[2].clone() * r[8].clone()),
            (l[3].clone() * r[0].clone()
                + l[4].clone() * r[3].clone()
                + l[5].clone() * r[6].clone()),
            (l[3].clone() * r[1].clone()
                + l[4].clone() * r[4].clone()
                + l[5].clone() * r[7].clone()),
            (l[3].clone() * r[2].clone()
                + l[4].clone() * r[5].clone()
                + l[5].clone() * r[8].clone()),
            (l[6].clone() * r[0].clone()
                + l[7].clone() * r[3].clone()
                + l[8].clone() * r[6].clone()),
            (l[6].clone() * r[1].clone()
                + l[7].clone() * r[4].clone()
                + l[8].clone() * r[7].clone()),
            (l[6].clone() * r[2].clone()
                + l[7].clone() * r[5].clone()
                + l[8].clone() * r[8].clone()),
        ])
    }
}

impl<T> Mul<&Matrix<T>> for Matrix<T>
where
    T: Clone + Mul<Output = T> + Add<Output = T>,
{
    type Output = Matrix<T>;

    fn mul(self, rhs: &Matrix<T>) -> Self::Output {
        let l = &self.0;
        let r = &rhs.0;

        Matrix([
            (l[0].clone() * r[0].clone()
                + l[1].clone() * r[3].clone()
                + l[2].clone() * r[6].clone()),
            (l[0].clone() * r[1].clone()
                + l[1].clone() * r[4].clone()
                + l[2].clone() * r[7].clone()),
            (l[0].clone() * r[2].clone()
                + l[1].clone() * r[5].clone()
                + l[2].clone() * r[8].clone()),
            (l[3].clone() * r[0].clone()
                + l[4].clone() * r[3].clone()
                + l[5].clone() * r[6].clone()),
            (l[3].clone() * r[1].clone()
                + l[4].clone() * r[4].clone()
                + l[5].clone() * r[7].clone()),
            (l[3].clone() * r[2].clone()
                + l[4].clone() * r[5].clone()
                + l[5].clone() * r[8].clone()),
            (l[6].clone() * r[0].clone()
                + l[7].clone() * r[3].clone()
                + l[8].clone() * r[6].clone()),
            (l[6].clone() * r[1].clone()
                + l[7].clone() * r[4].clone()
                + l[8].clone() * r[7].clone()),
            (l[6].clone() * r[2].clone()
                + l[7].clone() * r[5].clone()
                + l[8].clone() * r[8].clone()),
        ])
    }
}

impl<T> Mul<&[T; 3]> for &Matrix<T>
where
    T: Clone + Mul<Output = T> + Add<Output = T>,
{
    type Output = [T; 3];

    fn mul(self, rhs: &[T; 3]) -> Self::Output {
        let x = &rhs[0];
        let y = &rhs[1];
        let z = &rhs[2];
        [
            (self.a().clone() * x.clone()
                + self.b().clone() * y.clone()
                + self.c().clone() * z.clone()),
            (self.d().clone() * x.clone()
                + self.e().clone() * y.clone()
                + self.f().clone() * z.clone()),
            (self.g().clone() * x.clone()
                + self.h().clone() * y.clone()
                + self.i().clone() * z.clone()),
        ]
    }
}

impl<T> Mul<&[T; 3]> for Matrix<T>
where
    T: Clone + Mul<Output = T> + Add<Output = T>,
{
    type Output = [T; 3];

    fn mul(self, rhs: &[T; 3]) -> Self::Output {
        let x = &rhs[0];
        let y = &rhs[1];
        let z = &rhs[2];
        [
            (self.a().clone() * x.clone()
                + self.b().clone() * y.clone()
                + self.c().clone() * z.clone()),
            (self.d().clone() * x.clone()
                + self.e().clone() * y.clone()
                + self.f().clone() * z.clone()),
            (self.g().clone() * x.clone()
                + self.h().clone() * y.clone()
                + self.i().clone() * z.clone()),
        ]
    }
}

impl<T> Mul<[T; 3]> for &Matrix<T>
where
    T: Clone + Mul<Output = T> + Add<Output = T>,
{
    type Output = [T; 3];

    fn mul(self, rhs: [T; 3]) -> Self::Output {
        let x = &rhs[0];
        let y = &rhs[1];
        let z = &rhs[2];
        [
            (self.a().clone() * x.clone()
                + self.b().clone() * y.clone()
                + self.c().clone() * z.clone()),
            (self.d().clone() * x.clone()
                + self.e().clone() * y.clone()
                + self.f().clone() * z.clone()),
            (self.g().clone() * x.clone()
                + self.h().clone() * y.clone()
                + self.i().clone() * z.clone()),
        ]
    }
}

impl<'a, T> Mul<&Point<T>> for &'a Matrix<T>
where
    T: One + Clone + Div<Output = T>,
    &'a Matrix<T>: Mul<[T; 3], Output = [T; 3]>,
{
    type Output = Point<T>;

    fn mul(self, rhs: &Point<T>) -> Self::Output {
        self.apply_affine_to_point(rhs)
    }
}

impl<T> Mul<&Size<T>> for &Matrix<T>
where
    T: Clone + Mul<Output = T>,
{
    type Output = Size<T>;

    fn mul(self, rhs: &Size<T>) -> Self::Output {
        self.apply_only_scale(rhs)
    }
}

impl<'a, T> Mul<&Rect<T>> for &'a Matrix<T>
where
    T: One + Add<Output = T> + Sub<Output = T> + Div<Output = T> + Clone + PartialOrd,
    &'a Matrix<T>: Mul<[T; 3], Output = [T; 3]>,
{
    type Output = Rect<T>;

    fn mul(self, rhs: &Rect<T>) -> Self::Output {
        self.apply_affine_to_rect(rhs)
    }
}

/**
 * @brief Not - invert matrix
 * @return inverted matrix if it is invertable else None
 */
impl<T> Not for &Matrix<T>
where
    T: Clone
        + Zero
        + PartialEq
        + Default
        + Mul<Output = T>
        + Add<Output = T>
        + Sub<Output = T>
        + Div<Output = T>
        + Neg<Output = T>,
{
    type Output = Option<Matrix<T>>;

    fn not(self) -> Self::Output {
        let det = self.det();
        if det == T::zero() {
            return None;
        }
        let t = self.transposed();
        Some(Matrix([
            Matrix::det2x2(t.minor::<0, 0>()) / det.clone(),
            -Matrix::det2x2(t.minor::<1, 0>()) / det.clone(),
            Matrix::det2x2(t.minor::<2, 0>()) / det.clone(),
            -Matrix::det2x2(t.minor::<0, 1>()) / det.clone(),
            Matrix::det2x2(t.minor::<1, 1>()) / det.clone(),
            -Matrix::det2x2(t.minor::<2, 1>()) / det.clone(),
            Matrix::det2x2(t.minor::<0, 2>()) / det.clone(),
            -Matrix::det2x2(t.minor::<1, 2>()) / det.clone(),
            Matrix::det2x2(t.minor::<2, 2>()) / det.clone(),
        ]))
    }
}

impl Matrix<f32> {
    pub fn as_f64(self) -> Matrix<f64> {
        Matrix(self.0.map(|x| x as f64))
    }
}

impl Matrix<f64> {
    pub fn as_f32(self) -> Matrix<f32> {
        Matrix(self.0.map(|x| x as f32))
    }
}

impl<T> From<Matrix<T>> for [T; 9] {
    fn from(value: Matrix<T>) -> Self {
        value.0
    }
}
