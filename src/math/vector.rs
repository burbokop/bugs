use std::ops::{Add, Mul};

use super::{Atan2, Sqr, Sqrt};

#[derive(Debug, Clone, Copy)]
pub(crate) struct Vector<T> {
    x: T,
    y: T,
}

impl<T> From<(T, T)> for Vector<T> {
    fn from(value: (T, T)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

impl<T> Vector<T>
where
    T: Sqr,
    <T as Sqr>::Output: Add,
    <<T as Sqr>::Output as Add>::Output: Sqrt,
{
    pub fn len(self) -> <<<T as Sqr>::Output as Add>::Output as Sqrt>::Output {
        (self.x.sqr() + self.y.sqr()).sqrt()
    }
}

impl<T> Vector<T>
where
    T: Atan2,
{
    pub fn angle(self) -> <T as Atan2>::Output {
        self.y.atan2(self.x)
    }
}
