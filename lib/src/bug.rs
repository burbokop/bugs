use std::cell::RefCell;
use std::{cell::Ref, error::Error, f64::consts::PI, fmt::Display, ops::Deref, time::Duration};

use chromosome::Chromosome;
use rand::Rng;
use rand::RngCore;
use serde::{Deserialize, Serialize};

const EAT_FOOD_MAX_PROXIMITY: NoNeg<Float> = noneg_float(20.);

use crate::chunk::Position;
use crate::{
    brain::{self, Brain, VerboseOutput},
    chromo_utils::ExtendedChromosome as _,
    environment::{Environment, EnvironmentRequest, Food},
    math::{noneg_float, sign, AbsAsNoNeg as _, Angle, Complex, DeltaAngle, NoNeg},
    time_point::TimePoint,
    utils::{self, Color, Float},
};

use crate::math::Point;

mod capacity {
    use crate::{
        math::{noneg_float, NoNeg},
        utils::Float,
    };

    static BUG_ENERGY_CAPACITY_PER_SIZE: NoNeg<Float> = noneg_float(100.);
    static BUG_HEAT_CAPACITY_PER_SIZE: NoNeg<Float> = noneg_float(1000.);

    pub fn energy_capacity(size: NoNeg<Float>) -> NoNeg<Float> {
        size * BUG_ENERGY_CAPACITY_PER_SIZE
    }

    pub fn baby_charge_capacity(
        size: NoNeg<Float>,
        baby_charge_capacity_per_size: NoNeg<Float>,
    ) -> NoNeg<Float> {
        size * baby_charge_capacity_per_size
    }

    pub fn heat_capacity(size: NoNeg<Float>) -> NoNeg<Float> {
        size * BUG_HEAT_CAPACITY_PER_SIZE
    }
}

pub struct BrainLog {
    pub input: brain::Input,
    pub output: brain::Output,
    pub activations: ([Float; 16], [Float; 8], [Float; 8]),
}

#[derive(Serialize)]
pub struct Bug<T> {
    id: usize,
    chromosome: Chromosome<Float>,
    #[serde(skip)]
    brain: Brain,
    #[serde(skip)]
    last_brain_log: Option<BrainLog>,
    position: Point<Float>,
    rotation: Angle<Float>,
    #[serde(skip)]
    size: NoNeg<Float>,
    energy_level: NoNeg<Float>,
    birth_instant: T,
    #[serde(skip)]
    max_age: Duration,
    #[serde(skip)]
    color: Color,
    baby_charge_level: NoNeg<Float>,
    #[serde(skip)]
    baby_charge_capacity_per_size: NoNeg<Float>,
    heat_level: NoNeg<Float>,
    #[serde(skip)]
    vision_range: NoNeg<Float>,
}

impl<T> Position for RefCell<Bug<T>> {
    fn position(&self) -> Point<Float> {
        self.borrow().position
    }
}

impl<'a, T> Position for Ref<'a, Bug<T>> {
    fn position(&self) -> Point<Float> {
        self.position
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Bug<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct TmpBug<T> {
            id: usize,
            chromosome: Chromosome<Float>,
            position: Point<Float>,
            rotation: Angle<Float>,
            energy_level: NoNeg<Float>,
            birth_instant: T,
            baby_charge_level: NoNeg<Float>,
            heat_level: NoNeg<Float>,
        }

        let val = TmpBug::deserialize(deserializer)?;
        let features = GeneticFeatures::from_chromosome(&val.chromosome);

        Ok(Self {
            id: val.id,
            chromosome: val.chromosome,
            brain: features.brain,
            last_brain_log: None,
            position: val.position,
            rotation: val.rotation,
            size: features.size,
            energy_level: val.energy_level,
            birth_instant: val.birth_instant,
            max_age: features.max_age,
            color: features.color,
            baby_charge_level: val.baby_charge_level,
            baby_charge_capacity_per_size: features.baby_charge_capacity_per_size,
            heat_level: val.heat_level,
            vision_range: features.vision_range,
        })
    }
}

#[derive(Debug)]
pub(crate) struct BugEnergyCapacityExceeded {}

impl Display for BugEnergyCapacityExceeded {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for BugEnergyCapacityExceeded {}

struct GeneticFeatures {
    brain: Brain,
    max_age: Duration,
    size: NoNeg<Float>,
    color: Color,
    vision_range: NoNeg<Float>,
    baby_charge_capacity_per_size: NoNeg<Float>,
}

impl GeneticFeatures {
    fn from_chromosome(chromosome: &Chromosome<Float>) -> GeneticFeatures {
        let brain = Brain::new(&chromosome, 0..208);
        let body_genes = &chromosome.genes[208..256];
        let max_age =
            Duration::from_secs_f64(body_genes[0].abs() * body_genes[1].abs() * 60. * 60. * 24.);
        let size = body_genes[1].abs_as_noneg();
        let baby_charge_capacity_per_size = body_genes[2].abs_as_noneg();
        let vision_range = body_genes[3].abs_as_noneg() * noneg_float(100.);
        let color = Color {
            a: 1.,
            r: body_genes[4].rem_euclid(1.),
            g: body_genes[5].rem_euclid(1.),
            b: body_genes[6].rem_euclid(1.),
        };

        GeneticFeatures {
            brain,
            max_age,
            size,
            color,
            vision_range,
            baby_charge_capacity_per_size,
        }
    }
}

impl<T> Bug<T> {
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn chromosome(&self) -> &Chromosome<Float> {
        &self.chromosome
    }

    pub(crate) fn chromosome_mut(&mut self) -> &mut Chromosome<Float> {
        &mut self.chromosome
    }

    pub fn brain(&self) -> &Brain {
        &self.brain
    }

    pub fn last_brain_log(&self) -> &Option<BrainLog> {
        &self.last_brain_log
    }

    pub fn rotation(&self) -> Angle<Float> {
        self.rotation
    }

    pub fn position(&self) -> Point<Float> {
        self.position
    }

    pub fn size(&self) -> NoNeg<Float> {
        self.size
    }

    pub fn energy_level(&self) -> NoNeg<Float> {
        self.energy_level
    }

    pub fn energy_capacity(&self) -> NoNeg<Float> {
        capacity::energy_capacity(self.size)
    }

    pub fn baby_charge_level(&self) -> NoNeg<Float> {
        self.baby_charge_level
    }

    pub fn age(&self, now: T) -> NoNeg<Float>
    where
        T: TimePoint,
    {
        NoNeg::wrap(
            now.duration_since(&self.birth_instant)
                .div_duration_f64(self.max_age),
        )
        .unwrap()
    }

    pub fn color(&self) -> &Color {
        &self.color
    }

    pub fn baby_charge_capacity(&self) -> NoNeg<Float> {
        capacity::baby_charge_capacity(self.size, self.baby_charge_capacity_per_size)
    }

    pub fn heat_level(&self) -> NoNeg<Float> {
        self.heat_level
    }

    pub fn heat_capacity(&self) -> NoNeg<Float> {
        capacity::heat_capacity(self.size)
    }

    pub fn vision_range(&self) -> NoNeg<Float> {
        self.vision_range
    }

    pub fn eat_range(&self) -> NoNeg<Float> {
        self.size * EAT_FOOD_MAX_PROXIMITY
    }

    pub(crate) fn give_birth(
        next_id: &mut usize,
        chromosome: Chromosome<Float>,
        position: Point<Float>,
        rotation: Angle<Float>,
        energy_level: NoNeg<Float>,
        now: T,
    ) -> Result<Self, BugEnergyCapacityExceeded> {
        let features = GeneticFeatures::from_chromosome(&chromosome);

        let result = Self {
            id: *next_id,
            chromosome,
            brain: features.brain,
            last_brain_log: None,
            position,
            rotation,
            size: features.size,
            energy_level,
            birth_instant: now,
            max_age: features.max_age,
            color: features.color,
            baby_charge_level: noneg_float(0.),
            baby_charge_capacity_per_size: features.baby_charge_capacity_per_size,
            heat_level: noneg_float(0.),
            vision_range: features.vision_range,
        };

        *next_id += 1;

        if result.energy_level() > result.energy_capacity() {
            Err(BugEnergyCapacityExceeded {})
        } else {
            Ok(result)
        }
    }

    pub(crate) fn give_birth_with_max_energy(
        next_id: &mut usize,
        chromosome: Chromosome<Float>,
        position: Point<Float>,
        rotation: Angle<Float>,
        now: T,
    ) -> Self {
        let features = GeneticFeatures::from_chromosome(&chromosome);
        *next_id += 1;
        Self {
            id: *next_id - 1,
            chromosome,
            brain: features.brain,
            last_brain_log: None,
            position,
            rotation,
            size: features.size,
            energy_level: capacity::energy_capacity(features.size),
            birth_instant: now,
            max_age: features.max_age,
            color: features.color,
            baby_charge_level: noneg_float(0.),
            baby_charge_capacity_per_size: features.baby_charge_capacity_per_size,
            heat_level: noneg_float(0.),
            vision_range: features.vision_range,
        }
    }

    // Spend all energy to produce as much progeny as possible
    pub(crate) fn give_birth_to_twins(
        next_id: &mut usize,
        chromosome: Chromosome<Float>,
        position: Point<Float>,
        rotation: Angle<Float>,
        energy_level: NoNeg<Float>,
        now: T,
    ) -> Vec<Self>
    where
        T: Clone,
    {
        let features = GeneticFeatures::from_chromosome(&chromosome);
        let energy_capacity = capacity::energy_capacity(features.size);
        let mut result: Vec<Self> = Default::default();

        let n = (energy_level / energy_capacity).floor();

        for _ in 0..n.unwrap() as usize {
            result.push(Self {
                id: *next_id,
                chromosome: chromosome.clone(),
                brain: features.brain.clone(),
                last_brain_log: None,
                position,
                rotation,
                size: features.size,
                energy_level: energy_capacity,
                birth_instant: now.clone(),
                max_age: features.max_age,
                color: features.color.clone(),
                baby_charge_level: noneg_float(0.),
                baby_charge_capacity_per_size: features.baby_charge_capacity_per_size,
                heat_level: noneg_float(0.),
                vision_range: features.vision_range,
            });
            *next_id += 1;
        }

        let reminder = NoNeg::wrap(energy_level - energy_capacity * n).unwrap();

        result.push(Self {
            id: *next_id,
            chromosome: chromosome.clone(),
            brain: features.brain.clone(),
            last_brain_log: None,
            position,
            rotation,
            size: features.size,
            energy_level: reminder,
            birth_instant: now.clone(),
            max_age: features.max_age,
            color: features.color.clone(),
            baby_charge_level: noneg_float(0.),
            baby_charge_capacity_per_size: features.baby_charge_capacity_per_size,
            heat_level: noneg_float(0.),
            vision_range: features.vision_range,
        });
        *next_id += 1;

        result
    }

    fn dst_to_bug(&self, other: &Self) -> NoNeg<Float> {
        NoNeg::wrap((self.position - other.position).len()).unwrap()
    }

    fn dst_to_food(&self, other: &Food) -> NoNeg<Float> {
        NoNeg::wrap((self.position - other.position()).len()).unwrap()
    }

    fn manhattan_dst_to_food(&self, other: &Food) -> NoNeg<Float> {
        NoNeg::wrap((self.position - other.position()).manhattan_len()).unwrap()
    }

    /// return in redians
    fn direction_to_bug(&self, other: &Self) -> Angle<Float> {
        (other.position() - self.position).angle()
    }

    /// return in redians
    fn direction_to_food(&self, other: &Food) -> Angle<Float> {
        (other.position() - self.position).angle()
    }

    pub fn find_nearest_bug<'a>(
        &self,
        env: &'a Environment<T>,
    ) -> Option<(Ref<'a, Self>, NoNeg<Float>)> {
        env.find_nearest_bug(self.position, self.vision_range)
    }

    pub fn find_nearest_food<'a>(
        &self,
        env: &'a Environment<T>,
    ) -> Option<(&'a Food, NoNeg<Float>)> {
        env.find_nearest_food(self.position, self.vision_range)
    }

    fn reproduce_asexually<R: RngCore>(&self, rng: &mut R) -> EnvironmentRequest
    where
        T: Clone,
    {
        EnvironmentRequest::GiveBirth {
            chromosome: self.chromosome.mutated_ext(|_| 0.01..0.8, 0.01, rng),
            position: self.position,
            rotation: Angle::from_radians(rng.gen_range(0. ..(PI * 2.))),
            energy_level: self.baby_charge_capacity(),
        }
    }

    fn reproduce_sexually(&self, partner: &Self) -> Self {
        todo!()
    }

    /// return true if food is completely drained
    pub(crate) fn eat(&mut self, food: &mut Food, delta_energy: NoNeg<Float>) -> bool {
        let energy_capacity = self.energy_capacity();
        utils::transfer_energy(
            food.energy_mut(),
            &mut self.energy_level,
            delta_energy,
            energy_capacity,
        )
    }

    pub(crate) fn proceed<R: RngCore>(
        &mut self,
        env: &Environment<T>,
        dt: Duration,
        rng: &mut R,
    ) -> Vec<EnvironmentRequest>
    where
        T: TimePoint + Clone,
    {
        let mut requests: Vec<EnvironmentRequest> = Default::default();
        let age = self.age(env.now().clone());
        if age <= noneg_float(1.) {
            struct NearestFoodInfo<'a> {
                food: &'a Food,
                brain_input: brain::FoodInfo,
            }

            struct NearestBugInfo {
                brain_input: brain::BugInfo,
            }

            let nearest_food = self
                .find_nearest_food(env)
                .map(|(food, dst)| NearestFoodInfo {
                    food,
                    brain_input: brain::FoodInfo {
                        dst,
                        direction: self.direction_to_food(food),
                        relative_radius: food.radius() / self.eat_range(),
                    },
                });

            let nearest_bug = self.find_nearest_bug(env).map(|(bug, dst)| NearestBugInfo {
                brain_input: brain::BugInfo {
                    dst,
                    direction: self.direction_to_bug(&bug),
                    color: bug.color.clone(),
                    relative_radius: bug.eat_range() / self.eat_range(),
                },
            });

            let brain_input = brain::Input {
                energy_level: self.energy_level,
                energy_capacity: self.energy_capacity(),
                rotation: self.rotation,
                age,
                baby_charge_level: self.baby_charge_level,
                baby_charge_capacity: self.baby_charge_capacity(),
                vision_range: self.vision_range,
                nearest_food: nearest_food.as_ref().map(|x| x.brain_input.clone()),
                nearest_bug: nearest_bug.as_ref().map(|x| x.brain_input.clone()),
            };

            let VerboseOutput {
                output: brain_output,
                activations,
            } = self.brain.proceed_verbosely(brain_input.clone());

            self.last_brain_log = Some(BrainLog {
                input: brain_input.clone(),
                output: brain_output.clone(),
                activations,
            });

            {
                let raw_delta = (self.rotation + brain_output.relative_desired_rotation)
                    .signed_distance(self.rotation)
                    .radians();

                if raw_delta.abs() > 0.001 {
                    let delta_rotation = DeltaAngle::from_radians(
                        sign(raw_delta)
                            * raw_delta
                                .abs()
                                .min(brain_output.rotation_velocity.unwrap_radians())
                            * 0.1
                            * dt.as_secs_f64(),
                    );

                    self.rotation += delta_rotation;

                    let delta_energy =
                        delta_rotation.radians().abs_as_noneg() * noneg_float(0.001) * self.size();
                    utils::drain_energy(&mut self.energy_level, delta_energy);
                }
            }

            {
                let delta_distance = brain_output.velocity * dt.as_secs_f64();
                let new_pos = Complex::from_cartesian(*self.position.x(), *self.position.y())
                    + Complex::from_polar(delta_distance, self.rotation);

                self.position = (*new_pos.real(), *new_pos.imag()).into();

                let delta_energy = delta_distance.abs_as_noneg() * noneg_float(0.001) * self.size();
                utils::drain_energy(&mut self.energy_level, delta_energy);
            }

            {
                let delta_energy = brain_output.baby_charging_rate
                    * noneg_float(0.01)
                    * NoNeg::wrap(dt.as_secs_f64()).unwrap();

                let baby_charge_capacity = self.baby_charge_capacity();
                utils::transfer_energy(
                    &mut self.energy_level,
                    &mut self.baby_charge_level,
                    delta_energy,
                    baby_charge_capacity,
                );
            }

            /* heat generation */
            {
                let heat_capacity = self.heat_capacity();
                let delta_energy =
                    noneg_float(0.001) * self.size() * NoNeg::wrap(dt.as_secs_f64()).unwrap();
                utils::transfer_energy(
                    &mut self.energy_level,
                    &mut self.heat_level,
                    delta_energy,
                    heat_capacity,
                );
            }

            if let Some(nearest_food) = nearest_food {
                if nearest_food.brain_input.dst
                    < EAT_FOOD_MAX_PROXIMITY * self.size() + nearest_food.food.radius()
                {
                    let eat_rate = noneg_float(0.1) * self.size;
                    requests.push(EnvironmentRequest::TransferEnergyFromFoodToBug {
                        food_id: nearest_food.food.id(),
                        bug_id: self.id,
                        delta_energy: NoNeg::wrap(dt.as_secs_f64()).unwrap() * eat_rate,
                    });
                }
            }

            if self.baby_charge_level >= self.baby_charge_capacity() {
                requests.push(self.reproduce_asexually(rng));
                self.baby_charge_level =
                    NoNeg::wrap(self.baby_charge_level - self.baby_charge_capacity()).unwrap();
            }

            if self.energy_level == noneg_float(0.) {
                requests.push(EnvironmentRequest::Kill { id: self.id });
            }
        } else {
            requests.push(EnvironmentRequest::Kill { id: self.id });
        }
        requests
    }
}
