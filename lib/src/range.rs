use std::ops::{Bound, RangeBounds};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Range<Idx> {
    pub start: Idx,
    pub end: Idx,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RangeInclusive<Idx> {
    pub start: Idx,
    pub end: Idx,
}

impl<T> From<std::ops::Range<T>> for Range<T> {
    fn from(value: std::ops::Range<T>) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

impl<T> From<std::ops::RangeInclusive<T>> for RangeInclusive<T> {
    fn from(value: std::ops::RangeInclusive<T>) -> Self {
        let (start, end) = value.into_inner();
        RangeInclusive { start, end }
    }
}

impl<T> RangeBounds<T> for Range<T> {
    fn start_bound(&self) -> Bound<&T> {
        Bound::Included(&self.start)
    }
    fn end_bound(&self) -> Bound<&T> {
        Bound::Excluded(&self.end)
    }
}

impl<T> RangeBounds<T> for Range<&T> {
    fn start_bound(&self) -> Bound<&T> {
        Bound::Included(self.start)
    }
    fn end_bound(&self) -> Bound<&T> {
        Bound::Excluded(self.end)
    }
}

impl<Idx: PartialOrd<Idx>> Range<Idx> {
    #[inline]
    pub fn contains<U>(&self, item: &U) -> bool
    where
        Idx: PartialOrd<U>,
        U: ?Sized + PartialOrd<Idx>,
    {
        <Self as RangeBounds<Idx>>::contains(self, item)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        !(self.start < self.end)
    }
}

impl<Idx: PartialOrd<Idx>> RangeInclusive<Idx> {
    #[inline]
    pub fn contains<U>(&self, item: &U) -> bool
    where
        Idx: PartialOrd<U>,
        U: ?Sized + PartialOrd<Idx>,
    {
        <Self as RangeBounds<Idx>>::contains(self, item)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        !(self.start <= self.end)
    }
}

impl<T> RangeBounds<T> for RangeInclusive<T> {
    fn start_bound(&self) -> Bound<&T> {
        Bound::Included(&self.start)
    }
    fn end_bound(&self) -> Bound<&T> {
        Bound::Included(&self.end)
    }
}

impl<T> RangeBounds<T> for RangeInclusive<&T> {
    fn start_bound(&self) -> Bound<&T> {
        Bound::Included(self.start)
    }
    fn end_bound(&self) -> Bound<&T> {
        Bound::Included(self.end)
    }
}
