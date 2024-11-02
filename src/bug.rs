use std::{
    cell::Ref, error::Error, f64::consts::PI, fmt::Display, ops::Deref, sync::atomic::{AtomicUsize, Ordering}, time::Duration
};

use chromosome::Chromosome;
use rand::Rng;
use rand::RngCore;

#[deprecated]
pub const EAT_FOOD_MAX_PROXIMITY: NoNeg<Float> = noneg_float(20.);

use crate::{
    brain::{self, Brain, VerboseOutput}, chromo_utils::ExtendedChromosome as _, environment::{Environment, EnvironmentRequest, Food}, math::{noneg_float, sign, AbsAsNoNeg as _, Angle, Complex, DeltaAngle, NoNeg}, time_point::TimePoint, utils::{self, Color, Float}
};

use crate::math::Point;

static NEXT_BUG_ID: AtomicUsize = AtomicUsize::new(0);

mod capacity {
    use crate::{
        math::{noneg_float, NoNeg},
        utils::Float,
    };

    static BUG_ENERGY_CAPACITY_PER_SIZE: NoNeg<Float> = noneg_float(100.);
    static BUG_BABY_CHARGE_CAPACITY_PER_SIZE: NoNeg<Float> = noneg_float(1.);
    static BUG_HEAT_CAPACITY_PER_SIZE: NoNeg<Float> = noneg_float(1000.);

    pub fn energy_capacity(size: NoNeg<Float>) -> NoNeg<Float> {
        size * BUG_ENERGY_CAPACITY_PER_SIZE
    }

    pub fn baby_charge_capacity(size: NoNeg<Float>) -> NoNeg<Float> {
        size * BUG_BABY_CHARGE_CAPACITY_PER_SIZE
    }

    pub fn heat_capacity(size: NoNeg<Float>) -> NoNeg<Float> {
        size * BUG_HEAT_CAPACITY_PER_SIZE
    }

    #[cfg(test)]
    mod tests {
        use crate::bug::capacity::{
            BUG_BABY_CHARGE_CAPACITY_PER_SIZE, BUG_ENERGY_CAPACITY_PER_SIZE,
        };

        #[test]
        fn capacity_per_size() {
            // Because if BUG_BABY_CHARGE_CAPACITY_PER_SIZE > BUG_ENERGY_CAPACITY_PER_SIZE then newly born bug will exceed its energy capacity
            assert!(BUG_BABY_CHARGE_CAPACITY_PER_SIZE <= BUG_ENERGY_CAPACITY_PER_SIZE);
        }
    }
}

pub struct BrainLog {
    pub input: brain::Input,
    pub output: brain::Output,
    pub activations: ([Float; 16], [Float; 8], [Float; 8]),
}

pub struct Bug<T> {
    id: usize,
    chromosome: Chromosome<Float>,
    brain: Brain,
    last_brain_log: Option<BrainLog>,
    position: Point<Float>,
    rotation: Angle<Float>,
    size: NoNeg<Float>,
    energy_level: NoNeg<Float>,
    birth_instant: T,
    max_age: Duration,
    color: Color,
    baby_charge_level: NoNeg<Float>,
    heat_level: NoNeg<Float>,
}

#[derive(Debug)]
pub(crate) struct BugEnergyCapacityExceeded {}

impl Display for BugEnergyCapacityExceeded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for BugEnergyCapacityExceeded {}

struct GeneticFeatures {
    brain: Brain,
    max_age: Duration,
    size: NoNeg<Float>,
    color: Color,
}

impl GeneticFeatures {
    fn from_chromosome(chromosome: &Chromosome<Float>) -> GeneticFeatures {
        let brain = Brain::new(&chromosome, 0..208);
        let body_genes = &chromosome.genes[208..256];
        let max_age =
            Duration::from_secs_f64(body_genes[0].abs() * body_genes[1].abs() * 60. * 60. * 24.);
        let size = body_genes[1].abs_as_noneg();
        let color = Color {
            a: 1.,
            r: body_genes[2].rem_euclid(1.),
            g: body_genes[3].rem_euclid(1.),
            b: body_genes[4].rem_euclid(1.),
        };

        GeneticFeatures {
            brain,
            max_age,
            size,
            color,
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
    where T:TimePoint{
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
        capacity::baby_charge_capacity(self.size)
    }

    pub fn heat_level(&self) -> NoNeg<Float> {
        self.heat_level
    }

    pub fn heat_capacity(&self) -> NoNeg<Float> {
        capacity::heat_capacity(self.size)
    }

    pub fn give_birth(
        chromosome: Chromosome<Float>,
        position: Point<Float>,
        rotation: Angle<Float>,
        energy_level: NoNeg<Float>,
        now: T,
    ) -> Result<Self, BugEnergyCapacityExceeded> {
        let features = GeneticFeatures::from_chromosome(&chromosome);

        let result = Self {
            id: NEXT_BUG_ID.fetch_add(1, Ordering::SeqCst),
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
            heat_level: noneg_float(0.),
        };

        if result.energy_level() > result.energy_capacity() {
            Err(BugEnergyCapacityExceeded {})
        } else {
            Ok(result)
        }
    }

    pub fn give_birth_with_max_energy(
        chromosome: Chromosome<Float>,
        position: Point<Float>,
        rotation: Angle<Float>,
        now: T,
    ) -> Self {
        let features = GeneticFeatures::from_chromosome(&chromosome);

        Self {
            id: NEXT_BUG_ID.fetch_add(1, Ordering::SeqCst),
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
            heat_level: noneg_float(0.),
        }
    }

    // Spend all energy to produce as much progeny as possible
    pub fn give_birth_to_twins(
        chromosome: Chromosome<Float>,
        position: Point<Float>,
        rotation: Angle<Float>,
        energy_level: NoNeg<Float>,
        now: T,
    ) -> Vec<Self> where T:Clone{
        let features = GeneticFeatures::from_chromosome(&chromosome);
        let energy_capacity = capacity::energy_capacity(features.size);
        let mut result: Vec<Self> = Default::default();

        let n = (energy_level / energy_capacity).floor();

        for _ in 0..n.unwrap() as usize {
            result.push(Self {
                id: NEXT_BUG_ID.fetch_add(1, Ordering::SeqCst),
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
                heat_level: noneg_float(0.),
            });
        }

        let reminder = NoNeg::wrap(energy_level - energy_capacity * n).unwrap();

        result.push(Self {
            id: NEXT_BUG_ID.fetch_add(1, Ordering::SeqCst),
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
            heat_level: noneg_float(0.),
        });

        result
    }

    fn dst_to_bug(&self, other: &Self) -> Float {
        (self.position - other.position).len()
    }

    fn dst_to_food(&self, other: &Food) -> Float {
        (self.position - other.position()).len()
    }

    /// return in redians
    fn direction_to_bug(&self, other: &Self) -> Angle<Float> {
        (self.position - other.position()).angle()
    }

    /// return in redians
    fn direction_to_food(&self, other: &Food) -> Angle<Float> {
        (self.position - other.position()).angle()
    }

    fn find_nearest_bug<'a>(&self, env: &'a Environment<T>) -> Option<Ref<'a, Self>> {
        env.bugs()
            .min_by(|a, b| self.dst_to_bug(a.deref()).partial_cmp(&self.dst_to_bug(b.deref())).unwrap())
    }

    fn find_nearest_food<'a>(&self, env: &'a Environment<T>) -> Option<&'a Food> {
        env.food().iter().min_by(|a, b| {
            self.dst_to_food(a)
                .partial_cmp(&&self.dst_to_food(b))
                .unwrap()
        })
    }

    fn reproduce_asexually<R: RngCore>(&self, rng: &mut R, now: T) -> Vec<Self> where T:Clone {
        Bug::give_birth_to_twins(
            self.chromosome.mutated_ext(|_| 0.01..0.8, 0.01, rng),
            self.position,
            Angle::from_radians(rng.gen_range(0. ..(PI * 2.))),
            self.baby_charge_capacity(),
            now,
        )
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
    ) -> Vec<EnvironmentRequest<T>> where T: TimePoint + Clone {
        let nearest_food = self.find_nearest_food(env);
        let proximity_to_food = if let Some(nearest_food) = nearest_food {
            self.dst_to_food(nearest_food)
        } else {
            Float::MAX
        };

        let direction_to_nearest_food = if let Some(nearest_food) = nearest_food {
            self.direction_to_food(nearest_food)
        } else {
            Angle::from_radians(0.)
        };

        let age = self.age(env.now().clone());

        let nearest_bug = self.find_nearest_bug(env);
        let proximity_to_bug = if let Some(nearest_bug) = &nearest_bug {
            self.dst_to_bug(&nearest_bug)
        } else {
            Float::MAX
        };

        let direction_to_nearest_bug = if let Some(nearest_bug) = &nearest_bug {
            self.direction_to_bug(&nearest_bug)
        } else {
            Angle::from_radians(0.)
        };

        let color_of_nearest_bug = if let Some(nearest_bug) = &nearest_bug {
            nearest_bug.color.clone()
        } else {
            Color {
                a: 0.,
                r: 0.,
                g: 0.,
                b: 0.,
            }
        };

        let brain_input = brain::Input {
            energy_level: self.energy_level,
            energy_capacity: self.energy_capacity(),
            rotation: self.rotation,
            proximity_to_food,
            direction_to_nearest_food,
            age,
            proximity_to_bug,
            direction_to_nearest_bug,
            color_of_nearest_bug,
            baby_charge_level: self.baby_charge_level,
            baby_charge_capacity: self.baby_charge_capacity(),
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
                        * 0.01
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

        let mut requests: Vec<EnvironmentRequest<T>> = Default::default();

        if let Some(nearest_food) = nearest_food {
            if proximity_to_food
                < (EAT_FOOD_MAX_PROXIMITY * self.size() + nearest_food.radius()).unwrap()
            {
                let eat_rate = noneg_float(0.1) * self.size;
                requests.push(EnvironmentRequest::TransferEnergyFromFoodToBug {
                    food_id: nearest_food.id(),
                    bug_id: self.id,
                    delta_energy: NoNeg::wrap(dt.as_secs_f64()).unwrap() * eat_rate,
                });
            }
        }

        if self.baby_charge_level >= self.baby_charge_capacity() {
            for baby in self.reproduce_asexually(rng, env.now().clone()) {
                requests.push(EnvironmentRequest::GiveBirth(baby));
            }
            self.baby_charge_level =
                NoNeg::wrap(self.baby_charge_level - self.baby_charge_capacity()).unwrap();
        }

        if self.energy_level == noneg_float(0.) || age > noneg_float(1.) {
            requests.push(EnvironmentRequest::Kill { id: self.id });
        }

        requests
    }
}
