use std::ops::Sub;

use serde::{Deserialize, Serialize};

use super::{Vector, Zero};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point<T> {
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

impl<T> From<Point<T>> for (T, T) {
    fn from(value: Point<T>) -> Self {
        (value.x, value.y)
    }
}

impl<T> From<Point<T>> for [T; 2] {
    fn from(value: Point<T>) -> Self {
        [value.x, value.y]
    }
}

impl<T: Sub> Sub for Point<T> {
    type Output = Vector<<T as Sub>::Output>;
    fn sub(self, rhs: Self) -> Self::Output {
        (self.x - rhs.x, self.y - rhs.y).into()
    }
}

impl<T> Point<T> {
    pub fn origin() -> Self
    where
        T: Zero,
    {
        Self {
            x: T::zero(),
            y: T::zero(),
        }
    }

    pub fn x(&self) -> &T {
        &self.x
    }
    pub fn y(&self) -> &T {
        &self.y
    }
}

impl Point<f32> {
    pub fn as_f64(self) -> Point<f64> {
        Point {
            x: self.x as f64,
            y: self.y as f64,
        }
    }
}

impl Point<f64> {
    pub fn as_f32(self) -> Point<f32> {
        Point {
            x: self.x as f32,
            y: self.y as f32,
        }
    }
}
