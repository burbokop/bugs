use std::{
    cell::{Ref, RefCell},
    f64::consts::PI,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use crate::{bug::Bug, math::Point, utils::Float};
use chromosome::Chromosome;
use rand::Rng;
use rand::{distributions::uniform::SampleRange, RngCore};

static NEXT_FOOD_ID: AtomicUsize = AtomicUsize::new(0);

pub(crate) struct Food {
    id: usize,
    position: Point<Float>,
    energy: Float,
}

impl Food {
    pub(crate) fn id(&self) -> usize {
        self.id
    }

    pub(crate) fn position(&self) -> Point<Float> {
        self.position
    }

    pub(crate) fn energy(&self) -> Float {
        self.energy
    }

    pub(crate) fn set_energy(&mut self, e: Float) {
        self.energy = e
    }

    pub(crate) fn generate<R: RngCore, RR: SampleRange<Float>>(
        rng: &mut R,
        x_range: RR,
        y_range: RR,
        e_range: RR,
    ) -> Self {
        Food {
            id: NEXT_FOOD_ID.fetch_add(1, Ordering::SeqCst),
            position: (rng.gen_range(x_range), rng.gen_range(y_range)).into(),
            energy: rng.gen_range(e_range),
        }
    }

    pub(crate) fn generate_vec<R: RngCore, RR: SampleRange<Float> + Clone>(
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

pub(crate) enum EnvironmentRequest {
    Kill {
        id: usize,
    },
    GiveBirth(Bug),
    TransferEnergyFromFoodToBug {
        food_id: usize,
        bug_id: usize,
        delta_energy: Float,
    },
}

pub(crate) struct Environment {
    food: Vec<Food>,
    bugs: Vec<RefCell<Bug>>,
}

impl Environment {
    pub fn new<R: RngCore, RR: SampleRange<Float> + Clone>(
        rng: &mut R,
        x_range: RR,
        y_range: RR,
        food_e_range: RR,
        food_count: usize,
    ) -> Self {
        Self {
            food: Food::generate_vec(
                rng,
                x_range.clone(),
                y_range.clone(),
                food_e_range,
                food_count,
            ),
            bugs: vec![RefCell::new(Bug::give_birth(
                Chromosome {
                    genes: (0..256).map(|_| 1.).collect(),
                },
                (rng.gen_range(x_range), rng.gen_range(y_range)).into(),
                rng.gen_range(0. ..PI),
            ))],
        }
    }
}

impl Environment {
    pub(crate) fn proceed<R: RngCore>(&mut self, dt: Duration, rng: &mut R) {
        let mut requests: Vec<EnvironmentRequest> = Default::default();
        for b in &self.bugs {
            requests.append(&mut b.borrow_mut().proceed(&self, dt, rng));
        }

        for request in requests {
            match request {
                EnvironmentRequest::Kill { id } => self.kill(id),
                EnvironmentRequest::GiveBirth(bug) => self.bugs.push(RefCell::new(bug)),
                EnvironmentRequest::TransferEnergyFromFoodToBug {
                    food_id,
                    bug_id,
                    delta_energy,
                } => self.transfer_energy_from_food_to_bug(food_id, bug_id, delta_energy),
            }
        }
    }

    pub(crate) fn find_bug_by_id<'a>(&'a self, id: usize) -> Option<Ref<'a, Bug>> {
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
        delta_energy: f64,
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

    pub(crate) fn food(&self) -> &[Food] {
        &self.food
    }

    pub(crate) fn bugs<'a>(&'a self) -> impl Iterator<Item = Ref<'a, Bug>> {
        self.bugs.iter().filter_map(|x| x.try_borrow().ok())
    }
}
