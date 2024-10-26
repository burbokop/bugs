pub trait Sqr {
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

pub trait Sqrt {
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

pub trait Atan2<Rhs = Self> {
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
