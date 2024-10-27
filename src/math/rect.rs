use std::ops::{Add, Sub};

use super::Point;

#[derive(Debug, Clone, Copy)]
pub struct Rect<T> {
    x: T,
    y: T,
    w: T,
    h: T,
}

impl<T> Rect<T> {
    pub fn left(&self) -> T
    where
        T: Clone,
    {
        self.x.clone()
    }

    pub fn right(&self) -> T
    where
        T: Add<Output = T> + Clone,
    {
        self.x.clone() + self.w.clone()
    }

    pub fn top(&self) -> T
    where
        T: Clone,
    {
        self.y.clone()
    }

    pub fn bottom(&self) -> T
    where
        T: Add<Output = T> + Clone,
    {
        self.y.clone() + self.h.clone()
    }

    pub fn left_top(&self) -> Point<T>
    where
        T: Clone,
    {
        (self.left(), self.top()).into()
    }
    pub fn right_top(&self) -> Point<T>
    where
        T: Add<Output = T> + Clone,
    {
        (self.right(), self.top()).into()
    }
    pub fn right_bottom(&self) -> Point<T>
    where
        T: Add<Output = T> + Clone,
    {
        (self.right(), self.bottom()).into()
    }
    pub fn left_bottom(&self) -> Point<T>
    where
        T: Add<Output = T> + Clone,
    {
        (self.left(), self.bottom()).into()
    }

    pub fn from_lrtb(left: T, right: T, top: T, bottom: T) -> Self
    where
        T: Sub<Output = T> + Clone,
    {
        Self {
            x: left.clone(),
            y: top.clone(),
            w: right - left,
            h: bottom - top,
        }
    }

    pub fn aabb<I>(iter: I) -> Option<Rect<T>>
    where
        T: Add<Output = T> + Sub<Output = T> + Clone + PartialOrd,
        I: Iterator<Item = Rect<T>>,
    {
        let mut result: Option<(T, T, T, T)> = None;
        for rect in iter {
            let current = (rect.left(), rect.right(), rect.top(), rect.bottom());
            let result = result.get_or_insert(current.clone());
            if current.0 < result.0 {
                result.0 = current.0
            }
            if current.1 > result.1 {
                result.1 = current.1
            }
            if current.2 < result.2 {
                result.2 = current.2
            }
            if current.3 > result.3 {
                result.3 = current.3
            }
        }
        result.map(|a| Rect::from_lrtb(a.0, a.1, a.2, a.3))
    }

    pub fn aabb_from_points<I>(iter: I) -> Option<Rect<T>>
    where
        T: Add<Output = T> + Sub<Output = T> + Clone + PartialOrd,
        I: Iterator<Item = Point<T>>,
    {
        let mut result: Option<(T, T, T, T)> = None;
        for rect in iter {
            let current = (
                rect.x().clone(),
                rect.x().clone(),
                rect.y().clone(),
                rect.y().clone(),
            );
            let result = result.get_or_insert(current.clone());
            if current.0 < result.0 {
                result.0 = current.0
            }
            if current.1 > result.1 {
                result.1 = current.1
            }
            if current.2 < result.2 {
                result.2 = current.2
            }
            if current.3 > result.3 {
                result.3 = current.3
            }
        }
        result.map(|a| Rect::from_lrtb(a.0, a.1, a.2, a.3))
    }
}
