//! Renderer independent layout of a phylogenetic tree over time.
//!
//! Each shown species is a horizontal bar from its origin to its extinction (or now),
//! one row per species, rows in depth-first order so that every subtree occupies a block of
//! consecutive rows. A child is connected to its parent's row at the child's origin time.

use std::collections::BTreeSet;

use super::{SpeciesId, SpeciesRegistry};
use crate::{color::Color, time_point::TimePoint, utils::Float};

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub species: SpeciesId,
    pub row: usize,
    /// Row of the closest shown ancestor.
    pub parent_row: Option<usize>,
    /// Seconds since the environment was created.
    pub start: Float,
    pub end: Float,
    pub alive: bool,
    pub color: Color,
    pub population: usize,
    pub peak_population: usize,
}

#[derive(Debug, Clone, Default)]
pub struct TreeLayout {
    pub nodes: Vec<TreeNode>,
    pub rows: usize,
    /// Seconds since the environment was created.
    pub time_span: Float,
    /// Species whose peak population was below this were hidden (unless they have shown descendants).
    pub min_peak_population: usize,
}

impl<T: TimePoint> SpeciesRegistry<T> {
    /// Lays out species that reached at least `min_peak_population` individuals, together with
    /// their ancestors. If that is more than `max_rows`, the population bar is raised until it fits.
    pub fn layout(
        &self,
        creation_time: &T,
        now: &T,
        mut min_peak_population: usize,
        max_rows: usize,
    ) -> TreeLayout {
        min_peak_population = min_peak_population.max(1);
        let shown = loop {
            let shown = self.shown(min_peak_population);
            if shown.len() <= max_rows || min_peak_population > 1 << 20 {
                break shown;
            }
            min_peak_population *= 2;
        };

        let secs = |t: &T| t.duration_since(creation_time).as_secs_f64();
        let mut nodes = Vec::with_capacity(shown.len());
        let mut stack: Vec<(SpeciesId, Option<usize>)> = self
            .iter()
            .rev()
            .filter(|s| shown.contains(&s.id()) && s.parent().is_none_or(|p| !shown.contains(&p)))
            .map(|s| (s.id(), None))
            .collect();

        while let Some((id, parent_row)) = stack.pop() {
            let s = self.get(id).unwrap();
            let row = nodes.len();
            nodes.push(TreeNode {
                species: id,
                row,
                parent_row,
                start: secs(s.origin()),
                end: s.extinction().map(secs).unwrap_or_else(|| secs(now)),
                alive: !s.is_extinct(),
                color: s.color().clone(),
                population: s.population(),
                peak_population: s.peak_population(),
            });
            stack.extend(
                s.children()
                    .iter()
                    .rev()
                    .filter(|c| shown.contains(c))
                    .map(|c| (*c, Some(row))),
            );
        }

        TreeLayout {
            rows: nodes.len(),
            nodes,
            time_span: secs(now),
            min_peak_population,
        }
    }

    /// Species that reached `min_peak_population` and all their ancestors.
    fn shown(&self, min_peak_population: usize) -> BTreeSet<SpeciesId> {
        let mut shown = BTreeSet::new();
        // Children always have bigger ids than parents, so walking backwards visits
        // every child before its parent.
        for s in self.iter().rev() {
            if s.peak_population() >= min_peak_population
                || s.children().iter().any(|c| shown.contains(c))
            {
                shown.insert(s.id());
            }
        }
        shown
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time_point::StaticTimePoint;
    use std::time::Duration;

    fn at(secs: u64) -> StaticTimePoint {
        let mut t = StaticTimePoint::default();
        t += Duration::from_secs(secs);
        t
    }

    fn color() -> Color {
        Color {
            a: 1.,
            r: 0.,
            g: 0.,
            b: 0.,
        }
    }

    #[test]
    fn depth_first_rows_and_hidden_small_species() {
        let mut r = SpeciesRegistry::default();
        let root = r.found(None, color(), at(0));
        for _ in 0..5 {
            r.register_birth(root);
        }
        let a = r.found(Some(root), color(), at(10));
        r.transfer(root, a, 2, at(10));
        let small = r.found(Some(root), color(), at(12));
        r.register_birth(small);
        let a1 = r.found(Some(a), color(), at(20));
        r.transfer(a, a1, 1, at(20));
        r.register_birth(a1);

        let layout = r.layout(&at(0), &at(30), 2, 100);
        let order: Vec<_> = layout.nodes.iter().map(|n| n.species).collect();
        assert_eq!(order, vec![root, a, a1]);
        assert_eq!(layout.nodes[2].parent_row, Some(1));
        assert_eq!(layout.nodes[1].start, 10.);
        assert_eq!(layout.nodes[1].end, 30.);
        assert_eq!(layout.time_span, 30.);

        let limited = r.layout(&at(0), &at(30), 1, 2);
        assert!(limited.rows <= 2);
        assert!(limited.min_peak_population > 1);
    }
}
