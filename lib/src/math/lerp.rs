use std::ops::{Add, Mul};

use crate::utils::Float;

pub(crate) fn lerp<T>(a: T, b: T, t: Float) -> T
where
    T: Mul<Float, Output = T>,
    T: Add<Output = T>,
{
    a * (1. - t) + b * t
}

pub struct LerpIntegrator<T> {
    t: Float,
    prev: Option<T>,
}

impl<T> LerpIntegrator<T> {
    pub fn new(t: Float) -> Self {
        Self { t, prev: None }
    }

    pub fn proceed(&mut self, v: T) -> &T
    where
        T: Mul<Float, Output = T>,
        T: Add<Output = T>,
        T: Clone,
    {
        let prev = self.prev.get_or_insert(v.clone());
        *prev = lerp(prev.clone(), v, self.t);
        prev
    }
}
