//! Species bookkeeping.
//!
//! Two speciation methods run side by side on the same population so that they can be compared:
//! - [`Method::Incremental`]: a newborn that differs too much from its species' founder
//!   founds a new species right away (see `incremental.rs`),
//! - [`Method::Clustering`]: newborns inherit the species, and species are periodically split
//!   when their living members fall apart into distinct clusters (see `clustering.rs`).
//!
//! Each method keeps its own [`SpeciesRegistry`] with every species that ever existed, so each
//! produces its own phylogenetic tree. Every bug carries one species id per method ([`SpeciesTags`]).
//!
//! The environment only reports events (new lineage, birth, death) and periodically hands over
//! the living population for reclustering. Everything else lives in this module.

mod clustering;
mod incremental;
mod layout;
mod registry;
mod signature;

use std::{ops::DerefMut, time::Duration};

use serde::{Deserialize, Serialize};

use crate::{color::Color, time_point::TimePoint, utils::Float};
use clustering::ClusteringSpeciation;
use incremental::IncrementalSpeciation;

pub use layout::{TreeLayout, TreeNode};
pub use registry::{Species, SpeciesId, SpeciesRegistry};
pub use signature::Signature;
pub(crate) use signature::Traits;

/// Distance (see [`Signature::distance`]) above which a newborn founds a new species
/// in the incremental method.
const INCREMENTAL_THRESHOLD: Float = 0.3;
/// Distance at which two individuals are considered connected in the clustering method.
const CLUSTERING_THRESHOLD: Float = 0.15;
const CLUSTERING_MIN_CLUSTER_SIZE: usize = 5;
const CLUSTERING_MAX_SAMPLE: usize = 512;
const CLUSTERING_PERIOD: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Incremental,
    Clustering,
}

/// Species of an individual according to each method.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpeciesTags {
    pub incremental: SpeciesId,
    pub clustering: SpeciesId,
}

impl SpeciesTags {
    pub fn get(&self, method: Method) -> SpeciesId {
        match method {
            Method::Incremental => self.incremental,
            Method::Clustering => self.clustering,
        }
    }
}

/// What speciation needs to know about an individual.
pub(crate) trait Member {
    fn species(&self) -> SpeciesTags;
    fn set_species(&mut self, tags: SpeciesTags);
    fn signature(&self) -> &Signature;
    fn color(&self) -> &Color;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Speciation<T> {
    incremental: IncrementalSpeciation<T>,
    clustering: ClusteringSpeciation<T>,
}

impl<T> Default for Speciation<T> {
    fn default() -> Self {
        Self {
            incremental: IncrementalSpeciation::new(INCREMENTAL_THRESHOLD),
            clustering: ClusteringSpeciation::new(
                CLUSTERING_THRESHOLD,
                CLUSTERING_MIN_CLUSTER_SIZE,
                CLUSTERING_MAX_SAMPLE,
                CLUSTERING_PERIOD,
            ),
        }
    }
}

impl<T> Speciation<T> {
    pub fn registry(&self, method: Method) -> &SpeciesRegistry<T> {
        match method {
            Method::Incremental => self.incremental.registry(),
            Method::Clustering => self.clustering.registry(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.incremental.registry().is_empty() && self.clustering.registry().is_empty()
    }

    /// Creates a root species in every method for an individual with unknown ancestry
    /// and tags it. The founder still has to be registered with [`Self::register_birth`].
    pub(crate) fn found_lineage<M: Member>(&mut self, founder: &mut M, now: &T)
    where
        T: Clone,
    {
        let tags = SpeciesTags {
            incremental: self.incremental.found_lineage(
                founder.signature(),
                founder.color(),
                now.clone(),
            ),
            clustering: self.clustering.found_lineage(founder.color(), now.clone()),
        };
        founder.set_species(tags);
    }

    /// Decides the species of offspring with the given signature and color.
    /// Call [`Self::register_birth`] for every individual actually born with these tags.
    pub(crate) fn classify_offspring(
        &mut self,
        parent: SpeciesTags,
        signature: &Signature,
        color: &Color,
        now: &T,
    ) -> SpeciesTags
    where
        T: Clone,
    {
        SpeciesTags {
            incremental: self.incremental.classify_offspring(
                parent.incremental,
                signature,
                color,
                now.clone(),
            ),
            clustering: parent.clustering,
        }
    }

    pub(crate) fn register_birth(&mut self, tags: SpeciesTags) {
        self.incremental.register_birth(tags.incremental);
        self.clustering.register_birth(tags.clustering);
    }

    pub(crate) fn register_death(&mut self, tags: SpeciesTags, now: &T)
    where
        T: Clone,
    {
        self.incremental
            .register_death(tags.incremental, now.clone());
        self.clustering.register_death(tags.clustering, now.clone());
    }

    pub(crate) fn recluster_due(&self, now: &T) -> bool
    where
        T: TimePoint,
    {
        self.clustering.is_due(now)
    }

    /// `members` must be the whole living population.
    pub(crate) fn recluster<M: Member, R: DerefMut<Target = M>>(
        &mut self,
        members: &mut [R],
        now: &T,
    ) where
        T: Clone,
    {
        self.clustering.recluster(members, now);
    }
}
