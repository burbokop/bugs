use core::range::Range;
use std::{error::Error, fmt::{Debug, Display}, ops::RangeBounds, time::{Duration, SystemTime}};

use rand::distributions::uniform::{SampleRange, SampleUniform};
use serde::de::value;

use crate::math::NoNeg;

pub type Float = f64;

#[derive(Debug, Clone)]
pub struct Color {
    pub a: Float,
    pub r: Float,
    pub g: Float,
    pub b: Float,
}

pub(crate) fn normalize<const SIZE: usize>(v: [Float; SIZE]) -> [Float; SIZE] {
    let max = v.iter().cloned().reduce(Float::max).unwrap();
    v.map(|x| x / max)
}

pub(crate) fn normalize_opt<const SIZE: usize>(v: [Option<Float>; SIZE]) -> [Option<Float>; SIZE] {
    let max = v.iter().cloned().filter_map(|x|x).reduce(Float::max).unwrap();
    v.map(|x| x.map(|x|x/ max) )
}

pub(crate) fn transfer_energy(
    source: &mut NoNeg<Float>,
    dst: &mut NoNeg<Float>,
    mut delta_energy: NoNeg<Float>,
    capacity: NoNeg<Float>,
) -> bool {
    let mut completely_drained: bool = false;
    if *source < delta_energy {
        delta_energy = *source;
        completely_drained = true;
    }

    if (*dst + delta_energy) > capacity {
        delta_energy = NoNeg::wrap(capacity - *dst).unwrap();
    }

    *source = NoNeg::wrap(*source - delta_energy).unwrap();
    *dst += delta_energy;
    completely_drained
}

pub(crate) fn drain_energy(source: &mut NoNeg<Float>, mut delta_energy: NoNeg<Float>) -> bool {
    let mut completely_drained: bool = false;
    if *source < delta_energy {
        delta_energy = *source;
        completely_drained = true;
    }

    *source = NoNeg::wrap(*source - delta_energy).unwrap();
    completely_drained
}

pub(crate) fn sample_range_from_range<T: SampleUniform + PartialOrd>(
    r: Range<T>,
) -> impl SampleRange<T> {
    r.start..r.end
}

pub fn pretty_duration(duration: Duration) -> String {
    if duration > Duration::from_hours(1) {
        return format!("{:.2} h", duration.as_secs_f64() / 60. / 60.);
    } else if duration > Duration::from_mins(1) {
        return format!("{:.2} m", duration.as_secs_f64() / 60.);
    } else if duration > Duration::from_secs(1) {
        return format!("{:.2} s", duration.as_millis_f64() / 1000.);
    } else if duration > Duration::from_millis(1) {
        return format!("{:.2} ms", duration.as_micros() as f64 / 1000.);
    } else if duration > Duration::from_micros(1) {
        return format!("{:.2} µs", duration.as_nanos() as f64 / 1000.);
    } else {
        return format!("{} ns", duration.as_nanos());
    }
}



#[derive(Debug, Clone)]
pub(crate) struct RequiredToBeInRangeError<T, R> {
    value: T,
    range: R
}

impl<T, R> Display for RequiredToBeInRangeError<T, R>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl<T: Debug, R: Debug> Error for RequiredToBeInRangeError<T, R> {

}

pub(crate) trait RequiredToBeInRange: Sized {
    type RangeItem;
    fn required_to_be_in_range<R: RangeBounds<Self::RangeItem>>(self, range: R) -> Result<Self, RequiredToBeInRangeError<Self, R>>;
}


impl RequiredToBeInRange for f64 {
    type RangeItem = f64;
    fn required_to_be_in_range<R: RangeBounds<Self::RangeItem>>(self, range: R) -> Result<Self, RequiredToBeInRangeError<Self, R>> {
        if range.contains(&self) {
            Ok(self)
        } else {
            Err(RequiredToBeInRangeError { value: self, range: range })
        }
    }
}

impl<T: PartialOrd, const SIZE: usize> RequiredToBeInRange for [T; SIZE] {
    type RangeItem = T;
    fn required_to_be_in_range<R: RangeBounds<Self::RangeItem>>(self, range: R) -> Result<Self, RequiredToBeInRangeError<Self, R>> {
        for v in &self {
            if !range.contains(v) {
                return Err(RequiredToBeInRangeError { value: self, range })
            }
        }
        Ok(self)
    }
}
