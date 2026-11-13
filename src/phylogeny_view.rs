use std::collections::BTreeSet;

use bugs_lib::{
    color::Color,
    species::{Method, Speciation, SpeciesId},
    time_point::TimePoint,
};
use slint::{ModelRc, VecModel};

use crate::{app_utils::color_to_slint_rgba_f32_color, TreeBar, TreeInfo};

/// Species that never had this many members at once are hidden (unless they have shown descendants).
const MIN_PEAK_POPULATION: usize = 5;
const MAX_ROWS: usize = 150;
const BAR_THICKNESS: f32 = 0.6;
const EXTINCT_ALPHA: f64 = 0.35;

pub(crate) fn tree_info<T: TimePoint>(
    speciation: &Speciation<T>,
    method: Method,
    creation_time: &T,
    now: &T,
    selected_species: Option<SpeciesId>,
) -> TreeInfo {
    let registry = speciation.registry(method);
    let layout = registry.layout(creation_time, now, MIN_PEAK_POPULATION, MAX_ROWS);

    let lineage: BTreeSet<SpeciesId> = std::iter::successors(selected_species, |id| {
        registry.get(*id).and_then(|s| s.parent())
    })
    .collect();

    let rows = layout.rows.max(1) as f32;
    let span = layout.time_span.max(1e-9) as f32;
    let x = |secs: f64| secs as f32 / span;

    let connectors = layout.nodes.iter().filter_map(|node| {
        let parent_row = node.parent_row?;
        Some(TreeBar {
            x: x(node.start),
            y: (parent_row as f32 + 0.5) / rows,
            w: 0.,
            h: (node.row - parent_row) as f32 / rows,
            color: slint::Color::from_rgb_u8(0x99, 0x99, 0x99),
            highlighted: false,
        })
    });
    let bars = layout.nodes.iter().map(|node| TreeBar {
        x: x(node.start),
        y: (node.row as f32 + (1. - BAR_THICKNESS) / 2.) / rows,
        w: x(node.end) - x(node.start),
        h: BAR_THICKNESS / rows,
        color: color_to_slint_rgba_f32_color(&Color {
            a: if node.alive { 1. } else { EXTINCT_ALPHA },
            ..node.color.clone()
        })
        .into(),
        highlighted: lineage.contains(&node.species),
    });
    // Connectors first so that bars are drawn over them.
    let bars: Vec<TreeBar> = connectors.chain(bars).collect();

    let mut summary = format!(
        "alive: {}, remembered: {}, ever existed: {}, shown: {}",
        registry.alive_count(),
        registry.len(),
        registry.ever_existed(),
        layout.rows
    );
    summary += &format!(" (peak population ≥ {})", layout.min_peak_population);
    if let Some(s) = selected_species.and_then(|id| registry.get(id)) {
        summary += &format!(
            "\nselected bug: species #{}, population {}, peak {}",
            s.id(),
            s.population(),
            s.peak_population()
        );
    }

    TreeInfo {
        title: match method {
            Method::Incremental => "Incremental (classified at birth)",
            Method::Clustering => "Clustering (periodic splitting)",
        }
        .into(),
        summary: summary.into(),
        bars: ModelRc::new(VecModel::from(bars)),
    }
}
