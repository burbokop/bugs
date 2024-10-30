use std::{
    cell::Ref,
    f64::consts::PI,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

use chromosome::Chromosome;
use complexible::complex_numbers::{Angle, ComplexNumber};
use rand::RngCore;
use rand::{distributions::uniform::SampleRange, Rng};

pub const EAT_FOOD_MAX_PROXIMITY: Float = 20.;

use crate::{
    brain::{self, Brain, VerboseOutput},
    chromo_utils::ExtendedChromosome as _,
    environment::{Environment, EnvironmentRequest, Food},
    math::{noneg_float, AbsAsNoNeg as _, NoNeg},
    utils::{self, Color, Float},
};

use crate::math::Point;

static NEXT_BUG_ID: AtomicUsize = AtomicUsize::new(0);
pub(crate) static BUG_ENERGY_CAPACITY: NoNeg<Float> = noneg_float(10.);

pub(crate) struct BrainLog {
    pub input: brain::Input,
    pub output: brain::Output,
    pub activations: ([Float; 16], [Float; 8], [Float; 8]),
}

pub(crate) struct Bug {
    id: usize,
    chromosome: Chromosome<Float>,
    brain: Brain,
    last_brain_log: Option<BrainLog>,
    position: Point<Float>,
    rotation: Float,
    size: NoNeg<Float>,
    energy_level: NoNeg<Float>,
    birth_instant: Instant,
    max_age: Duration,
    color: Color,
    baby_charge: NoNeg<Float>,
}

impl Bug {
    pub(crate) fn id(&self) -> usize {
        self.id
    }

    pub(crate) fn chromosome(&self) -> &Chromosome<Float> {
        &self.chromosome
    }

    pub(crate) fn brain(&self) -> &Brain {
        &self.brain
    }

    pub(crate) fn last_brain_log(&self) -> &Option<BrainLog> {
        &self.last_brain_log
    }

    pub(crate) fn rotation(&self) -> Float {
        self.rotation
    }

    pub(crate) fn position(&self) -> Point<Float> {
        self.position
    }

    pub(crate) fn size(&self) -> NoNeg<Float> {
        self.size
    }

    pub(crate) fn energy_level(&self) -> NoNeg<Float> {
        self.energy_level
    }

    pub(crate) fn baby_charge(&self) -> NoNeg<Float> {
        self.baby_charge
    }

    pub(crate) fn age(&self, now: Instant) -> NoNeg<Float> {
        NoNeg::wrap((now - self.birth_instant).div_duration_f64(self.max_age)).unwrap()
    }

    pub(crate) fn color(&self) -> &Color {
        &self.color
    }

    pub(crate) fn give_birth(
        chromosome: Chromosome<Float>,
        position: Point<Float>,
        rotation: Float,
        energy_level: NoNeg<Float>,
        now: Instant,
    ) -> Self {
        let brain = Brain::new(&chromosome, 0..208);
        let body_genes = &chromosome.genes[208..256];
        let max_age = Duration::from_secs_f64(body_genes[0].abs() * 60. * 60.);
        let color = Color {
            a: 1.,
            r: body_genes[1],
            g: body_genes[2],
            b: body_genes[3],
        };

        Self {
            id: NEXT_BUG_ID.fetch_add(1, Ordering::SeqCst),
            chromosome,
            brain,
            last_brain_log: None,
            position,
            rotation,
            size: noneg_float(1.),
            energy_level,
            birth_instant: now,
            max_age,
            color,
            baby_charge: noneg_float(0.),
        }
    }

    fn dst_to_bug(&self, other: &Bug) -> Float {
        (self.position - other.position).len()
    }

    fn dst_to_food(&self, other: &Food) -> Float {
        (self.position - other.position()).len()
    }

    /// return in redians
    fn direction_to_bug(&self, other: &Bug) -> Float {
        (self.position - other.position()).angle()
    }

    /// return in redians
    fn direction_to_food(&self, other: &Food) -> Float {
        (self.position - other.position()).angle()
    }

    fn find_nearest_bug<'a, R: SampleRange<Float>>(
        &self,
        env: &'a Environment<R>,
    ) -> Option<Ref<'a, Bug>> {
        env.bugs()
            .min_by(|a, b| self.dst_to_bug(a).partial_cmp(&self.dst_to_bug(b)).unwrap())
    }

    fn find_nearest_food<'a, R: SampleRange<Float>>(
        &self,
        env: &'a Environment<R>,
    ) -> Option<&'a Food> {
        env.food().iter().min_by(|a, b| {
            self.dst_to_food(a)
                .partial_cmp(&&self.dst_to_food(b))
                .unwrap()
        })
    }

    fn reproduce_asexually<R: RngCore>(&self, rng: &mut R, now: Instant) -> Bug {
        Bug::give_birth(
            self.chromosome.mutated_ext(|_| 0.01..10., 0.01, rng),
            self.position,
            rng.gen_range(0. ..(PI * 2.)),
            noneg_float(1.),
            now,
        )
    }

    fn reproduce_sexually(&self, partner: &Bug) -> Bug {
        todo!()
    }

    /// return true if food is completely drained
    pub(crate) fn eat(&mut self, food: &mut Food, delta_energy: NoNeg<Float>) -> bool {
        utils::transfer_energy(food.energy_mut(), &mut self.energy_level, delta_energy, BUG_ENERGY_CAPACITY)
    }

    pub(crate) fn proceed<R: RngCore, Range: SampleRange<Float>>(
        &mut self,
        env: &Environment<Range>,
        dt: Duration,
        rng: &mut R,
    ) -> Vec<EnvironmentRequest> {
        let nearest_food = self.find_nearest_food(env);
        let proximity_to_food = if let Some(nearest_food) = nearest_food {
            self.dst_to_food(nearest_food)
        } else {
            Float::MAX
        };

        let direction_to_nearest_food = if let Some(nearest_food) = nearest_food {
            self.direction_to_food(nearest_food) / PI / 2.
        } else {
            0.
        };

        let age = self.age(env.now().clone());

        let nearest_bug = self.find_nearest_bug(env);
        let proximity_to_bug = if let Some(nearest_bug) = &nearest_bug {
            self.dst_to_bug(&nearest_bug)
        } else {
            Float::MAX
        };

        let direction_to_nearest_bug = if let Some(nearest_bug) = &nearest_bug {
            self.direction_to_bug(&nearest_bug) / PI / 2.
        } else {
            0.
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
            proximity_to_food,
            direction_to_nearest_food,
            age,
            proximity_to_bug,
            direction_to_nearest_bug,
            color_of_nearest_bug,
            baby_charge: self.baby_charge,
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
            let delta_rotation = brain_output.rotation_velocity * 0.01 * dt.as_secs_f64();
            self.rotation = (self.rotation + delta_rotation) % (PI * 2.);

            let delta_energy = delta_rotation.abs_as_noneg() * noneg_float(0.001);
            utils::drain_energy(&mut self.energy_level, delta_energy);
        }

        {
            let delta_distance = brain_output.velocity * dt.as_secs_f64();
            let new_pos =
                ComplexNumber::from_cartesian(*self.position.x(), *self.position.y()).add(
                    &ComplexNumber::from_polar(delta_distance, Angle::from_radians(self.rotation)),
                );
            self.position = (new_pos.real(), new_pos.imag()).into();

            let delta_energy = delta_distance.abs_as_noneg() * noneg_float(0.001);
            utils::drain_energy(&mut self.energy_level, delta_energy);
        }

        {
            let delta_energy = brain_output.baby_charging_rate
                * noneg_float(0.01)
                * NoNeg::wrap(dt.as_secs_f64()).unwrap();

            utils::transfer_energy(&mut self.energy_level, &mut self.baby_charge, delta_energy, noneg_float(1.));
        }

        let mut requests: Vec<EnvironmentRequest> = Default::default();

        if let Some(nearest_food) = nearest_food {
            if proximity_to_food < EAT_FOOD_MAX_PROXIMITY {
                let eat_rate = 0.1;
                requests.push(EnvironmentRequest::TransferEnergyFromFoodToBug {
                    food_id: nearest_food.id(),
                    bug_id: self.id,
                    delta_energy: NoNeg::wrap(dt.as_secs_f64() * eat_rate).unwrap(),
                });
            }
        }

        if self.baby_charge >= noneg_float(1.) {
            // give birth
            requests.push(EnvironmentRequest::GiveBirth(
                self.reproduce_asexually(rng, env.now().clone()),
            ));
            self.baby_charge = NoNeg::wrap(self.baby_charge - noneg_float(1.)).unwrap();
        }

        if self.energy_level == noneg_float(0.) || age > noneg_float(1.) {
            requests.push(EnvironmentRequest::Kill { id: self.id });
        }

        requests
    }
}
