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

pub(crate) trait Atan2<Rhs = Self> {
    type Output;
    fn atan2(self, rhs: Rhs) -> Self::Output;
}

impl Atan2 for f32 {
    type Output = f32;

    fn atan2(self, rhs: Self) -> Self::Output {
        f32::atan2(self, rhs)
    }
}

impl Atan2 for f64 {
    type Output = f64;

    fn atan2(self, rhs: Self) -> Self::Output {
        f64::atan2(self, rhs)
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
