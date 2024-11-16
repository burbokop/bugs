use super::{Angle, Cos, Point, Sin};
use std::ops::{Add, Mul};

#[derive(Debug, Clone, Copy)]
pub struct Complex<T> {
    real: T,
    imag: T,
}

impl<T> From<(T, T)> for Complex<T> {
    fn from(value: (T, T)) -> Self {
        Self {
            real: value.0,
            imag: value.1,
        }
    }
}

impl<T> Complex<T> {
    pub fn from_cartesian(real: T, imag: T) -> Self {
        Self { real, imag }
    }

    pub fn from_polar(r: T, a: Angle<T>) -> Self
    where
        T: Clone + Cos<Output = T> + Sin<Output = T> + Mul<Output = T>,
    {
        Self {
            real: a.clone().cos() * r.clone(),
            imag: a.clone().sin() * r.clone(),
        }
    }

    pub fn real(&self) -> &T {
        &self.real
    }

    pub fn imag(&self) -> &T {
        &self.imag
    }

    pub fn into_cartesian(self) -> Point<T> {
        (self.real, self.imag).into()
    }
}

impl<T> Add for Complex<T>
where
    T: Add,
{
    type Output = Complex<<T as Add>::Output>;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            real: self.real + rhs.real,
            imag: self.imag + rhs.imag,
        }
    }
}
