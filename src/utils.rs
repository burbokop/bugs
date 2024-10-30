use core::range::Range;
use std::ops::{Add, Div, Mul, Sub};

use crate::math::NoNeg;

pub type Float = f64;

#[derive(Debug, Clone)]
pub(crate) struct Color {
    pub(crate) a: Float,
    pub(crate) r: Float,
    pub(crate) g: Float,
    pub(crate) b: Float,
}

pub(crate) fn color_to_slint_rgba_color(c: &Color) -> slint::RgbaColor<f32> {
    slint::RgbaColor {
        alpha: c.a as f32,
        red: c.r as f32,
        green: c.g as f32,
        blue: c.b as f32,
    }
}

pub(crate) fn color_to_sdl2_rgba_color(c: &Color) -> sdl2::pixels::Color {
    sdl2::pixels::Color::RGBA(
        (c.r * 255.) as u8,
        (c.g * 255.) as u8,
        (c.b * 255.) as u8,
        (c.a * 255.) as u8,
    )
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
