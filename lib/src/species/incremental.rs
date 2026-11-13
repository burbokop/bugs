//! "Incremental, at birth" speciation.
//!
//! Every living species keeps the signature of its founder as a representative.
//! A newborn is compared with the representative of its parent's species:
//! - close enough: it stays in the parent's species,
//! - too far, but close to a living species that already branched off the parent's species:
//!   it joins that one (so the same mutation doesn't found many sibling species),
//! - otherwise: it founds a new child species and becomes its representative.
//!
//! Cheap (O(1) distance computations per birth in the common case), but because the
//! representative never moves, a lineage that slowly drifts without branching is also cut
//! into a chain of successive species.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{Signature, SpeciesId, SpeciesRegistry};
use crate::{color::Color, utils::Float};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct IncrementalSpeciation<T> {
    registry: SpeciesRegistry<T>,
    /// Only living species have a representative.
    representatives: BTreeMap<SpeciesId, Signature>,
    threshold: Float,
}

impl<T> IncrementalSpeciation<T> {
    pub(crate) fn new(threshold: Float) -> Self {
        Self {
            registry: Default::default(),
            representatives: Default::default(),
            threshold,
        }
    }

    pub(crate) fn registry(&self) -> &SpeciesRegistry<T> {
        &self.registry
    }

    pub(crate) fn found_lineage(
        &mut self,
        signature: &Signature,
        color: &Color,
        now: T,
    ) -> SpeciesId {
        let id = self.registry.found(None, color.clone(), now);
        self.representatives.insert(id, signature.clone());
        id
    }

    pub(crate) fn classify_offspring(
        &mut self,
        parent_species: SpeciesId,
        signature: &Signature,
        color: &Color,
        now: T,
    ) -> SpeciesId {
        let Some(representative) = self.representatives.get(&parent_species) else {
            debug_assert!(
                false,
                "living species {parent_species} has no representative"
            );
            return parent_species;
        };
        if signature.distance(representative) <= self.threshold {
            return parent_species;
        }

        let closest_sibling = self
            .registry
            .get(parent_species)
            .into_iter()
            .flat_map(|s| s.children())
            .filter_map(|c| {
                self.representatives
                    .get(c)
                    .map(|r| (*c, signature.distance(r)))
            })
            .filter(|(_, d)| *d <= self.threshold)
            .min_by(|a, b| a.1.total_cmp(&b.1));
        if let Some((id, _)) = closest_sibling {
            return id;
        }

        let id = self
            .registry
            .found(Some(parent_species), color.clone(), now);
        self.representatives.insert(id, signature.clone());
        id
    }

    pub(crate) fn register_birth(&mut self, id: SpeciesId) {
        self.registry.register_birth(id);
    }

    pub(crate) fn register_death(&mut self, id: SpeciesId, now: T) {
        if self.registry.register_death(id, now) {
            self.representatives.remove(&id);
        }
    }
}
