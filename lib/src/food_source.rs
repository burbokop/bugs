use std::{f64::consts::PI, time::Duration};

use crate::{
    environment::{EnvironmentRequest, FoodCreateInfo},
    math::{Angle, Complex, NoNeg, Point, Rect, Size},
    range::Range,
    time_point::TimePoint,
    utils::{sample_range_from_range, Float},
};
use rand::Rng;
use rand::RngCore;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum FoodSourceShape {
    Rect { size: Size<Float> },
    Circle { radius: NoNeg<Float> },
}

/// Generates food around itself over time
#[derive(Debug, Serialize, Deserialize)]
pub struct FoodSource<T> {
    position: Point<Float>,
    shape: FoodSourceShape,
    energy_range: Range<Float>,
    spawn_interval: Duration,
    last_food_creation_instant: T,
}

impl<T> FoodSource<T> {
    pub fn position(&self) -> Point<Float> {
        self.position
    }

    pub fn shape(&self) -> &FoodSourceShape {
        &self.shape
    }

    pub(crate) fn new(
        position: Point<Float>,
        shape: FoodSourceShape,
        energy_range: Range<Float>,
        spawn_interval: Duration,
        last_food_creation_instant: T,
    ) -> Self {
        Self {
            position,
            shape,
            energy_range,
            spawn_interval,
            last_food_creation_instant,
        }
    }

    pub(crate) fn proceed<R: RngCore>(&mut self, now: &T, rng: &mut R) -> Vec<EnvironmentRequest>
    where
        T: TimePoint + Clone,
    {
        let mut requests: Vec<EnvironmentRequest> = Default::default();

        let n = now
            .duration_since(&self.last_food_creation_instant)
            .div_duration_f64(self.spawn_interval)
            .floor();

        for _ in 0..(n as usize) {
            match self.shape {
                FoodSourceShape::Rect { size } => {
                    let rect = Rect::from_center(self.position, size);
                    requests.push(EnvironmentRequest::PlaceFood(FoodCreateInfo::generate(
                        rng,
                        sample_range_from_range(rect.x_range()),
                        sample_range_from_range(rect.y_range()),
                        sample_range_from_range(self.energy_range),
                    )));
                }
                FoodSourceShape::Circle { radius } => {
                    requests.push(EnvironmentRequest::PlaceFood(FoodCreateInfo {
                        position: Complex::from_polar(
                            rng.random_range(0. ..radius.unwrap()),
                            Angle::from_radians(rng.random_range(0. ..(PI * 2.))),
                        )
                        .into_cartesian(),
                        energy: NoNeg::wrap(
                            rng.random_range(sample_range_from_range(self.energy_range)),
                        )
                        .unwrap(),
                    }));
                }
            }
        }
        self.last_food_creation_instant += self.spawn_interval.mul_f64(n);
        requests
    }
}
