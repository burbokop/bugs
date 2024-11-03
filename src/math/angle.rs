use std::ops::{Add, AddAssign, Div, Mul, Rem, Sub};

use super::{Abs, Cos, NoNeg, Pi, RemEuclid, Sin, Two, Zero};

#[derive(Debug, Clone, Copy)]
pub struct Angle<T> {
    value: T,
}

impl<T> Angle<T> {
    pub fn from_radians(value: T) -> Self {
        Self { value }
    }

    pub(crate) fn from_degrees(value: T) -> Self {
        todo!()
    }

    /// Result in range 0..PI*2
    pub fn radians(self) -> T
    where
        T: Pi + Two + Mul<Output = T> + RemEuclid<Output = T>,
    {
        normalize_radians(self.value)
    }

    pub fn degrees(self) -> T
    where
        T: Pi
            + Two
            + Mul<Output = T>
            + Mul<f64, Output = T>
            + Div<Output = T>
            + RemEuclid<Output = T>,
    {
        normalize_radians(self.value) / T::pi() * 180.
    }

    pub(crate) fn cos(self) -> <T as Cos>::Output
    where
        T: Cos,
    {
        self.value.cos()
    }

    pub(crate) fn sin(self) -> <T as Sin>::Output
    where
        T: Sin,
    {
        self.value.sin()
    }

    pub(crate) fn signed_distance(self, other: Angle<T>) -> DeltaAngle<T>
    where
        T: Clone
            + Pi
            + Two
            + Zero
            + Mul<Output = T>
            + RemEuclid<Output = T>
            + Add<Output = T>
            + Sub<Output = T>
            + Abs<Output = T>
            + PartialOrd,
    {
        let max = T::pi() * T::two();
        let diff = self.value - other.value;
        DeltaAngle {
            value: if diff.clone().abs() > T::pi() {
                if diff >= T::zero() {
                    diff - max
                } else {
                    diff + max
                }
            } else {
                diff
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DeltaAngle<T> {
    value: T,
}

impl<T> DeltaAngle<T> {
    pub fn from_radians(value: T) -> Self {
        Self { value }
    }

    pub(crate) fn from_degrees(value: T) -> Self {
        todo!()
    }

    /// Result in range -PI*2..PI*2
    pub(crate) fn radians(self) -> T
    where
        T: Pi + Two + Mul<Output = T> + Rem<Output = T>,
    {
        normalize_delta_radians(self.value)
    }

    pub(crate) fn degrees(self) -> T
    where
        T: Pi + Two + Mul<Output = T> + Mul<f64, Output = T> + Div<Output = T> + Rem<Output = T>,
    {
        normalize_delta_radians(self.value) / T::pi() * 180.
    }
}

impl<T> DeltaAngle<NoNeg<T>> {
    pub(crate) fn unwrap_radians(self) -> T
    where
        T: Pi + Two + Mul<Output = T> + Rem<Output = T>,
    {
        normalize_delta_radians(self.value.unwrap())
    }

    pub fn unwrap_degrees(self) -> T
    where
        T: Pi + Two + Mul<Output = T> + Mul<f64, Output = T> + Div<Output = T> + Rem<Output = T>,
    {
        normalize_delta_radians(self.value.unwrap()) / T::pi() * 180.
    }
}

impl<T, U> AddAssign<DeltaAngle<U>> for Angle<T>
where
    T: AddAssign<U>,
{
    fn add_assign(&mut self, rhs: DeltaAngle<U>) {
        self.value += rhs.value
    }
}

impl<T, U> Add<DeltaAngle<U>> for Angle<T>
where
    T: Add<U>,
{
    type Output = Angle<<T as Add<U>>::Output>;

    fn add(self, rhs: DeltaAngle<U>) -> Self::Output {
        Self::Output {
            value: self.value + rhs.value,
        }
    }
}

fn normalize_radians<T>(value: T) -> T
where
    T: Pi + Two + Mul<Output = T> + RemEuclid<Output = T>,
{
    value.rem_euclid(T::pi() * T::two())
}

fn normalize_delta_radians<T>(value: T) -> T
where
    T: Pi + Two + Mul<Output = T> + Rem<Output = T>,
{
    value.rem(T::pi() * T::two())
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    #[test]
    fn normalize() {
        assert_eq!(super::normalize_radians(0.5 * PI), 0.5 * PI);
        assert_eq!(super::normalize_radians(-0.5 * PI), 1.5 * PI);
        assert_eq!(super::normalize_radians(2.5 * PI), 0.5 * PI);
        assert_eq!(super::normalize_radians(-2.5 * PI), 1.5 * PI);
    }

    #[test]
    fn normalize_delta() {
        assert_eq!(super::normalize_delta_radians(0.5 * PI), 0.5 * PI);
        assert_eq!(super::normalize_delta_radians(-0.5 * PI), -0.5 * PI);
        assert_eq!(super::normalize_delta_radians(2.5 * PI), 0.5 * PI);
        assert_eq!(super::normalize_delta_radians(-2.5 * PI), -0.5 * PI);
        assert_eq!(super::normalize_delta_radians(1.5 * PI), 1.5 * PI);
        assert_eq!(super::normalize_delta_radians(-1.5 * PI), -1.5 * PI);
    }
}
