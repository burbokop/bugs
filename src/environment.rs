use std::{
    cell::{Ref, RefCell},
    f64::consts::PI,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

use crate::{
    bug::{Bug, BUG_ENERGY_CAPACITY},
    math::{NoNeg, Point},
    utils::Float,
};
use chromosome::Chromosome;
use rand::Rng;
use rand::{distributions::uniform::SampleRange, RngCore};

static NEXT_FOOD_ID: AtomicUsize = AtomicUsize::new(0);

pub(crate) struct Food {
    id: usize,
    position: Point<Float>,
    energy: NoNeg<Float>,
}

impl Food {
    pub(crate) fn id(&self) -> usize {
        self.id
    }

    pub(crate) fn position(&self) -> Point<Float> {
        self.position
    }

    pub(crate) fn energy(&self) -> NoNeg<Float> {
        self.energy
    }

    pub(crate) fn energy_mut(&mut self) -> &mut NoNeg<Float> {
        &mut self.energy
    }

    pub(crate) fn set_energy(&mut self, e: NoNeg<Float>) {
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
            energy: NoNeg::wrap(rng.gen_range(e_range)).unwrap(),
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
        delta_energy: NoNeg<Float>,
    },
}

pub(crate) struct Environment<R: SampleRange<Float>> {
    food: Vec<Food>,
    bugs: Vec<RefCell<Bug>>,
    now: Instant,
    x_range: R,
    y_range: R,
    food_e_range: R,
    last_food_creation_instant: Instant,
}

impl<Range: SampleRange<Float>> Environment<Range> {
    pub fn new<R: RngCore>(
        rng: &mut R,
        x_range: Range,
        y_range: Range,
        food_e_range: Range,
        food_count: usize,
        bug_position: Point<Float>,
    ) -> Self
    where
        Range: Clone,
    {
        let now = Instant::now();
        Self {
            food: Food::generate_vec(
                rng,
                x_range.clone(),
                y_range.clone(),
                food_e_range.clone(),
                food_count,
            ),
            bugs: vec![RefCell::new(Bug::give_birth(
                Chromosome {
                    genes: (0..256)
                        .map(|i| {
                            if i == 0 || i == 128 || i == 128 + 8 + 8 {
                                1.
                            } else if (0..208).contains(&i) {
                                0.
                            } else {
                                1.
                            }
                        })
                        .collect(),
                },
                bug_position,
                rng.gen_range(0. ..PI),
                BUG_ENERGY_CAPACITY,
                now,
            ))],
            now,
            x_range,
            y_range,
            food_e_range,
            last_food_creation_instant: now,
        }
    }

    pub(crate) fn now(&self) -> &Instant {
        &self.now
    }

    pub(crate) fn proceed<R: RngCore>(&mut self, dt: Duration, rng: &mut R)
    where
        Range: Clone,
    {
        self.now += dt;

        if self.now - self.last_food_creation_instant > Duration::from_millis(10) {
            self.food.push(Food::generate(
                rng,
                self.x_range.clone(),
                self.y_range.clone(),
                self.food_e_range.clone(),
            ));
            self.last_food_creation_instant = self.now;
        }

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

    pub(crate) fn food(&self) -> &[Food] {
        &self.food
    }

    pub(crate) fn bugs<'a>(&'a self) -> impl Iterator<Item = Ref<'a, Bug>> {
        self.bugs.iter().filter_map(|x| x.try_borrow().ok())
    }
}
