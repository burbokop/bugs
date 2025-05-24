use crate::utils::Float;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Color {
    pub a: Float,
    pub r: Float,
    pub g: Float,
    pub b: Float,
}

impl Color {
    pub fn with_a(self, a: Float) -> Self {
        Self {
            a,
            r: self.r,
            g: self.g,
            b: self.b,
        }
    }

    pub fn with_r(self, r: Float) -> Self {
        todo!()
    }

    pub fn with_g(self, g: Float) -> Self {
        todo!()
    }

    pub fn with_b(self, b: Float) -> Self {
        todo!()
    }

    pub fn map_a<F: FnOnce(Float) -> Float>(self, f: F) -> Self {
        Self {
            a: f(self.a),
            r: self.r,
            g: self.g,
            b: self.b,
        }
    }

    pub fn map_r<F: FnOnce(Float) -> Float>(self, f: F) -> Self {
        todo!()
    }

    pub fn map_g<F: FnOnce(Float) -> Float>(self, f: F) -> Self {
        todo!()
    }

    pub fn map_b<F: FnOnce(Float) -> Float>(self, f: F) -> Self {
        todo!()
    }

    pub fn from_rgb24(r: u8, g: u8, b: u8) -> Self {
        Self {
            a: 1.,
            r: r as Float / 256.,
            g: g as Float / 256.,
            b: b as Float / 256.,
        }
    }

    /// Converts an HSV color value to RGB. Conversion formula
    /// adapted from http://en.wikipedia.org/wiki/HSV_color_space.
    /// Assumes h, s, and v are contained in the set [0, 1] and
    pub fn from_hsv(a: Float, h: Float, s: Float, v: Float) -> Self {
        let i = (h * 6.).floor() as u64;
        let f = h * 6. - i as Float;
        let p = v * (1. - s);
        let q = v * (1. - f * s);
        let t = v * (1. - (1. - f) * s);

        match i % 6 {
            0 => Self {
                a,
                r: v,
                g: t,
                b: p,
            },
            1 => Self {
                a,
                r: q,
                g: v,
                b: p,
            },
            2 => Self {
                a,
                r: p,
                g: v,
                b: t,
            },
            3 => Self {
                a,
                r: p,
                g: q,
                b: v,
            },
            4 => Self {
                a,
                r: t,
                g: p,
                b: v,
            },
            5 => Self {
                a,
                r: v,
                g: p,
                b: q,
            },
            _ => unreachable!(),
        }
    }
}
