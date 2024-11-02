use core::range::Range;
use std::time::SystemTime;

use rand::distributions::uniform::{SampleRange, SampleUniform};

use crate::math::NoNeg;

pub type Float = f64;
pub type TimePoint = SystemTime;

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
