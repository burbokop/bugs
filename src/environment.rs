use std::{
    cell::{Ref, RefCell},
    f64::consts::PI,
    time::Duration,
};

use crate::{
    bug::Bug,
    math::{noneg_float, Angle, NoNeg, Point, Rect, Size},
    range::Range,
    time_point::TimePoint,
    utils::{sample_range_from_range, Float},
};
use chromosome::Chromosome;
use rand::Rng;
use rand::{distributions::uniform::SampleRange, RngCore};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Food {
    id: usize,
    position: Point<Float>,
    energy: NoNeg<Float>,
}

/// Generates food around itself over time
#[derive(Serialize, Deserialize)]
pub struct FoodSource<T> {
    position: Point<Float>,
    size: Size<Float>,
    energy_range: Range<Float>,
    spawn_interval: Duration,
    last_food_creation_instant: T,
}

pub struct FoodSourceCreateInfo {
    pub position: Point<Float>,
    pub size: Size<Float>,
    pub energy_range: Range<Float>,
    pub spawn_interval: Duration,
}

impl FoodSourceCreateInfo {
    fn create<T>(self, last_food_creation_instant: T) -> FoodSource<T> {
        FoodSource {
            position: self.position,
            size: self.size,
            energy_range: self.energy_range,
            spawn_interval: self.spawn_interval,
            last_food_creation_instant,
        }
    }
}

impl<T> FoodSource<T> {
    pub fn position(&self) -> Point<Float> {
        self.position
    }

    pub fn size(&self) -> Size<Float> {
        self.size
    }
}

impl Food {
    pub(crate) fn id(&self) -> usize {
        self.id
    }

    pub fn position(&self) -> Point<Float> {
        self.position
    }

    pub(crate) fn energy(&self) -> NoNeg<Float> {
        self.energy
    }

    pub fn radius(&self) -> NoNeg<Float> {
        (self.energy / noneg_float(PI)).sqrt() * noneg_float(10.)
    }

    pub(crate) fn energy_mut(&mut self) -> &mut NoNeg<Float> {
        &mut self.energy
    }

    pub(crate) fn new(next_id: &mut usize, position: Point<Float>, energy: NoNeg<Float>) -> Self {
        *next_id += 1;
        Self {
            id: *next_id - 1,
            position,
            energy,
        }
    }

    pub(crate) fn generate<R: RngCore, RR: SampleRange<Float>>(
        next_id: &mut usize,
        rng: &mut R,
        x_range: RR,
        y_range: RR,
        e_range: RR,
    ) -> Self {
        Self::new(
            next_id,
            (rng.gen_range(x_range), rng.gen_range(y_range)).into(),
            NoNeg::wrap(rng.gen_range(e_range)).unwrap(),
        )
    }

    pub fn generate_vec<R: RngCore, RR: SampleRange<Float> + Clone>(
        next_id: &mut usize,
        rng: &mut R,
        x_range: RR,
        y_range: RR,
        e_range: RR,
        count: usize,
    ) -> Vec<Self> {
        (0..count)
            .map(|_| {
                Self::generate(
                    next_id,
                    rng,
                    x_range.clone(),
                    y_range.clone(),
                    e_range.clone(),
                )
            })
            .collect()
    }
}

pub struct FoodCreateInfo {
    pub position: Point<Float>,
    pub energy: NoNeg<Float>,
}

impl FoodCreateInfo {
    pub(crate) fn generate<R: RngCore, RR: SampleRange<Float>>(
        rng: &mut R,
        x_range: RR,
        y_range: RR,
        e_range: RR,
    ) -> Self {
        Self {
            position: (rng.gen_range(x_range), rng.gen_range(y_range)).into(),
            energy: NoNeg::wrap(rng.gen_range(e_range)).unwrap(),
        }
    }

    pub fn generate_vec<R: RngCore, RR: SampleRange<Float> + Clone>(
        rng: &mut R,
        x_range: RR,
        y_range: RR,
        e_range: RR,
        count: usize,
    ) -> Vec<Self> {
        (0..count)
            .map(|_| Self::generate(rng, x_range.clone(), y_range.clone(), e_range.clone()))
            .collect()
    }
}

pub struct BugCreateInfo {
    pub chromosome: Chromosome<Float>,
    pub position: Point<Float>,
    pub rotation: Angle<Float>,
}

pub(crate) enum EnvironmentRequest {
    Kill {
        id: usize,
    },
    GiveBirth {
        chromosome: Chromosome<Float>,
        position: Point<Float>,
        rotation: Angle<Float>,
        energy_level: NoNeg<Float>,
    },
    TransferEnergyFromFoodToBug {
        food_id: usize,
        bug_id: usize,
        delta_energy: NoNeg<Float>,
    },
}

#[derive(Serialize, Deserialize)]
pub struct Environment<T> {
    food: Vec<Food>,
    food_sources: Vec<FoodSource<T>>,
    bugs: Vec<RefCell<Bug<T>>>,
    creation_time: T,
    now: T,
    next_food_id: usize,
    next_bug_id: usize,
}

impl<T> Environment<T> {
    pub fn new(
        now: T,
        food: Vec<FoodCreateInfo>,
        food_sources: Vec<FoodSourceCreateInfo>,
        bugs: Vec<BugCreateInfo>,
    ) -> Self
    where
        T: Clone,
    {
        let mut next_food_id = 0;
        let mut next_bug_id = 0;

        let food = food
            .into_iter()
            .map(|create_info| {
                Food::new(&mut next_food_id, create_info.position, create_info.energy)
            })
            .collect();
        let food_sources = food_sources
            .into_iter()
            .map(|create_info| create_info.create(now.clone()))
            .collect();
        let bugs = bugs
            .into_iter()
            .map(|create_info| {
                RefCell::new(Bug::give_birth_with_max_energy(
                    &mut next_bug_id,
                    create_info.chromosome,
                    create_info.position,
                    create_info.rotation,
                    now.clone(),
                ))
            })
            .collect();

        Self {
            food,
            food_sources,
            bugs,
            creation_time: now.clone(),
            now,
            next_food_id: 0,
            next_bug_id: 0,
        }
    }

    pub fn generate<R: RngCore, Range: SampleRange<Float>>(
        now: T,
        rng: &mut R,
        food_sources: Vec<FoodSourceCreateInfo>,
        x_range: Range,
        y_range: Range,
        food_e_range: Range,
        food_count: usize,
        bug_position: Point<Float>,
    ) -> Self
    where
        Range: Clone,
        T: Clone,
    {
        let mut next_food_id = 0;
        let mut next_bug_id = 0;

        let food = Food::generate_vec(
            &mut next_food_id,
            rng,
            x_range,
            y_range,
            food_e_range,
            food_count,
        );
        let food_sources = food_sources
            .into_iter()
            .map(|x| x.create(now.clone()))
            .collect();
        let bugs = vec![RefCell::new(
            Bug::give_birth(
                &mut next_bug_id,
                Chromosome {
                    genes: (0..256)
                        .map(|i| {
                            if i == 0 {
                                1.
                            } else if i == 128 {
                                2.
                            } else if i == 128 + 8 + 8 + 8 {
                                0.5
                            // } else if i == 16 + 1 || i == 128 + 8 + 1 {
                            //     2.
                            } else if (0..208).contains(&i) {
                                0.
                            } else {
                                1.
                            }
                        })
                        .collect(),
                },
                bug_position,
                Angle::from_radians(rng.gen_range(0. ..(PI * 2.))),
                noneg_float(50.),
                now.clone(),
            )
            .unwrap(),
        )];

        Self {
            food,
            food_sources,
            bugs,
            creation_time: now.clone(),
            now,
            next_bug_id,
            next_food_id,
        }
    }

    pub fn now(&self) -> &T {
        &self.now
    }

    pub fn creation_time(&self) -> &T {
        &self.creation_time
    }

    pub fn proceed<R: RngCore>(&mut self, dt: Duration, rng: &mut R)
    where
        T: TimePoint + Clone,
    {
        self.now += dt;

        for food_source in &mut self.food_sources {
            let n = self
                .now
                .duration_since(&food_source.last_food_creation_instant)
                .div_duration_f64(food_source.spawn_interval)
                .floor();

            for _ in 0..(n as usize) {
                let rect = Rect::from_center(food_source.position, food_source.size);
                self.food.push(Food::generate(
                    &mut self.next_food_id,
                    rng,
                    sample_range_from_range(rect.x_range()),
                    sample_range_from_range(rect.y_range()),
                    sample_range_from_range(food_source.energy_range),
                ));
            }
            food_source.last_food_creation_instant += food_source.spawn_interval.mul_f64(n);
        }

        let mut requests: Vec<EnvironmentRequest> = Default::default();
        for b in &self.bugs {
            requests.append(&mut b.borrow_mut().proceed(&self, dt, rng));
        }

        for request in requests {
            match request {
                EnvironmentRequest::Kill { id } => self.kill(id),
                EnvironmentRequest::GiveBirth {
                    chromosome,
                    position,
                    rotation,
                    energy_level,
                } => {
                    for bug in Bug::give_birth_to_twins(
                        &mut self.next_bug_id,
                        chromosome,
                        position,
                        rotation,
                        energy_level,
                        self.now.clone(),
                    ) {
                        self.bugs.push(RefCell::new(bug));
                    }
                }
                EnvironmentRequest::TransferEnergyFromFoodToBug {
                    food_id,
                    bug_id,
                    delta_energy,
                } => self.transfer_energy_from_food_to_bug(food_id, bug_id, delta_energy),
            }
        }
    }

    pub fn find_bug_by_id<'a>(&'a self, id: usize) -> Option<Ref<'a, Bug<T>>> {
        self.bugs
            .iter()
            .find_map(|bug| bug.try_borrow().ok().filter(|bug| bug.id() == id))
    }

    fn kill(&mut self, id: usize) {
        self.bugs.retain(|x| x.borrow().id() != id);
    }

    fn transfer_energy_from_food_to_bug(
        &mut self,
        food_id: usize,
        bug_id: usize,
        delta_energy: NoNeg<Float>,
    ) {
        if let Some(bug_index) = self.bugs.iter().position(|b| b.borrow().id() == bug_id) {
            if let Some(food_index) = self.food.iter().position(|b| b.id() == food_id) {
                if self.bugs[bug_index]
                    .borrow_mut()
                    .eat(&mut self.food[food_index], delta_energy)
                {
                    self.food.remove(food_index);
                }
            }
        }
    }

    pub fn food(&self) -> &[Food] {
        &self.food
    }

    pub fn food_sources(&self) -> &[FoodSource<T>] {
        &self.food_sources
    }

    pub fn bugs_count(&self) -> usize {
        self.bugs.len()
    }

    pub fn bugs<'a>(&'a self) -> impl Iterator<Item = Ref<'a, Bug<T>>> {
        self.bugs.iter().filter_map(|x| x.try_borrow().ok())
    }
}
