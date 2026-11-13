//! "Periodic splitting by clustering" speciation.
//!
//! Newborns always inherit their parent's species. Every `period` of simulated time the living
//! members of each species are clustered with single-linkage clustering: two individuals are
//! connected when their signatures are within `threshold`, and a cluster is a connected group.
//! When a species falls apart into two or more clusters of at least `min_cluster_size`
//! individuals, the biggest cluster keeps the species and every other one becomes a new child
//! species. Small clusters stay with the biggest one.
//!
//! Unlike the incremental method, a species that drifts as a whole is never split, only one that
//! actually branches.

use std::{collections::BTreeMap, ops::DerefMut, time::Duration};

use serde::{Deserialize, Serialize};

use super::{Member, SpeciesId, SpeciesRegistry};
use crate::{color::Color, time_point::TimePoint, utils::Float};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ClusteringSpeciation<T> {
    registry: SpeciesRegistry<T>,
    threshold: Float,
    min_cluster_size: usize,
    /// Pairwise clustering is quadratic, so big species are clustered on an evenly spread
    /// sample and the rest of the members join the cluster of their nearest sampled member.
    max_sample: usize,
    period: Duration,
    last_pass: Option<T>,
}

struct UnionFind(Vec<usize>);

impl UnionFind {
    fn new(n: usize) -> Self {
        Self((0..n).collect())
    }

    fn find(&mut self, mut i: usize) -> usize {
        while self.0[i] != i {
            self.0[i] = self.0[self.0[i]];
            i = self.0[i];
        }
        i
    }

    fn union(&mut self, a: usize, b: usize) {
        let (a, b) = (self.find(a), self.find(b));
        self.0[a] = b;
    }
}

impl<T> ClusteringSpeciation<T> {
    pub(crate) fn new(
        threshold: Float,
        min_cluster_size: usize,
        max_sample: usize,
        period: Duration,
    ) -> Self {
        Self {
            registry: Default::default(),
            threshold,
            min_cluster_size,
            max_sample,
            period,
            last_pass: None,
        }
    }

    pub(crate) fn registry(&self) -> &SpeciesRegistry<T> {
        &self.registry
    }

    pub(crate) fn found_lineage(&mut self, color: &Color, now: T) -> SpeciesId {
        self.registry.found(None, color.clone(), now)
    }

    pub(crate) fn register_birth(&mut self, id: SpeciesId) {
        self.registry.register_birth(id);
    }

    pub(crate) fn register_death(&mut self, id: SpeciesId, now: T) {
        self.registry.register_death(id, now);
    }

    pub(crate) fn is_due(&self, now: &T) -> bool
    where
        T: TimePoint,
    {
        match &self.last_pass {
            None => true,
            Some(last) => now.duration_since(last) >= self.period,
        }
    }

    pub(crate) fn recluster<M: Member, R: DerefMut<Target = M>>(
        &mut self,
        members: &mut [R],
        now: &T,
    ) where
        T: Clone,
    {
        self.last_pass = Some(now.clone());

        let mut groups: BTreeMap<SpeciesId, Vec<usize>> = Default::default();
        for (i, m) in members.iter().enumerate() {
            groups.entry(m.species().clustering).or_default().push(i);
        }

        for (species, indices) in groups {
            if indices.len() < 2 * self.min_cluster_size {
                continue;
            }
            let clusters = self.find_clusters(members, &indices);
            for cluster in clusters.into_iter().skip(1) {
                let color = members[cluster[0]].color().clone();
                let child = self.registry.found(Some(species), color, now.clone());
                self.registry
                    .transfer(species, child, cluster.len(), now.clone());
                for i in cluster {
                    let mut tags = members[i].species();
                    tags.clustering = child;
                    members[i].set_species(tags);
                }
            }
        }
    }

    /// Returns clusters (as indices into `members`) of at least `min_cluster_size` members,
    /// biggest first, with small clusters merged into the biggest one.
    /// Returns a single cluster if the species does not split.
    fn find_clusters<M: Member, R: DerefMut<Target = M>>(
        &self,
        members: &[R],
        indices: &[usize],
    ) -> Vec<Vec<usize>> {
        let n = indices.len();
        let s = n.min(self.max_sample);
        let sample: Vec<usize> = (0..s).map(|k| indices[k * n / s]).collect();
        let signature = |i: usize| members[i].signature();

        let mut uf = UnionFind::new(s);
        for a in 0..s {
            for b in (a + 1)..s {
                if uf.find(a) != uf.find(b)
                    && signature(sample[a]).distance(signature(sample[b])) <= self.threshold
                {
                    uf.union(a, b);
                }
            }
        }

        let mut sample_cluster: Vec<usize> = Vec::with_capacity(s);
        let mut root_to_cluster: BTreeMap<usize, usize> = Default::default();
        for k in 0..s {
            let root = uf.find(k);
            let next = root_to_cluster.len();
            sample_cluster.push(*root_to_cluster.entry(root).or_insert(next));
        }
        let mut sample_sizes = vec![0usize; root_to_cluster.len()];
        for &c in &sample_cluster {
            sample_sizes[c] += 1;
        }

        // Estimate full sizes from the sample to skip the expensive assignment in the common case.
        let scale = n as Float / s as Float;
        let is_major =
            |c: usize| sample_sizes[c] as Float * scale >= self.min_cluster_size as Float;
        if (0..sample_sizes.len()).filter(|c| is_major(*c)).count() < 2 {
            return vec![indices.to_vec()];
        }

        // Every member joins the major cluster of its nearest sampled member.
        let major_samples: Vec<usize> = (0..s).filter(|k| is_major(sample_cluster[*k])).collect();
        let mut clusters: BTreeMap<usize, Vec<usize>> = Default::default();
        for &i in indices {
            let nearest = major_samples
                .iter()
                .map(|&k| {
                    let d = if sample[k] == i {
                        0.
                    } else {
                        signature(i).distance(signature(sample[k]))
                    };
                    (k, d)
                })
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .unwrap()
                .0;
            clusters.entry(sample_cluster[nearest]).or_default().push(i);
        }

        let mut clusters: Vec<Vec<usize>> = clusters.into_values().collect();
        clusters.sort_by_key(|c| std::cmp::Reverse(c.len()));
        let (mut big, small): (Vec<_>, Vec<_>) = clusters
            .into_iter()
            .partition(|c| c.len() >= self.min_cluster_size);
        if big.is_empty() {
            return vec![indices.to_vec()];
        }
        for c in small {
            big[0].extend(c);
        }
        big
    }
}
