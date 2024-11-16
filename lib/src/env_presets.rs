use crate::{
    environment::{FoodSourceCreateInfo, SeededEnvironment},
    food_source::FoodSourceShape,
    math::noneg_float,
};
use rand::SeedableRng;
use rand_pcg::Pcg64;
use std::time::Duration;

pub fn less_food_further_from_center<T: Clone>(
    now: T,
    seed: <Pcg64 as SeedableRng>::Seed,
) -> SeededEnvironment<T> {
    SeededEnvironment::generate(
        now,
        seed,
        // max energy increases by 2^x, and spawn interval increases by 3^x
        vec![
            FoodSourceCreateInfo {
                position: (0., 0.).into(),
                shape: FoodSourceShape::Rect {
                    size: (1000., 1000.).into(),
                },
                energy_range: (0. ..1.).into(),
                spawn_interval: Duration::from_millis((4_u64).pow(0) * 1000),
            },
            FoodSourceCreateInfo {
                position: (0., 0.).into(),
                shape: FoodSourceShape::Rect {
                    size: (2000., 2000.).into(),
                },
                energy_range: (0. ..2.).into(),
                spawn_interval: Duration::from_millis((4_u64).pow(1) * 1000),
            },
            FoodSourceCreateInfo {
                position: (0., 0.).into(),
                shape: FoodSourceShape::Rect {
                    size: (4000., 4000.).into(),
                },
                energy_range: (0. ..4.).into(),
                spawn_interval: Duration::from_millis((4_u64).pow(2) * 1000),
            },
            FoodSourceCreateInfo {
                position: (0., 0.).into(),
                shape: FoodSourceShape::Rect {
                    size: (16000., 16000.).into(),
                },
                energy_range: (0. ..8.).into(),
                spawn_interval: Duration::from_millis((4_u64).pow(3) * 1000),
            },
            FoodSourceCreateInfo {
                position: (0., 0.).into(),
                shape: FoodSourceShape::Rect {
                    size: (32000., 32000.).into(),
                },
                energy_range: (0. ..16.).into(),
                spawn_interval: Duration::from_millis((4_u64).pow(4) * 1000),
            },
            FoodSourceCreateInfo {
                position: (0., 0.).into(),
                shape: FoodSourceShape::Rect {
                    size: (64000., 64000.).into(),
                },
                energy_range: (0. ..32.).into(),
                spawn_interval: Duration::from_millis((4_u64).pow(5) * 1000),
            },
        ],
        -1000. ..1000.,
        -1000. ..1000.,
        0. ..1.,
        32768,
        (0., 0.).into(),
    )
}

pub fn one_big_circle<T: Clone>(
    now: T,
    seed: <Pcg64 as SeedableRng>::Seed,
) -> SeededEnvironment<T> {
    SeededEnvironment::generate(
        now,
        seed,
        vec![FoodSourceCreateInfo {
            position: (0., 0.).into(),
            shape: FoodSourceShape::Circle {
                radius: noneg_float(131072.),
            },
            energy_range: (0. ..128.).into(),
            spawn_interval: Duration::from_millis(5000),
        }],
        -10000. ..10000.,
        -10000. ..10000.,
        0. ..1.,
        262144,
        (0., 0.).into(),
    )
}
