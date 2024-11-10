use std::ops::Div;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Size<T> {
    w: T,
    h: T,
}

impl<T> From<(T, T)> for Size<T> {
    fn from(value: (T, T)) -> Self {
        Self {
            w: value.0,
            h: value.1,
        }
    }
}

impl<T> Size<T> {
    pub fn w(&self) -> &T {
        &self.w
    }
    pub fn h(&self) -> &T {
        &self.h
    }
}

impl<T> Div<T> for Size<T>
where
    T: Div<Output = T> + Clone,
{
    type Output = Size<T>;

    fn div(self, rhs: T) -> Self::Output {
        Self::Output {
            w: self.w / rhs.clone(),
            h: self.h / rhs,
        }
    }
}
