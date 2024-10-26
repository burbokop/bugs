use std::ops::Sub;

use super::Vector;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Point<T> {
    x: T,
    y: T,
}

impl<T> From<(T, T)> for Point<T> {
    fn from(value: (T, T)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

impl<T: Sub> Sub for Point<T> {
    type Output = Vector<<T as Sub>::Output>;
    fn sub(self, rhs: Self) -> Self::Output {
        (self.x - rhs.x, self.y - rhs.y).into()
    }
}

impl<T> Point<T> {
    pub fn x(self) -> T {
        self.x
    }
    pub fn y(self) -> T {
        self.y
    }
}
