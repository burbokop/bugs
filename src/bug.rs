use std::{
    cell::Ref,
    f64::consts::PI,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

use chromosome::Chromosome;
use complexible::complex_numbers::{Angle, ComplexNumber};
use rand::Rng;
use rand::RngCore;

pub const EAT_FOOD_MAX_PROXIMITY: Float = 20.;

use crate::{
    brain::{self, Brain},
    chromo_utils::ExtendedChromosome as _,
    environment::{Environment, EnvironmentRequest, Food},
    utils::{Color, Float},
};

use crate::math::Point;

static NEXT_BUG_ID: AtomicUsize = AtomicUsize::new(0);

pub(crate) struct Bug {
    id: usize,
    chromosome: Chromosome<Float>,
    brain: Brain,
    position: Point<Float>,
    rotation: Float,
    size: Float,
    energy_level: Float,
    birth_instant: Instant,
    max_age: Duration,
    color: Color,
    baby_charge: Float,
}

impl Bug {
    pub(crate) fn id(&self) -> usize {
        self.id
    }

    pub(crate) fn rotation(&self) -> Float {
        self.rotation
    }

    pub(crate) fn position(&self) -> Point<Float> {
        self.position
    }

    pub(crate) fn size(&self) -> Float {
        self.size
    }

    pub(crate) fn energy_level(&self) -> Float {
        self.energy_level
    }

    pub(crate) fn baby_charge(&self) -> Float {
        self.baby_charge
    }

    pub(crate) fn age(&self) -> Float {
        (Instant::now() - self.birth_instant).div_duration_f64(self.max_age)
    }

    pub(crate) fn color(&self) -> &Color {
        &self.color
    }

    pub(crate) fn give_birth(
        chromosome: Chromosome<Float>,
        position: Point<Float>,
        rotation: Float,
    ) -> Self {
        let brain = Brain::new(&chromosome, 0..208);
        let body_genes = &chromosome.genes[208..256];
        let max_age = Duration::from_secs_f64(body_genes[0] * 60.);
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
            position,
            rotation,
            size: 1.,
            energy_level: 1.,
            birth_instant: Instant::now(),
            max_age,
            color,
            baby_charge: 0.,
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

    fn find_nearest_bug<'a>(&self, env: &'a Environment) -> Option<Ref<'a, Bug>> {
        env.bugs()
            .min_by(|a, b| self.dst_to_bug(a).partial_cmp(&self.dst_to_bug(b)).unwrap())
    }

    fn find_nearest_food<'a>(&self, env: &'a Environment) -> Option<&'a Food> {
        env.food().iter().min_by(|a, b| {
            self.dst_to_food(a)
                .partial_cmp(&&self.dst_to_food(b))
                .unwrap()
        })
    }

    fn reproduce_asexually<R: RngCore>(&self, rng: &mut R) -> Bug {
        Bug::give_birth(
            self.chromosome.mutated_ext(|_| 1., 0.1, rng),
            self.position,
            rng.gen_range(0. ..(PI * 2.)),
        )
    }

    fn reproduce_sexually(&self, partner: &Bug) -> Bug {
        todo!()
    }

    /// return true if food is completely drained
    pub(crate) fn eat(&mut self, food: &mut Food, mut delta_energy: Float) -> bool {
        let mut completely_drained: bool = false;
        if food.energy() < delta_energy {
            delta_energy = food.energy();
            completely_drained = true;
        }
        food.set_energy(food.energy() - delta_energy);
        self.energy_level += delta_energy;
        completely_drained
    }

    pub(crate) fn proceed<R: RngCore>(
        &mut self,
        env: &Environment,
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

        let age = self.age();

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

        let brain_output = self.brain.proceed(brain_input.clone());

        // println!("{:#?}", (self.id, &brain_input, &brain_output));

        {
            let delta_rotation = brain_output.rotation_velocity * 0.01 * dt.as_secs_f64();
            self.rotation = (self.rotation + delta_rotation) % (PI * 2.);
            self.energy_level -= delta_rotation * 0.001;
        }

        {
            let delta_distance = brain_output.velocity * dt.as_secs_f64();
            let new_pos = ComplexNumber::from_cartesian(self.position.x(), self.position.y()).add(
                &ComplexNumber::from_polar(delta_distance, Angle::from_radians(self.rotation)),
            );
            self.position = (new_pos.real(), new_pos.imag()).into();
            self.energy_level -= delta_distance * 0.001;
        }

        {
            let delta_baby_charge = brain_output.baby_charging_rate * 0.01 * dt.as_secs_f64();
            self.baby_charge += delta_baby_charge;
            self.energy_level -= delta_baby_charge;
        }

        let mut requests: Vec<EnvironmentRequest> = Default::default();

        if let Some(nearest_food) = nearest_food {
            if proximity_to_food < EAT_FOOD_MAX_PROXIMITY {
                let eat_rate = 0.1;
                requests.push(EnvironmentRequest::TransferEnergyFromFoodToBug {
                    food_id: nearest_food.id(),
                    bug_id: self.id,
                    delta_energy: dt.as_secs_f64() * eat_rate,
                });
            }
        }

        if self.baby_charge >= 1. {
            // give birth
            requests.push(EnvironmentRequest::GiveBirth(self.reproduce_asexually(rng)));
            self.baby_charge -= 1.;
        }

        if self.energy_level <= 0. || age > 1. {
            requests.push(EnvironmentRequest::Kill { id: self.id });
        }

        requests
    }
}
