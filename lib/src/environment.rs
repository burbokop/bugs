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
    species::{Member as _, Speciation, SpeciesTags},
    time_point::TimePoint,
    utils::Float,
};
use chromosome::Chromosome;
use rand::{distr::uniform::SampleRange, rand_core, Rng, RngCore, SeedableRng};
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
            (rng.random_range(x_range), rng.random_range(y_range)).into(),
            NoNeg::wrap(rng.random_range(e_range)).unwrap(),
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
            position: (rng.random_range(x_range), rng.random_range(y_range)).into(),
            energy: NoNeg::wrap(rng.random_range(e_range)).unwrap(),
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
            position: (rng.random_range(x_range), rng.random_range(y_range)).into(),
            rotation: Angle::from_radians(rng.random_range(r_range)),
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
        parent_species: SpeciesTags,
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

#[derive(Serialize)]
pub struct Environment<T> {
    food: FoodChunkedVec,
    food_sources: Vec<Rc<RefCell<FoodSource<T>>>>,
    bugs: BugsChunkedVec<T>,
    speciation: Speciation<T>,
    creation_time: T,
    now: T,
    next_food_id: usize,
    next_bug_id: usize,
    iteration: usize,
}

impl<'de, T: Deserialize<'de> + Clone> Deserialize<'de> for Environment<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(bound(deserialize = "T: Deserialize<'de>"))]
        struct TmpEnvironment<T> {
            food: FoodChunkedVec,
            food_sources: Vec<Rc<RefCell<FoodSource<T>>>>,
            bugs: BugsChunkedVec<T>,
            /// Missing in saves made before species were introduced.
            #[serde(default)]
            speciation: Speciation<T>,
            creation_time: T,
            now: T,
            next_food_id: usize,
            next_bug_id: usize,
            iteration: usize,
        }

        let mut val = TmpEnvironment::<T>::deserialize(deserializer)?;

        if val.speciation.is_empty() {
            // Legacy save: ancestry is unknown, so all living bugs go into a single root species.
            let mut bugs = val.bugs.iter();
            if let Some(first) = bugs.next() {
                val.speciation
                    .found_lineage(&mut *first.borrow_mut(), &val.creation_time);
                let legacy = first.borrow().species();
                for bug in std::iter::once(first).chain(bugs) {
                    bug.borrow_mut().set_species(legacy);
                    val.speciation.register_birth(legacy);
                }
            }
        }

        Ok(Self {
            food: val.food,
            food_sources: val.food_sources,
            bugs: val.bugs,
            speciation: val.speciation,
            creation_time: val.creation_time,
            now: val.now,
            next_food_id: val.next_food_id,
            next_bug_id: val.next_bug_id,
            iteration: val.iteration,
        })
    }
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

        let mut result = Self {
            food,
            food_sources,
            bugs: Default::default(),
            speciation: Default::default(),
            creation_time: now.clone(),
            now,
            next_food_id,
            next_bug_id: 0,
            iteration: 0,
        };

        for create_info in bugs {
            let bug = Bug::give_birth_with_max_energy(
                &mut result.next_bug_id,
                SpeciesTags::default(),
                create_info.chromosome,
                create_info.position,
                create_info.rotation,
                result.now.clone(),
            );
            result.add_founder(bug);
        }

        result
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
        let founder_chromosome = Chromosome {
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
        };

        let mut result = Self {
            food: food.into_iter().collect(),
            food_sources,
            bugs: Default::default(),
            speciation: Default::default(),
            creation_time: now.clone(),
            now,
            next_bug_id: 0,
            next_food_id,
            iteration: 0,
        };

        let founder = Bug::give_birth(
            &mut result.next_bug_id,
            SpeciesTags::default(),
            founder_chromosome,
            bug_position,
            Angle::from_radians(rng.random_range(0. ..(PI * 2.))),
            noneg_float(50.),
            result.now.clone(),
        )
        .unwrap();
        result.add_founder(founder);
        result
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
                        let (position, id, species) = {
                            let b = requester.bug_ref().unwrap();
                            (b.position(), b.id(), b.species())
                        };
                        let chunk_found = self
                            .bugs
                            .retain_by_position(position, |x| x.borrow().id() != id);
                        assert!(chunk_found);
                        self.speciation.register_death(species, &self.now);
                    }
                    EnvironmentRequest::GiveBirth {
                        parent_species,
                        chromosome,
                        position,
                        rotation,
                        energy_level,
                    } => {
                        let mut twins = Bug::give_birth_to_twins(
                            &mut self.next_bug_id,
                            parent_species,
                            chromosome,
                            position,
                            rotation,
                            energy_level,
                            self.now.clone(),
                        );
                        // Twins are genetically identical, so one classification fits all of them.
                        let species = self.speciation.classify_offspring(
                            parent_species,
                            twins[0].signature(),
                            twins[0].color(),
                            &self.now,
                        );
                        for mut bug in twins.drain(..) {
                            bug.set_species(species);
                            self.push_bug(bug);
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

        if self.speciation.recluster_due(&self.now) {
            let mut members: Vec<_> = self.bugs.iter().map(|b| b.borrow_mut()).collect();
            self.speciation.recluster(&mut members, &self.now);
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

    pub fn irradiate_area<R: rand_core::RngCore>(
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
            NoNeg::wrap(rng.random_range((0.)..8.)).unwrap(),
        ));
    }

    pub fn add_bug<R: RngCore>(&mut self, center: Point<Float>, rng: &mut R)
    where
        T: Clone,
    {
        let chromosome = Chromosome {
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
        };
        let bug = Bug::give_birth_with_max_energy(
            &mut self.next_bug_id,
            SpeciesTags::default(),
            chromosome,
            center,
            Angle::from_radians(rng.random_range(0. ..(PI * 2.))),
            self.now.clone(),
        );
        self.add_founder(bug);
    }

    /// Adds a bug with unknown ancestry: it starts a new lineage.
    fn add_founder(&mut self, mut bug: Bug<T>)
    where
        T: Clone,
    {
        self.speciation.found_lineage(&mut bug, &self.now);
        self.push_bug(bug);
    }

    /// The only way a bug enters the world after construction, so species populations stay in sync.
    fn push_bug(&mut self, bug: Bug<T>) {
        self.speciation.register_birth(bug.species());
        self.bugs.push(Rc::new(RefCell::new(bug)));
    }

    pub fn speciation(&self) -> &Speciation<T> {
        &self.speciation
    }

    pub fn food_chunks<'a>(&'a self) -> impl Iterator<Item = (ChunkIndex, &'a [Food])> + 'a {
        self.food.chunks()
    }

    pub fn food_chunks_in_area<'a>(
        &'a self,
        rect: Rect<Float>,
    ) -> impl Iterator<Item = (ChunkIndex, &'a [Food])> + 'a {
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

    pub fn bug_chunks<'a>(
        &'a self,
    ) -> impl Iterator<Item = (ChunkIndex, &'a [Rc<RefCell<Bug<T>>>])> + 'a {
        self.bugs.chunks()
    }

    pub fn bug_chunks_in_area<'a>(
        &'a self,
        rect: Rect<Float>,
    ) -> impl Iterator<Item = (ChunkIndex, &'a [Rc<RefCell<Bug<T>>>])> + 'a {
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
#[serde(bound(deserialize = "T: Deserialize<'de> + Clone"))]
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::{
        env_presets,
        species::{Method, SpeciesId},
        time_point::StaticTimePoint,
    };

    fn assert_species_in_sync<T>(env: &Environment<T>) {
        for method in [Method::Incremental, Method::Clustering] {
            let mut counted: HashMap<SpeciesId, usize> = HashMap::new();
            for bug in env.bugs() {
                *counted.entry(bug.species().get(method)).or_default() += 1;
            }
            let registry = env.speciation().registry(method);
            for (id, count) in &counted {
                let species = registry
                    .get(*id)
                    .expect("living bug in a forgotten species");
                assert_eq!(species.population(), *count, "{method:?} {id}");
            }
            for species in registry.iter() {
                let living = counted.get(&species.id()).copied().unwrap_or(0);
                assert_eq!(species.population(), living, "{method:?} {}", species.id());
                assert_eq!(species.is_extinct(), living == 0);
            }
        }
    }

    #[test]
    fn species_population_matches_living_bugs() {
        let mut env =
            env_presets::less_food_further_from_center(StaticTimePoint::default(), [7; 32]);
        env.add_bug((10., 10.).into());
        env.add_bug((-10., 10.).into());
        for method in [Method::Incremental, Method::Clustering] {
            assert_eq!(env.speciation().registry(method).len(), 3);
        }

        // ~20 simulated minutes
        for i in 0..36_000 {
            env.proceed(Duration::from_millis(1000 / 30));
            if i % 1000 == 0 {
                assert_species_in_sync(&env);
            }
            if env.bugs_count() == 0 {
                break;
            }
        }
        assert_species_in_sync(&env);
        assert!(env.bugs_count() > 0);
        for method in [Method::Incremental, Method::Clustering] {
            assert!(
                env.speciation().registry(method).ever_existed() > 3,
                "{method:?} never split"
            );
        }
    }
}

#[cfg(test)]
mod calibration {
    use super::*;
    use crate::{env_presets, time_point::StaticTimePoint};
    use rand::SeedableRng;

    fn quantiles(mut v: Vec<Float>) -> String {
        v.sort_by(|a, b| a.total_cmp(b));
        [0.1, 0.25, 0.5, 0.75, 0.9, 0.99]
            .iter()
            .map(|q| format!("q{}={:.4}", q, v[((v.len() - 1) as Float * q) as usize]))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Helps choosing speciation thresholds: prints distributions of parent-to-child and pairwise
    /// signature distances and the resulting species counts.
    /// `cargo test --release -p bugs-lib print_distance_distributions -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn print_distance_distributions() {
        let mut rng = Pcg64::seed_from_u64(1);
        for seed in [7u8, 42] {
            let mut env =
                env_presets::less_food_further_from_center(StaticTimePoint::default(), [seed; 32]);
            for minutes in [10, 30, 90] {
                while env.now().duration_since(env.creation_time())
                    < Duration::from_secs(minutes * 60)
                {
                    env.proceed(Duration::from_millis(1000 / 30));
                }
                let bugs: Vec<_> = env
                    .bugs()
                    .map(|b| (b.chromosome().clone(), b.signature().clone()))
                    .collect();
                let mutation: Vec<Float> = bugs
                    .iter()
                    .flat_map(|(c, s)| {
                        (0..5)
                            .map(|_| {
                                let m = c.clone().mutated(|_, _| 0.01..0.8, 0.01, &mut rng);
                                crate::bug::signature_of_chromosome(&m).distance(s)
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect();
                let step = (bugs.len() / 300).max(1);
                let sample: Vec<_> = bugs.iter().step_by(step).collect();
                let mut pairs = vec![];
                for a in 0..sample.len() {
                    for b in a + 1..sample.len() {
                        pairs.push(sample[a].1.distance(&sample[b].1));
                    }
                }
                let zero = mutation.iter().filter(|d| **d == 0.).count() as Float
                    / mutation.len() as Float;
                println!("seed {seed} t={minutes}m bugs={}", bugs.len());
                println!(
                    "  parent->child (zero {:.0}%): {}",
                    zero * 100.,
                    quantiles(mutation)
                );
                println!("  pairwise:                 {}", quantiles(pairs));
                for method in [
                    crate::species::Method::Incremental,
                    crate::species::Method::Clustering,
                ] {
                    let r = env.speciation().registry(method);
                    let alive: Vec<_> = r.alive().map(|s| s.population()).collect();
                    let big = r.iter().filter(|s| s.peak_population() >= 5).count();
                    println!(
                        "  {method:?}: ever={} remembered={} peak>=5: {} alive={} pops={:?}",
                        r.ever_existed(),
                        r.len(),
                        big,
                        alive.len(),
                        &alive[..alive.len().min(12)]
                    );
                }
            }
        }
    }
}
