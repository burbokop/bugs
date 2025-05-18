use std::{
    error::Error,
    fmt::{Debug, Display},
    ops::{Add, AddAssign, Div, Mul, Sub},
};

use serde::{Deserialize, Serialize};

use crate::utils::Float;

use super::{Abs, Floor, IsNeg, Pi, Sqrt, Zero};

/// Can not store negative numbers
#[derive(Clone, Copy, Debug)]
pub struct NoNeg<T> {
    value: T,
}

impl<T: Serialize> Serialize for NoNeg<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.value.serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de> + IsNeg + Debug> Deserialize<'de> for NoNeg<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = T::deserialize(deserializer)?;
        if value.is_neg() {
            Err(serde::de::Error::custom(&format!(
                "Can not deserialize {:?} as NoNeg because it is negative.",
                value
            )))
        } else {
            Ok(Self { value })
        }
    }
}

impl<T: Display> Display for NoNeg<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
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
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

    pub(crate) fn floor(self) -> NoNeg<<T as Floor>::Output>
    where
        T: Floor,
    {
        NoNeg {
            value: self.value.floor(),
        }
    }

    pub(crate) fn limited_sub<U>(self, rhs: NoNeg<U>) -> NoNeg<<T as Sub<U>>::Output>
    where
        T: Sub<U>,
        <T as Sub<U>>::Output: IsNeg,
        NoNeg<<T as Sub<U>>::Output>: Zero,
    {
        NoNeg::wrap(self.value - rhs.value).unwrap_or(Zero::zero())
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

impl<T: Zero> Zero for NoNeg<T> {
    fn zero() -> Self {
        Self { value: T::zero() }
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

#[cfg(test)]
mod tests {
    use crate::math::noneg_float;

    #[test]
    fn limited_sub() {
        assert_eq!(
            noneg_float(0.5).limited_sub(noneg_float(1.0)),
            noneg_float(0.0)
        );
        assert_eq!(
            noneg_float(1.0).limited_sub(noneg_float(0.5)),
            noneg_float(0.5)
        );
    }
}
