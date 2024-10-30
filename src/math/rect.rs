use core::range::Range;
use std::{
    ops::{Add, Div, Sub},
    process::Output,
};

use super::{Point, Size, Two};

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

    pub(crate) fn from_center(center: Point<T>, size: Size<T>) -> Self
    where
        T: Clone + Two + Sub<Output = T> + Div<Output = T>,
    {
        Self {
            x: center.x().clone() - size.w().clone() / T::two(),
            y: center.y().clone() - size.h().clone() / T::two(),
            w: size.w().clone(),
            h: size.h().clone(),
        }
    }

    pub(crate) fn x_range(&self) -> Range<T>
    where
        T: Clone + Add<Output = T>,
    {
        Range {
            start: self.x.clone(),
            end: self.w.clone() + self.x.clone(),
        }
    }

    pub(crate) fn y_range(&self) -> Range<T>
    where
        T: Clone + Add<Output = T>,
    {
        Range {
            start: self.y.clone(),
            end: self.h.clone() + self.y.clone(),
        }
    }

    pub(crate) fn contains(&self, other: &Rect<T>) -> bool
    where
        T: PartialOrd + Add<Output = T> + Clone,
    {
        return other.left() >= self.left()
            && other.right() <= self.right()
            && other.top() >= self.top()
            && other.bottom() <= self.bottom();
    }

    pub(crate) fn instersects(&self, other: &Rect<T>) -> bool
    where
        T: PartialOrd + Add<Output = T> + Clone,
    {
        let max = |x, y| if x > y { x } else { y };
        let min = |x, y| if x < y { x } else { y };
        let l = max(self.left(), other.left());
        let r = min(self.right(), other.right());
        let t = max(self.top(), other.top());
        let b = min(self.bottom(), other.bottom());
        return l < r && t < b;
    }
}

impl<T> From<(Point<T>, Size<T>)> for Rect<T> {
    fn from(value: (Point<T>, Size<T>)) -> Self {
        todo!()
    }
}

impl<T> From<(T, T, T, T)> for Rect<T> {
    fn from(value: (T, T, T, T)) -> Self {
        Self {
            x: value.0,
            y: value.1,
            w: value.2,
            h: value.3,
        }
    }
}
