use std::{
    error::Error,
    fmt::{Debug, Display},
    ops::{Add, Div, Mul, Sub},
};

use crate::range::{Range, RangeInclusive};

use super::{Clamp, MinusOne, One, Zero};

pub fn map_into_range<T, I, O>(x: T, input: I, output: O) -> T
where
    T: Clone + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T>,
    I: Into<Range<T>>,
    O: Into<Range<T>>,
{
    let input: Range<T> = input.into();
    let output: Range<T> = output.into();
    (x - input.start.clone()) * (output.end - output.start.clone()) / (input.end - input.start)
        + output.start
}

#[derive(Debug)]
pub(crate) struct FitIntoRangeError<T> {
    x: T,
    input: Range<T>,
    output: Range<T>,
}

impl<T> Display for FitIntoRangeError<T> {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl<T> Error for FitIntoRangeError<T> where T: Debug {}

pub(crate) fn fit_into_range<T, I, O>(x: T, input: I, output: O) -> Result<T, FitIntoRangeError<T>>
where
    T: Clone + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T> + PartialOrd,
    I: Into<Range<T>>,
    O: Into<Range<T>>,
{
    let input: Range<T> = input.into();
    if input.contains(&x) {
        Ok(map_into_range(x, input, output))
    } else {
        Err(FitIntoRangeError {
            x,
            input,
            output: output.into(),
        })
    }
}

pub(crate) fn map_into_range_inclusive<T, I, O>(x: T, input: I, output: O) -> T
where
    T: Clone + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T>,
    I: Into<RangeInclusive<T>>,
    O: Into<RangeInclusive<T>>,
{
    let input: RangeInclusive<T> = input.into();
    let output: RangeInclusive<T> = output.into();
    (x - input.start.clone()) * (output.end - output.start.clone()) / (input.end - input.start)
        + output.start
}

#[derive(Debug)]
pub(crate) struct FitIntoRangeInclusiveError<T> {
    x: T,
    input: RangeInclusive<T>,
    output: RangeInclusive<T>,
}

impl<T> Display for FitIntoRangeInclusiveError<T> {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl<T> Error for FitIntoRangeInclusiveError<T> where T: Debug {}

pub(crate) fn fit_into_range_inclusive<T, I, O>(
    x: T,
    input: I,
    output: O,
) -> Result<T, FitIntoRangeInclusiveError<T>>
where
    T: Clone + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T> + PartialOrd,
    I: Into<RangeInclusive<T>>,
    O: Into<RangeInclusive<T>>,
{
    let input: RangeInclusive<T> = input.into();
    if input.contains(&x) {
        Ok(map_into_range_inclusive(x, input, output))
    } else {
        Err(FitIntoRangeInclusiveError {
            x,
            input,
            output: output.into(),
        })
    }
}

pub(crate) fn clamp_into_range<T, I, O>(x: T, input: I, output: O) -> T
where
    T: Clone
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + Div<Output = T>
        + Clamp<Output = T>,
    I: Into<RangeInclusive<T>>,
    O: Into<RangeInclusive<T>>,
{
    let input: RangeInclusive<T> = input.into();
    let output: RangeInclusive<T> = output.into();
    (x.clamp(input.clone()) - input.start.clone()) * (output.end - output.start.clone())
        / (input.end - input.start)
        + output.start
}

pub(crate) fn sign<T>(v: T) -> T
where
    T: Zero + One + MinusOne + PartialOrd,
{
    if v >= T::zero() {
        T::one()
    } else {
        T::minus_one()
    }
}
