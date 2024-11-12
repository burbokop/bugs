use std::ops::Add;

use super::{Abs, Angle, Atan2, Sqr, Sqrt};

#[derive(Debug, Clone, Copy)]
pub struct Vector<T> {
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

impl<T> From<Vector<T>> for (T, T) {
    fn from(value: Vector<T>) -> Self {
        (value.x, value.y)
    }
}

impl<T> Add for Vector<T>
where
    T: Add,
{
    type Output = Vector<<T as Add>::Output>;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T> Vector<T> {
    pub fn len(self) -> <<<T as Sqr>::Output as Add>::Output as Sqrt>::Output
    where
        T: Sqr,
        <T as Sqr>::Output: Add,
        <<T as Sqr>::Output as Add>::Output: Sqrt,
    {
        (self.x.sqr() + self.y.sqr()).sqrt()
    }

    pub fn manhattan_len(self) -> <<T as Abs>::Output as Add>::Output
    where
        T: Abs,
        <T as Abs>::Output: Add,
    {
        self.x.abs() + self.y.abs()
    }

    pub fn len_sqr(self) -> <<T as Sqr>::Output as Add>::Output
    where
        T: Sqr,
        <T as Sqr>::Output: Add,
    {
        self.x.sqr() + self.y.sqr()
    }
}

impl<T> Vector<T>
where
    T: Atan2,
{
    pub fn angle(self) -> Angle<<T as Atan2>::Output> {
        self.y.atan2(self.x)
    }
}

impl<T> Vector<T> {
    pub fn x(&self) -> &T {
        &self.x
    }
    pub fn y(&self) -> &T {
        &self.y
    }
}
