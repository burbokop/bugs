use core::f32;

use super::Angle;

pub(crate) trait Sqr {
    type Output;
    fn sqr(self) -> Self::Output;
}

impl Sqr for f32 {
    type Output = f32;
    fn sqr(self) -> Self::Output {
        self * self
    }
}

impl Sqr for f64 {
    type Output = f64;
    fn sqr(self) -> Self::Output {
        self * self
    }
}

pub(crate) trait Sqrt {
    type Output;
    fn sqrt(self) -> Self::Output;
}

impl Sqrt for f32 {
    type Output = f32;

    fn sqrt(self) -> Self::Output {
        f32::sqrt(self)
    }
}

impl Sqrt for f64 {
    type Output = f64;

    fn sqrt(self) -> Self::Output {
        f64::sqrt(self)
    }
}

pub(crate) trait Cos {
    type Output;
    fn cos(self) -> Self::Output;
}

impl Cos for f32 {
    type Output = f32;

    fn cos(self) -> Self::Output {
        f32::cos(self)
    }
}

impl Cos for f64 {
    type Output = f64;

    fn cos(self) -> Self::Output {
        f64::cos(self)
    }
}

pub(crate) trait Sin {
    type Output;
    fn sin(self) -> Self::Output;
}

impl Sin for f32 {
    type Output = f32;

    fn sin(self) -> Self::Output {
        f32::sin(self)
    }
}

impl Sin for f64 {
    type Output = f64;

    fn sin(self) -> Self::Output {
        f64::sin(self)
    }
}

pub(crate) trait Atan2<Rhs = Self> {
    type Output;
    fn atan2(self, rhs: Rhs) -> Angle<Self::Output>;
}

impl Atan2 for f32 {
    type Output = f32;

    fn atan2(self, rhs: Self) -> Angle<Self::Output> {
        Angle::from_radians(f32::atan2(self, rhs))
    }
}

impl Atan2 for f64 {
    type Output = f64;

    fn atan2(self, rhs: Self) -> Angle<Self::Output> {
        Angle::from_radians(f64::atan2(self, rhs))
    }
}

pub(crate) trait RemEuclid<Rhs = Self> {
    type Output;
    fn rem_euclid(self, rhs: Rhs) -> Self::Output;
}

impl RemEuclid for f32 {
    type Output = f32;

    fn rem_euclid(self, rhs: Self) -> Self::Output {
        self.rem_euclid(rhs)
    }
}

impl RemEuclid for f64 {
    type Output = f64;

    fn rem_euclid(self, rhs: Self) -> Self::Output {
        self.rem_euclid(rhs)
    }
}

pub(crate) trait Abs {
    type Output;
    fn abs(self) -> Self::Output;
}

impl Abs for f32 {
    type Output = f32;

    fn abs(self) -> Self::Output {
        f32::abs(self)
    }
}

impl Abs for f64 {
    type Output = f64;

    fn abs(self) -> Self::Output {
        f64::abs(self)
    }
}

pub(crate) trait Zero {
    fn zero() -> Self;
}

impl Zero for f32 {
    fn zero() -> Self {
        0.
    }
}

impl Zero for f64 {
    fn zero() -> Self {
        0.
    }
}

pub(crate) trait One {
    fn one() -> Self;
}

impl One for f32 {
    fn one() -> Self {
        1.
    }
}

impl One for f64 {
    fn one() -> Self {
        1.
    }
}

pub(crate) trait MinusOne {
    fn minus_one() -> Self;
}

impl MinusOne for f32 {
    fn minus_one() -> Self {
        -1.
    }
}

impl MinusOne for f64 {
    fn minus_one() -> Self {
        -1.
    }
}

pub(crate) trait Two {
    fn two() -> Self;
}

impl Two for f32 {
    fn two() -> Self {
        2.
    }
}

impl Two for f64 {
    fn two() -> Self {
        2.
    }
}

pub(crate) trait Pi {
    fn pi() -> Self;
}

impl Pi for f32 {
    fn pi() -> Self {
        std::f32::consts::PI
    }
}

impl Pi for f64 {
    fn pi() -> Self {
        std::f64::consts::PI
    }
}

pub(crate) trait IsNeg {
    fn is_neg(&self) -> bool;
}

impl IsNeg for f32 {
    fn is_neg(&self) -> bool {
        *self < 0.
    }
}

impl IsNeg for f64 {
    fn is_neg(&self) -> bool {
        *self < 0.
    }
}
