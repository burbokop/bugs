//! Phenotypic signature used to measure how different two individuals are.
//!
//! Raw genes are a poor basis for that: most body genes are unused, several are read through
//! `abs()` or `rem_euclid()`, and many different brain weights produce the same behaviour.
//! So the signature is built from what the genes *express*:
//! - behaviour: brain outputs (only the ones that drive the body) for a fixed set of probe inputs,
//! - body traits: compared on a log scale, so a 10% change means the same for big and small values,
//! - color: compared on a circle, because it is computed with `rem_euclid(1.)`.

use std::sync::LazyLock;

use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use serde::{Deserialize, Serialize};

use crate::{brain::Brain, color::Color, utils::Float};

const PROBE_COUNT: usize = 24;
/// velocity, desired rotation, rotation velocity, baby charging rate.
/// The remaining 4 outputs of the net are not connected to anything.
const USED_OUTPUTS: usize = 4;
const BEHAVIOUR_LEN: usize = PROBE_COUNT * USED_OUTPUTS;
const LOG_TRAITS_LEN: usize = 4;
const COLOR_LEN: usize = 3;
const SIGNATURE_LEN: usize = BEHAVIOUR_LEN + LOG_TRAITS_LEN + COLOR_LEN;

/// Probe situations a bug could encounter, encoded the same way `brain::Input` is.
static PROBES: LazyLock<Vec<[Float; 16]>> = LazyLock::new(|| {
    let mut rng = Pcg64::seed_from_u64(0x5bec1e5);
    (0..PROBE_COUNT)
        .map(|_| {
            let mut p = [0.; 16];
            p[0] = rng.random(); // energy level
            if rng.random_bool(0.5) {
                p[1] = rng.random(); // food distance
                p[2] = rng.random_range(-0.5..0.5); // food direction
                p[3] = rng.random(); // food relative radius
            } else {
                p[1] = 1.;
                p[2] = 0.;
                p[3] = 1.;
            }
            p[4] = rng.random(); // age
            if rng.random_bool(0.5) {
                p[5] = rng.random(); // bug distance
                p[6] = rng.random_range(-0.5..0.5); // bug direction
                p[7] = 1.; // bug color alpha
                p[8] = rng.random(); // bug color r
                p[9] = rng.random(); // bug color g
                p[10] = rng.random(); // bug color b
                p[11] = rng.random(); // bug relative radius
            } else {
                p[5] = 1.;
                p[11] = 1.;
            }
            p[12] = rng.random(); // baby charge level
            p
        })
        .collect()
});

/// Expressed body traits that go into a [`Signature`].
pub(crate) struct Traits<'a> {
    pub size: Float,
    pub max_age_secs: Float,
    pub baby_charge_capacity_per_size: Float,
    pub vision_range: Float,
    pub color: &'a Color,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    values: Vec<Float>,
}

impl Default for Signature {
    fn default() -> Self {
        Self {
            values: vec![0.; SIGNATURE_LEN],
        }
    }
}

impl Signature {
    pub(crate) fn new(brain: &Brain, traits: Traits) -> Self {
        let mut values = Vec::with_capacity(SIGNATURE_LEN);
        for probe in PROBES.iter() {
            let out = brain.proceed_raw(probe);
            values.push(out[0]);
            values.push(out[1]);
            // these two are read through abs()
            values.push(out[2].abs());
            values.push(out[3].abs());
        }
        let log = |x: Float| (x.abs() + 1e-3).ln();
        values.push(log(traits.size));
        values.push(log(traits.max_age_secs));
        values.push(log(traits.baby_charge_capacity_per_size));
        values.push(log(traits.vision_range));
        values.push(traits.color.r);
        values.push(traits.color.g);
        values.push(traits.color.b);
        debug_assert_eq!(values.len(), SIGNATURE_LEN);
        Self { values }
    }

    /// Phenotypic distance. Each of the three parts (behaviour, body traits, color)
    /// contributes its mean squared difference, so none of them dominates just by having more values.
    pub fn distance(&self, other: &Self) -> Float {
        let (a, b) = (&self.values, &other.values);
        let mean_sq = |range: std::ops::Range<usize>, diff: &dyn Fn(Float, Float) -> Float| {
            let n = range.len() as Float;
            range.map(|i| diff(a[i], b[i]).powi(2)).sum::<Float>() / n
        };
        let linear = |x: Float, y: Float| x - y;
        let circular = |x: Float, y: Float| {
            let d = (x - y).abs();
            d.min(1. - d)
        };
        let behaviour = mean_sq(0..BEHAVIOUR_LEN, &linear);
        let traits = mean_sq(BEHAVIOUR_LEN..BEHAVIOUR_LEN + LOG_TRAITS_LEN, &linear);
        let color = mean_sq(BEHAVIOUR_LEN + LOG_TRAITS_LEN..SIGNATURE_LEN, &circular);
        (behaviour + traits + color).sqrt()
    }
}
