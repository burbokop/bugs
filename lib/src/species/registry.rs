use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::color::Color;

pub type SpeciesId = usize;

/// Extinct leaf species that never had more members than this are forgotten
/// to keep the registry (and the tree) from filling up with failed single mutants.
const MIN_MEMBERS_TO_REMEMBER: usize = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Species<T> {
    id: SpeciesId,
    parent: Option<SpeciesId>,
    children: Vec<SpeciesId>,
    origin: T,
    extinction: Option<T>,
    /// Color of the individual the species was founded from.
    color: Color,
    population: usize,
    peak_population: usize,
    /// Number of individuals that ever belonged to the species
    /// (born into it or moved into it when it split off its parent).
    total_members: usize,
}

impl<T> Species<T> {
    pub fn id(&self) -> SpeciesId {
        self.id
    }

    /// Species this one branched from. `None` for root lineages.
    pub fn parent(&self) -> Option<SpeciesId> {
        self.parent
    }

    /// Species that branched from this one, in order of appearance.
    pub fn children(&self) -> &[SpeciesId] {
        &self.children
    }

    pub fn origin(&self) -> &T {
        &self.origin
    }

    /// Time the last individual died. `None` while the species is alive.
    pub fn extinction(&self) -> Option<&T> {
        self.extinction.as_ref()
    }

    pub fn is_extinct(&self) -> bool {
        self.extinction.is_some()
    }

    pub fn color(&self) -> &Color {
        &self.color
    }

    /// Number of currently living individuals.
    pub fn population(&self) -> usize {
        self.population
    }

    pub fn peak_population(&self) -> usize {
        self.peak_population
    }

    pub fn total_members(&self) -> usize {
        self.total_members
    }
}

/// Record of every species that has ever existed, extinct ones included,
/// except trivial extinct leaves (see [`MIN_MEMBERS_TO_REMEMBER`]).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeciesRegistry<T> {
    species: BTreeMap<SpeciesId, Species<T>>,
    next_id: SpeciesId,
    forgotten: usize,
}

impl<T> Default for SpeciesRegistry<T> {
    fn default() -> Self {
        Self {
            species: Default::default(),
            next_id: 0,
            forgotten: 0,
        }
    }
}

impl<T> SpeciesRegistry<T> {
    pub fn get(&self, id: SpeciesId) -> Option<&Species<T>> {
        self.species.get(&id)
    }

    /// All remembered species in order of appearance.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &Species<T>> {
        self.species.values()
    }

    pub fn alive(&self) -> impl Iterator<Item = &Species<T>> {
        self.iter().filter(|s| !s.is_extinct())
    }

    pub fn alive_count(&self) -> usize {
        self.alive().count()
    }

    /// Number of remembered species.
    pub fn len(&self) -> usize {
        self.species.len()
    }

    pub fn is_empty(&self) -> bool {
        self.species.is_empty()
    }

    /// Number of species that ever existed, including forgotten trivial ones.
    pub fn ever_existed(&self) -> usize {
        self.next_id
    }

    /// Creates a species with no members yet. Its members have to be added with
    /// [`Self::register_birth`] or [`Self::transfer`].
    pub(crate) fn found(&mut self, parent: Option<SpeciesId>, color: Color, now: T) -> SpeciesId {
        let id = self.next_id;
        self.next_id += 1;
        if let Some(parent) = parent {
            self.species
                .get_mut(&parent)
                .expect("parent species must exist")
                .children
                .push(id);
        }
        self.species.insert(
            id,
            Species {
                id,
                parent,
                children: Default::default(),
                origin: now,
                extinction: None,
                color,
                population: 0,
                peak_population: 0,
                total_members: 0,
            },
        );
        id
    }

    pub(crate) fn register_birth(&mut self, id: SpeciesId) {
        self.add_members(id, 1);
    }

    /// Returns true if the species went extinct.
    pub(crate) fn register_death(&mut self, id: SpeciesId, now: T) -> bool {
        self.remove_members(id, 1, now)
    }

    /// Moves `count` living individuals from one species to another (used when a species splits).
    pub(crate) fn transfer(&mut self, from: SpeciesId, to: SpeciesId, count: usize, now: T) {
        self.remove_members(from, count, now);
        self.add_members(to, count);
    }

    fn add_members(&mut self, id: SpeciesId, count: usize) {
        let s = self.species.get_mut(&id).expect("species must exist");
        s.population += count;
        s.total_members += count;
        s.peak_population = s.peak_population.max(s.population);
        s.extinction = None;
    }

    fn remove_members(&mut self, id: SpeciesId, count: usize, now: T) -> bool {
        let s = self.species.get_mut(&id).expect("species must exist");
        debug_assert!(s.population >= count);
        s.population = s.population.saturating_sub(count);
        if s.population == 0 {
            s.extinction = Some(now);
            self.forget_if_trivial(id);
            true
        } else {
            false
        }
    }

    fn forget_if_trivial(&mut self, mut id: SpeciesId) {
        loop {
            let Some(s) = self.species.get(&id) else {
                return;
            };
            if !s.is_extinct()
                || !s.children.is_empty()
                || s.total_members > MIN_MEMBERS_TO_REMEMBER
            {
                return;
            }
            let parent = s.parent;
            self.species.remove(&id);
            self.forgotten += 1;
            match parent {
                Some(parent) => {
                    let p = self.species.get_mut(&parent).unwrap();
                    p.children.retain(|c| *c != id);
                    id = parent;
                }
                None => return,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn color() -> Color {
        Color {
            a: 1.,
            r: 0.,
            g: 0.,
            b: 0.,
        }
    }

    #[test]
    fn population_and_extinction() {
        let mut r = SpeciesRegistry::<u32>::default();
        let a = r.found(None, color(), 0);
        r.register_birth(a);
        r.register_birth(a);
        r.register_birth(a);
        assert_eq!(r.get(a).unwrap().population(), 3);
        assert_eq!(r.get(a).unwrap().peak_population(), 3);

        r.register_death(a, 5);
        r.register_death(a, 6);
        assert!(!r.get(a).unwrap().is_extinct());
        assert!(r.register_death(a, 7));
        assert_eq!(r.get(a).unwrap().extinction(), Some(&7));
        assert_eq!(r.alive_count(), 0);
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn trivial_extinct_leaves_are_forgotten() {
        let mut r = SpeciesRegistry::<u32>::default();
        let root = r.found(None, color(), 0);
        r.register_birth(root);
        let mutant = r.found(Some(root), color(), 1);
        r.register_birth(mutant);
        assert_eq!(r.get(root).unwrap().children(), &[mutant]);

        r.register_death(mutant, 2);
        assert!(r.get(mutant).is_none());
        assert!(r.get(root).unwrap().children().is_empty());
        assert_eq!(r.ever_existed(), 2);
    }

    #[test]
    fn transfer_moves_population() {
        let mut r = SpeciesRegistry::<u32>::default();
        let root = r.found(None, color(), 0);
        for _ in 0..10 {
            r.register_birth(root);
        }
        let child = r.found(Some(root), color(), 1);
        r.transfer(root, child, 4, 1);
        assert_eq!(r.get(root).unwrap().population(), 6);
        assert_eq!(r.get(child).unwrap().population(), 4);
        assert_eq!(r.get(child).unwrap().total_members(), 4);
    }
}
