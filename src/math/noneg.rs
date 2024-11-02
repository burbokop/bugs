use std::{
    error::Error,
    fmt::{Debug, Display},
    ops::{Add, AddAssign, Div, Mul, Sub},
};

use crate::utils::Float;

use super::{Abs, IsNeg, Pi, Sqrt};

/// Can not store negative numbers
#[derive(Clone, Copy, Debug)]
pub struct NoNeg<T> {
    value: T,
}

impl<T> Display for NoNeg<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Debug)]
pub(crate) struct NegError<T> {
    original_value: T,
}

impl<T> NegError<T> {
    pub(crate) fn original_value(self) -> T {
        self.original_value
    }
}

impl<T> Display for NegError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl<T> Error for NegError<T> where T: Debug {}

// impl<T> TryInto<NoNeg<T>> for T {
//     type Error = NegError;

//     fn try_into(self) -> Result<NoNeg<T>, Self::Error> {
//         todo!()
//     }
// }

impl<T> NoNeg<T> {
    pub(crate) fn wrap(value: T) -> Result<Self, NegError<T>>
    where
        T: IsNeg,
    {
        if value.is_neg() {
            Err(NegError {
                original_value: value,
            })
        } else {
            Ok(Self { value })
        }
    }

    pub fn unwrap(self) -> T {
        self.value
    }

    pub(crate) fn sqrt(self) -> NoNeg<<T as Sqrt>::Output>
    where
        T: Sqrt,
    {
        NoNeg {
            value: self.value.sqrt(),
        }
    }
}

impl<T, U> PartialEq<NoNeg<U>> for NoNeg<T>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &NoNeg<U>) -> bool {
        self.value.eq(&other.value)
    }
}

impl<T> Eq for NoNeg<T>
where
    T: Eq,
{
    fn assert_receiver_is_total_eq(&self) {
        self.value.assert_receiver_is_total_eq()
    }
}

impl<T, U> PartialOrd<NoNeg<U>> for NoNeg<T>
where
    T: PartialOrd<U>,
{
    fn partial_cmp(&self, other: &NoNeg<U>) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl<T, U> Add<NoNeg<U>> for NoNeg<T>
where
    T: Add<U>,
{
    type Output = NoNeg<<T as Add<U>>::Output>;

    fn add(self, rhs: NoNeg<U>) -> Self::Output {
        Self::Output {
            value: self.value + rhs.value,
        }
    }
}

impl<T, U> AddAssign<NoNeg<U>> for NoNeg<T>
where
    T: AddAssign<U>,
{
    fn add_assign(&mut self, rhs: NoNeg<U>) {
        self.value += rhs.value
    }
}

impl<T, U> Sub<NoNeg<U>> for NoNeg<T>
where
    T: Sub<U>,
{
    type Output = <T as Sub<U>>::Output;

    fn sub(self, rhs: NoNeg<U>) -> Self::Output {
        self.value - rhs.value
    }
}

impl<T, U> Mul<NoNeg<U>> for NoNeg<T>
where
    T: Mul<U>,
{
    type Output = NoNeg<<T as Mul<U>>::Output>;

    fn mul(self, rhs: NoNeg<U>) -> Self::Output {
        Self::Output {
            value: self.value * rhs.value,
        }
    }
}

impl<T, U> Div<NoNeg<U>> for NoNeg<T>
where
    T: Div<U>,
{
    type Output = NoNeg<<T as Div<U>>::Output>;

    fn div(self, rhs: NoNeg<U>) -> Self::Output {
        Self::Output {
            value: self.value / rhs.value,
        }
    }
}

pub(crate) trait AbsAsNoNeg
where
    Self: Sized,
{
    type Output;
    fn abs_as_noneg(self) -> NoNeg<Self::Output>;
}

impl<T> AbsAsNoNeg for T
where
    T: Abs,
{
    type Output = <T as Abs>::Output;
    fn abs_as_noneg(self) -> NoNeg<Self::Output> {
        NoNeg { value: self.abs() }
    }
}

impl<T> Pi for NoNeg<T>
where
    T: Pi,
{
    fn pi() -> Self {
        Self { value: T::pi() }
    }
}

pub(crate) const fn noneg_f32(value: f32) -> NoNeg<f32> {
    assert!(value >= 0.);
    NoNeg { value }
}

pub(crate) const fn noneg_f64(value: f64) -> NoNeg<f64> {
    assert!(value >= 0.);
    NoNeg { value }
}

pub const fn noneg_float(value: Float) -> NoNeg<Float> {
    assert!(value >= 0.);
    NoNeg { value }
}
