use core::range::Range;
use std::ops::{Add, Div, Mul, Sub};

pub(crate) fn map_into_range<T, I, O>(x: T, input: I, output: O) -> T
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
