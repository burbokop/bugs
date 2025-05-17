use std::{
    cell::{Ref, RefCell, RefMut},
    f64::consts::PI,
    ops::Deref,
    rc::Rc,
    time::Duration,
};

use crate::{
    bug::Bug,
    chunk::{ChunkIndex, ChunkedVec, Position, RawChunkIndex},
    food_source::{FoodSource, FoodSourceShape},
    math::{noneg_float, Angle, DeltaAngle, NoNeg, Point, Rect},
    range::Range,
    time_point::TimePoint,
    utils::Float,
};
use chromosome::Chromosome;
use rand::{distributions::uniform::SampleRange, RngCore};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Food {
    id: usize,
    position: Point<Float>,
    energy: NoNeg<Float>,
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

impl Position for Food {
    fn position(&self) -> Point<Float> {
        self.position
    }
}

impl Position for &Food {
    fn position(&self) -> Point<Float> {
        self.position
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

    pub(crate) fn create(self, next_id: &mut usize) -> Food {
        Food::new(next_id, self.position, self.energy)
    }
}

pub struct BugCreateInfo {
    pub chromosome: Chromosome<Float>,
    pub position: Point<Float>,
    pub rotation: Angle<Float>,
}

impl BugCreateInfo {
    pub(crate) fn generate<R: RngCore, RR: SampleRange<Float> + Clone>(
        rng: &mut R,
        g_range: RR,
        x_range: RR,
        y_range: RR,
        r_range: RR,
    ) -> Self {
        Self {
            chromosome: Chromosome::new_random(256, g_range, rng),
            position: (rng.gen_range(x_range), rng.gen_range(y_range)).into(),
            rotation: Angle::from_radians(rng.gen_range(r_range)),
        }
    }

    pub fn generate_vec<R: RngCore, RR: SampleRange<Float> + Clone>(
        rng: &mut R,
        g_range: RR,
        x_range: RR,
        y_range: RR,
        r_range: RR,
        count: usize,
    ) -> Vec<Self> {
        (0..count)
            .map(|_| {
                Self::generate(
                    rng,
                    g_range.clone(),
                    x_range.clone(),
                    y_range.clone(),
                    r_range.clone(),
                )
            })
            .collect()
    }
}

pub struct FoodSourceCreateInfo {
    pub position: Point<Float>,
    pub shape: FoodSourceShape,
    pub energy_range: Range<Float>,
    pub spawn_interval: Duration,
}

impl FoodSourceCreateInfo {
    pub(crate) fn create<T>(self, last_food_creation_instant: T) -> FoodSource<T> {
        FoodSource::new(
            self.position,
            self.shape,
            self.energy_range,
            self.spawn_interval,
            last_food_creation_instant,
        )
    }
}

pub(crate) enum EnvironmentRequest {
    Suicide,
    GiveBirth {
        chromosome: Chromosome<Float>,
        position: Point<Float>,
        rotation: Angle<Float>,
        energy_level: NoNeg<Float>,
    },
    TransferEnergyFromFoodToBug {
        food_id: usize,
        delta_energy: NoNeg<Float>,
    },
    PlaceFood(FoodCreateInfo),
}

type FoodChunkedVec = ChunkedVec<Food, 256, 256>;
type BugsChunkedVec<T> = ChunkedVec<Rc<RefCell<Bug<T>>>, 256, 256>;

#[derive(Serialize, Deserialize)]
pub struct Environment<T> {
    food: FoodChunkedVec,
    food_sources: Vec<Rc<RefCell<FoodSource<T>>>>,
    bugs: BugsChunkedVec<T>,
    creation_time: T,
    now: T,
    next_food_id: usize,
    next_bug_id: usize,
    iteration: usize,
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
            .map(|create_info| Rc::new(RefCell::new(create_info.create(now.clone()))))
            .collect();
        let bugs = bugs
            .into_iter()
            .map(|create_info| {
                Rc::new(RefCell::new(Bug::give_birth_with_max_energy(
                    &mut next_bug_id,
                    create_info.chromosome,
                    create_info.position,
                    create_info.rotation,
                    now.clone(),
                )))
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
            iteration: 0,
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
            .map(|x| Rc::new(RefCell::new(x.create(now.clone()))))
            .collect();
        let bugs = vec![Rc::new(RefCell::new(
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
        ))];

        Self {
            food: food.into_iter().collect(),
            food_sources,
            bugs: bugs.into_iter().collect(),
            creation_time: now.clone(),
            now,
            next_bug_id,
            next_food_id,
            iteration: 0,
        }
    }

    pub fn now(&self) -> &T {
        &self.now
    }

    pub fn creation_time(&self) -> &T {
        &self.creation_time
    }

    pub fn iteration(&self) -> usize {
        self.iteration
    }

    pub fn proceed<R: RngCore>(&mut self, dt: Duration, rng: &mut R)
    where
        T: TimePoint + Clone,
    {
        self.now += dt;

        enum Requester<T> {
            FoodSource(Rc<RefCell<FoodSource<T>>>),
            Bug(Rc<RefCell<Bug<T>>>),
        }

        impl<T> Requester<T> {
            fn bug_ref<'a>(&'a self) -> Option<RefMut<'a, Bug<T>>> {
                match self {
                    Requester::FoodSource(_) => None,
                    Requester::Bug(rc) => Some(rc.borrow_mut()),
                }
            }
        }

        let mut requests: Vec<(Requester<T>, Vec<EnvironmentRequest>)> = Default::default();
        {
            let now = self.now().clone();
            for food_source in &mut self.food_sources {
                let r = food_source.as_ref().borrow_mut().proceed(&now, rng);
                requests.push((Requester::FoodSource(food_source.clone()), r));
            }
        }

        for b in self.bugs.iter() {
            let r = b.as_ref().borrow_mut().proceed(&self, dt, rng);
            requests.push((Requester::Bug(b.clone()), r));
        }

        self.bugs.shuffle();

        for (requester, requests) in requests {
            for request in requests {
                match request {
                    EnvironmentRequest::Suicide => {
                        let (position, id) = {
                            let b = requester.bug_ref().unwrap();
                            (b.position(), b.id())
                        };
                        let chunk_found = self
                            .bugs
                            .retain_by_position(position, |x| x.borrow().id() != id);
                        assert!(chunk_found);
                    }
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
                            self.bugs.push(Rc::new(RefCell::new(bug)));
                        }
                    }
                    EnvironmentRequest::TransferEnergyFromFoodToBug {
                        food_id,
                        delta_energy,
                    } => self.transfer_energy_from_food_to_bug(
                        food_id,
                        &mut requester.bug_ref().unwrap(),
                        delta_energy,
                    ),
                    EnvironmentRequest::PlaceFood(food_create_info) => self
                        .food
                        .push(food_create_info.create(&mut self.next_food_id)),
                }
            }
        }

        self.iteration += 1;
    }

    pub fn find_bug_by_id<'a>(&'a self, id: usize) -> Option<Ref<'a, Bug<T>>> {
        self.bugs
            .iter()
            .find_map(|bug| bug.try_borrow().ok().filter(|bug| bug.id() == id))
    }

    fn transfer_energy_from_food_to_bug(
        &mut self,
        food_id: usize,
        bug: &mut Bug<T>,
        delta_energy: NoNeg<Float>,
    ) {
        if let Some(food_index) =
            self.food
                .index_of_in_range(|b| b.id() == food_id, bug.position(), bug.eat_range())
        {
            if bug.eat(&mut self.food[food_index.clone()], delta_energy) {
                self.food.remove(food_index);
            }
        }
    }

    pub fn food(&self) -> impl Iterator<Item = &Food> {
        self.food.iter()
    }

    pub fn food_count(&self) -> usize {
        self.food.len()
    }

    pub(crate) fn find_nearest_food_in_vision_arc(
        &self,
        position: Point<Float>,
        range: NoNeg<Float>,
        vision_rotation: Angle<Float>,
        vision_half_arc: DeltaAngle<NoNeg<Float>>,
    ) -> Option<(&Food, NoNeg<Float>)> {
        self.food.find_nearest_filter_map(position, range, |food| {
            let arc = Range {
                start: vision_rotation - vision_half_arc.unwrap(),
                end: vision_rotation + vision_half_arc.unwrap(),
            };

            if vision_half_arc == DeltaAngle::from_radians(noneg_float(PI))
                || (food.position().clone() - position)
                    .angle()
                    .is_contained_in(arc)
            {
                Some(food)
            } else {
                None
            }
        })
    }

    pub(crate) fn find_nearest_bug_in_vision_arc<'a>(
        &'a self,
        position: Point<Float>,
        range: NoNeg<Float>,
        vision_rotation: Angle<Float>,
        vision_half_arc: DeltaAngle<NoNeg<Float>>,
    ) -> Option<(Ref<'a, Bug<T>>, NoNeg<Float>)> {
        self.bugs.find_nearest_filter_map(position, range, |x| {
            x.try_borrow().ok().and_then(|other| {
                if vision_half_arc == DeltaAngle::from_radians(noneg_float(PI))
                    || (other.position().clone() - position)
                        .angle()
                        .is_contained_in(Range {
                            start: vision_rotation - vision_half_arc.unwrap(),
                            end: vision_rotation + vision_half_arc.unwrap(),
                        })
                {
                    Some(other)
                } else {
                    None
                }
            })
        })
    }

    pub fn food_sources<'a>(&'a self) -> impl Iterator<Item = Ref<'a, FoodSource<T>>> {
        self.food_sources.iter().map(|x| x.as_ref().borrow())
    }

    pub fn bugs_count(&self) -> usize {
        self.bugs.len()
    }

    pub fn bugs<'a>(&'a self) -> impl Iterator<Item = Ref<'a, Bug<T>>> {
        self.bugs.iter().filter_map(|x| x.try_borrow().ok())
    }

    pub fn irradiate_area<R: RngCore>(
        &mut self,
        center: Point<Float>,
        radius: NoNeg<Float>,
        rng: &mut R,
    ) {
        self.bugs
            .iter_mut()
            .filter_map(|x| x.try_borrow_mut().ok())
            .filter(|bug| (center - bug.position()).len() < radius.unwrap())
            .for_each(|mut bug| {
                bug.chromosome_mut().mutate(|_, _| 0.001..1., 1., rng);
            });
    }

    pub fn add_food<R: RngCore>(&mut self, center: Point<Float>, rng: &mut R) {
        self.food.push(Food::new(
            &mut self.next_bug_id,
            center,
            NoNeg::wrap(rng.gen_range((0.)..8.)).unwrap(),
        ));
    }

    pub fn add_bug<R: RngCore>(&mut self, center: Point<Float>, rng: &mut R)
    where
        T: Clone,
    {
        self.bugs
            .push(Rc::new(RefCell::new(Bug::give_birth_with_max_energy(
                &mut self.next_bug_id,
                Chromosome {
                    genes: (0..256)
                        .map(|i| {
                            if i == 0 {
                                2.
                            } else if i == 128 {
                                0.
                            } else if i == 18 {
                                2.
                            } else if i == 137 {
                                2.
                            } else if i == 33 {
                                2.
                            } else if i == 146 {
                                -2.
                            } else if i == 202 {
                                1.
                            } else if i == 130 {
                                2.
                            } else if i == 128 + 8 + 8 + 8 {
                                2. // baby charge
                            } else if (0..208).contains(&i) {
                                0.
                            } else {
                                1.
                            }
                        })
                        .collect(),
                },
                center,
                Angle::from_radians(rng.gen_range(0. ..(PI * 2.))),
                self.now.clone(),
            ))));
    }

    pub fn food_chunks<'a>(&'a self) -> impl Iterator<Item = (ChunkIndex, usize)> + 'a {
        self.food.chunks()
    }

    pub fn food_chunks_in_area<'a>(
        &'a self,
        rect: Rect<Float>,
    ) -> impl Iterator<Item = (ChunkIndex, usize)> + 'a {
        self.food.chunks_in_area(rect)
    }

    pub fn food_chunks_circular_traverse_iter(
        &self,
        position: Point<Float>,
        range: NoNeg<Float>,
    ) -> impl Iterator<Item = (isize, isize)> {
        FoodChunkedVec::circular_traverse_iter(position, range).map(|a| {
            let i: RawChunkIndex = a.into();
            (i.x(), i.y())
        })
    }

    pub fn bug_chunks<'a>(&'a self) -> impl Iterator<Item = (ChunkIndex, usize)> + 'a {
        self.bugs.chunks()
    }

    pub fn bug_chunks_in_area<'a>(
        &'a self,
        rect: Rect<Float>,
    ) -> impl Iterator<Item = (ChunkIndex, usize)> + 'a {
        self.bugs.chunks_in_area(rect)
    }

    pub fn bug_chunks_circular_traverse_iter(
        &self,
        position: Point<Float>,
        range: NoNeg<Float>,
    ) -> impl Iterator<Item = (isize, isize)> {
        BugsChunkedVec::<T>::circular_traverse_iter(position, range).map(|a| {
            let i: RawChunkIndex = a.into();
            (i.x(), i.y())
        })
    }

    pub(crate) fn collect_unused_chunks(&mut self) {
        self.bugs.collect_unused_chunks();
        self.food.collect_unused_chunks();
    }
}

#[derive(Serialize, Deserialize)]
pub struct SeededEnvironment<T> {
    env: Environment<T>,
    rng: Pcg64,
}

impl<T> SeededEnvironment<T> {
    pub fn generate<Range: SampleRange<Float>>(
        now: T,
        seed: <Pcg64 as SeedableRng>::Seed,
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
        let mut rng = Pcg64::from_seed(seed);
        Self {
            env: Environment::generate(
                now,
                &mut rng,
                food_sources,
                x_range,
                y_range,
                food_e_range,
                food_count,
                bug_position,
            ),
            rng,
        }
    }

    pub fn proceed(&mut self, dt: Duration)
    where
        T: TimePoint + Clone,
    {
        self.env.proceed(dt, &mut self.rng);
    }

    pub fn irradiate_area(&mut self, center: Point<Float>, radius: NoNeg<Float>) {
        self.env.irradiate_area(center, radius, &mut self.rng);
    }

    pub fn add_food(&mut self, center: Point<Float>) {
        self.env.add_food(center, &mut self.rng);
    }

    pub fn add_bug(&mut self, center: Point<Float>)
    where
        T: Clone,
    {
        self.env.add_bug(center, &mut self.rng);
    }

    pub fn collect_unused_chunks(&mut self) {
        self.env.collect_unused_chunks();
    }
}

// Note this impl does not brake SeededEnvironment invariant only if there is no immutable member function in Environment which accepts rng as argument
impl<T> Deref for SeededEnvironment<T> {
    type Target = Environment<T>;

    fn deref(&self) -> &Self::Target {
        &self.env
    }
}

pub mod benchmark_internals {
    use std::{cell::RefCell, rc::Rc};

    use crate::{bug::Bug, math::NoNeg, utils::Float};

    use super::Environment;

    pub fn transfer_energy_from_food_to_bug<T>(
        env: &mut Environment<T>,
        food_id: usize,
        bug: &mut Bug<T>,
        delta_energy: NoNeg<Float>,
    ) {
        env.transfer_energy_from_food_to_bug(food_id, bug, delta_energy)
    }

    pub fn find_bug_by_id<T>(env: &Environment<T>, id: usize) -> Option<Rc<RefCell<Bug<T>>>> {
        env.bugs.iter().find(|b| b.borrow().id() == id).cloned()
    }
}
