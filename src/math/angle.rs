use serde::{Deserialize, Serialize};

use super::{Abs, Cos, DegToRad, NoNeg, Pi, RadToDeg, RemEuclid, Sin, Two, Zero};
use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, Mul, Rem, Sub},
};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Angle<T>(T);

impl<T> Angle<T> {
    pub fn from_radians(value: T) -> Self {
        Self(value)
    }

    pub(crate) fn from_degrees<U>(value: U) -> Self
    where
        U: DegToRad<Output = T>,
    {
        Self(value.deg_to_rad())
    }

    /// Result in range 0..PI*2
    pub fn radians(self) -> T
    where
        T: Pi + Two + Mul<Output = T> + RemEuclid<Output = T>,
    {
        normalize_radians(self.0)
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
        normalize_radians(self.0) / T::pi() * 180.
    }

    pub(crate) fn cos(self) -> <T as Cos>::Output
    where
        T: Cos,
    {
        self.0.cos()
    }

    pub(crate) fn sin(self) -> <T as Sin>::Output
    where
        T: Sin,
    {
        self.0.sin()
    }

    pub fn signed_distance(self, other: Angle<T>) -> DeltaAngle<T>
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
        let diff = normalize_radians(self.0) - normalize_radians(other.0);
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeltaAngle<T> {
    value: T,
}

impl<T> DeltaAngle<T> {
    pub fn from_radians(value: T) -> Self {
        Self { value }
    }

    pub(crate) fn from_degrees<U>(value: U) -> Self
    where
        U: DegToRad<Output = T>,
    {
        Self {
            value: value.deg_to_rad(),
        }
    }

    /// Result in range -PI*2..PI*2
    pub(crate) fn radians(self) -> T
    where
        T: Pi + Two + Mul<Output = T> + Rem<Output = T>,
    {
        normalize_delta_radians(self.value)
    }

    pub fn degrees(self) -> T
    where
        T: Pi + Two + Mul<Output = T> + Mul<f64, Output = T> + Div<Output = T> + Rem<Output = T>,
    {
        normalize_delta_radians(self.value) / T::pi() * 180.
    }
}

impl<U: Display, T: RadToDeg<Output = U> + Clone> Display for DeltaAngle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{} Δ°", self.value.clone().rad_to_deg()))
    }
}

impl<U: Display, T: RadToDeg<Output = U> + Clone> Display for Angle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}°", self.0.clone().rad_to_deg()))
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
        self.0 += rhs.value
    }
}

impl<T, U> Add<DeltaAngle<U>> for Angle<T>
where
    T: Add<U>,
{
    type Output = Angle<<T as Add<U>>::Output>;

    fn add(self, rhs: DeltaAngle<U>) -> Self::Output {
        Self::Output {
            0: self.0 + rhs.value,
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
    use super::{Angle, DeltaAngle};
    use approx::{abs_diff_eq, assert_abs_diff_eq, AbsDiffEq};
    use std::f64::consts::PI;

    impl<T: AbsDiffEq<Epsilon = T>> AbsDiffEq for DeltaAngle<T> {
        type Epsilon = DeltaAngle<T>;

        fn default_epsilon() -> Self::Epsilon {
            DeltaAngle::from_radians(T::default_epsilon())
        }

        fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
            self.value.abs_diff_eq(&other.value, epsilon.value)
        }
    }

    impl<T: AbsDiffEq<Epsilon = T>> AbsDiffEq for Angle<T> {
        type Epsilon = DeltaAngle<T>;

        fn default_epsilon() -> Self::Epsilon {
            DeltaAngle::from_radians(T::default_epsilon())
        }

        fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
            self.0.abs_diff_eq(&other.0, epsilon.value)
        }
    }

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

    #[test]
    fn signed_distance() {
        assert_abs_diff_eq!(
            Angle::from_degrees(37.8476276094871)
                .signed_distance(Angle::from_degrees(345.1476306413394)),
            DeltaAngle::from_degrees(52.6999969681),
            epsilon = DeltaAngle::from_degrees(1.)
        );
        assert_abs_diff_eq!(
            Angle::from_degrees(169.17968075472993)
                .signed_distance(Angle::from_degrees(138.21107193537364)),
            DeltaAngle::from_degrees(30.96860881935628),
            epsilon = DeltaAngle::from_degrees(1.)
        );

        assert_abs_diff_eq!(
            Angle(0.66056571585).signed_distance(Angle(6.02396256015)),
            DeltaAngle {
                value: 0.91978846288
            },
            epsilon = DeltaAngle::from_degrees(1.)
        );
        assert_abs_diff_eq!(
            Angle(2.95274245664).signed_distance(Angle(2.41223826798)),
            DeltaAngle {
                value: 0.54050418866
            },
            epsilon = DeltaAngle::from_degrees(1.)
        );

        // -172.7996970736128° - 491.62570194155563°
        // assert_abs_diff_eq!(Angle { value: 3.2672616468 } .signed_distance( Angle { value: 2.29730187913 }), DeltaAngle { value: -5.31322553951 }, epsilon = DeltaAngle::from_degrees(1.));

        assert_abs_diff_eq!(
            Angle(-3.01592366038).signed_distance(Angle(8.58048718631)),
            DeltaAngle {
                value: 0.9699597676700003
            },
            epsilon = DeltaAngle::from_degrees(1.)
        );
    }
}
