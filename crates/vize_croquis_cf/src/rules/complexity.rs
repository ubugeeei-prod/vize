//! Cross-file complexity scoring.
//!
//! The template-control-flow dimension is fed by the S2
//! `template-complexity` facts (Davinci P4-9a/P4-9b, [`template`]): own
//! cyclomatic and cognitive complexity per component, summed over the
//! components in scope. Nothing here scans expression text.

mod counts;
mod hotspots;
mod record;
mod rendered;
mod template;

use counts::{add_count, fallthrough_risk_count};

use crate::analyzer::CrossFileResult;
use crate::registry::{FileId, ModuleRegistry};
use crate::rules::cross_file_reactivity::CrossFileReactivityIssueKind;
use vize_carton::FxHashMap;
use vize_croquis::EffectGraphSummary;

pub use hotspots::ComplexityHotspot;
pub(crate) use hotspots::summarize_complexity_hotspots_with_effect_graphs;
pub use rendered::ComponentComplexity;
pub(crate) use rendered::summarize_template_complexity;
pub use template::{
    ComplexityContributor, TEMPLATE_COGNITIVE_WARN_ABOVE, TEMPLATE_CYCLOMATIC_WARN_ABOVE,
    TemplateComplexity, TemplateScores,
};

/// Own template facts by file, recorded as files are added.
pub(crate) type TemplateFacts = FxHashMap<FileId, TemplateComplexity>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComplexityInput {
    pub component_count: usize,
    /// Σ own template cyclomatic complexity (S2 facts).
    pub template_cyclomatic: usize,
    /// Σ own template cognitive complexity (S2 facts).
    pub template_cognitive: usize,
    /// Evaluated template positions without a retained expression AST.
    pub template_unknown: usize,
    /// Deepest template nesting reached by one component.
    pub template_max_nesting: usize,
    /// Scoped-slot regions across the templates.
    pub template_scoped_slot_count: usize,
    pub slot_count: usize,
    pub prop_drilling_edge_count: usize,
    pub global_state_reference_count: usize,
    pub provide_inject_max_depth: usize,
    pub provide_inject_reference_count: usize,
    pub provide_inject_fanout_count: usize,
    pub fallthrough_risk_count: usize,
    pub reactive_node_count: usize,
    pub reactive_edge_count: usize,
    pub reactive_cycle_count: usize,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComplexityDimensionScores {
    pub template_control_flow: u32,
    pub slot_usage: u32,
    pub prop_drilling: u32,
    pub global_state: u32,
    pub provide_inject: u32,
    pub fallthrough_attrs: u32,
    pub reactive_graph: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ComplexityDimension {
    TemplateControlFlow,
    SlotUsage,
    PropDrilling,
    GlobalState,
    ProvideInject,
    FallthroughAttrs,
    ReactiveGraph,
}

impl ComplexityDimension {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TemplateControlFlow => "template-control-flow",
            Self::SlotUsage => "slot-usage",
            Self::PropDrilling => "prop-drilling",
            Self::GlobalState => "global-state",
            Self::ProvideInject => "provide-inject",
            Self::FallthroughAttrs => "fallthrough-attrs",
            Self::ReactiveGraph => "reactive-graph",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComplexityDimensionBreakdown {
    pub dimension: ComplexityDimension,
    pub score: u32,
}

impl ComplexityDimensionScores {
    pub fn total(self) -> u32 {
        self.template_control_flow
            .saturating_add(self.slot_usage)
            .saturating_add(self.prop_drilling)
            .saturating_add(self.global_state)
            .saturating_add(self.provide_inject)
            .saturating_add(self.fallthrough_attrs)
            .saturating_add(self.reactive_graph)
    }

    pub fn breakdown(self) -> [ComplexityDimensionBreakdown; 7] {
        [
            (
                ComplexityDimension::TemplateControlFlow,
                self.template_control_flow,
            ),
            (ComplexityDimension::SlotUsage, self.slot_usage),
            (ComplexityDimension::PropDrilling, self.prop_drilling),
            (ComplexityDimension::GlobalState, self.global_state),
            (ComplexityDimension::ProvideInject, self.provide_inject),
            (
                ComplexityDimension::FallthroughAttrs,
                self.fallthrough_attrs,
            ),
            (ComplexityDimension::ReactiveGraph, self.reactive_graph),
        ]
        .map(|(dimension, score)| ComplexityDimensionBreakdown { dimension, score })
    }

    pub fn dominant_dimension(self) -> Option<ComplexityDimensionBreakdown> {
        self.breakdown()
            .into_iter()
            .filter(|breakdown| breakdown.score > 0)
            .max_by_key(|breakdown| breakdown.score)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ComplexityBand {
    #[default]
    Low,
    Moderate,
    High,
    Extreme,
}

/// Explainable complexity score for one cross-file graph or component slice.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComplexityReport {
    pub input: ComplexityInput,
    pub dimensions: ComplexityDimensionScores,
    pub cyclomatic_score: u32,
    pub cognitive_score: u32,
    pub total_score: u32,
    pub band: ComplexityBand,
}

impl ComplexityReport {
    pub fn from_input(input: ComplexityInput) -> Self {
        let cyclomatic_score = cyclomatic_score(input);
        let cognitive_score = cognitive_score(input);
        let dimensions = ComplexityDimensionScores {
            template_control_flow: cyclomatic_score.saturating_add(cognitive_score),
            slot_usage: weighted(input.slot_count, 2)
                .saturating_add(weighted(input.template_scoped_slot_count, 2)),
            prop_drilling: weighted(input.prop_drilling_edge_count, 3),
            global_state: weighted(input.global_state_reference_count, 2),
            provide_inject: weighted(input.provide_inject_max_depth.saturating_sub(1), 2)
                .saturating_add(weighted(input.provide_inject_reference_count, 1))
                .saturating_add(provide_inject_fanout_score(
                    input.provide_inject_fanout_count,
                )),
            fallthrough_attrs: weighted(input.fallthrough_risk_count, 4),
            reactive_graph: weighted(input.reactive_node_count, 1)
                .saturating_add(weighted(input.reactive_edge_count, 2))
                .saturating_add(weighted(input.reactive_cycle_count, 10)),
        };
        let total_score = dimensions.total();

        Self {
            input,
            dimensions,
            cyclomatic_score,
            cognitive_score,
            total_score,
            band: band_for_score(total_score),
        }
    }

    pub fn dominant_dimension(self) -> Option<ComplexityDimensionBreakdown> {
        self.dimensions.dominant_dimension()
    }
}

/// Convert a total score to a human-facing complexity band.
pub fn band_for_score(score: u32) -> ComplexityBand {
    match score {
        0..=14 => ComplexityBand::Low,
        15..=34 => ComplexityBand::Moderate,
        35..=69 => ComplexityBand::High,
        _ => ComplexityBand::Extreme,
    }
}

/// Summarize complexity without template facts or effect graphs.
#[cfg(test)]
pub(super) fn summarize_complexity(
    registry: &ModuleRegistry,
    result: &CrossFileResult,
) -> ComplexityReport {
    summarize_complexity_with_effect_graphs(
        registry,
        &TemplateFacts::default(),
        &FxHashMap::default(),
        result,
    )
}

/// Summarize complexity from the analyzer's cross-file facts.
pub(crate) fn summarize_complexity_with_effect_graphs(
    registry: &ModuleRegistry,
    templates: &TemplateFacts,
    effect_graphs: &FxHashMap<FileId, EffectGraphSummary>,
    result: &CrossFileResult,
) -> ComplexityReport {
    ComplexityReport::from_input(complexity_input(registry, templates, effect_graphs, result))
}

/// Add one component's own template facts to an input.
pub(super) fn add_template_facts(input: &mut ComplexityInput, template: &TemplateComplexity) {
    add_count(
        &mut input.template_cyclomatic,
        template.own.cyclomatic as usize,
    );
    add_count(
        &mut input.template_cognitive,
        template.own.cognitive as usize,
    );
    add_count(&mut input.template_unknown, template.unknown as usize);
    add_count(
        &mut input.template_scoped_slot_count,
        template.scoped_slots as usize,
    );
    input.template_max_nesting = input
        .template_max_nesting
        .max(template.max_nesting as usize);
}

fn complexity_input(
    registry: &ModuleRegistry,
    templates: &TemplateFacts,
    effect_graphs: &FxHashMap<FileId, EffectGraphSummary>,
    result: &CrossFileResult,
) -> ComplexityInput {
    let mut input = ComplexityInput {
        fallthrough_risk_count: fallthrough_risk_count(result),
        global_state_reference_count: result
            .reactivity_issues
            .iter()
            .filter(|issue| {
                matches!(
                    issue.kind,
                    super::ReactivityIssueKind::ShouldUseStoreToRefs { .. }
                )
            })
            .count()
            .saturating_add(
                result
                    .cross_file_reactivity_issues
                    .iter()
                    .filter(|issue| {
                        matches!(
                            issue.kind,
                            CrossFileReactivityIssueKind::StoreDestructured { .. }
                        )
                    })
                    .count(),
            ),
        ..ComplexityInput::default()
    };

    if let Some(summary) = result.provide_inject_tree_summary {
        input.provide_inject_max_depth = summary.max_depth;
        input.provide_inject_reference_count =
            summary.provide_count.saturating_add(summary.inject_count);
        let fanout = summary
            .max_child_fanout
            .max(summary.max_provider_consumer_count);
        input.provide_inject_fanout_count = fanout;
    } else {
        input.provide_inject_reference_count = result.provide_inject_matches.len();
    }

    for entry in registry.vue_components() {
        let analysis = &entry.analysis;
        add_count(&mut input.component_count, 1);
        if let Some(template) = templates.get(&entry.id) {
            add_template_facts(&mut input, template);
        }
        add_count(&mut input.slot_count, analysis.macros.slots().len());
        add_count(
            &mut input.slot_count,
            vize_croquis::facts::component_usage_list(analysis)
                .iter()
                .map(|usage| usage.slots.len())
                .fold(0usize, usize::saturating_add),
        );
        add_count(
            &mut input.prop_drilling_edge_count,
            vize_croquis::facts::component_usage_list(analysis)
                .iter()
                .map(|usage| usage.props.len())
                .fold(0usize, usize::saturating_add),
        );
        add_count(
            &mut input.reactive_node_count,
            vize_croquis::facts::reactivity_count(analysis),
        );
        let effect_graph = effect_graphs.get(&entry.id).copied().unwrap_or_default();
        add_count(&mut input.reactive_edge_count, effect_graph.edge_count);
        add_count(&mut input.reactive_cycle_count, effect_graph.cycle_count);
    }

    input
}

fn weighted(count: usize, weight: u32) -> u32 {
    u32::try_from(count)
        .unwrap_or(u32::MAX)
        .saturating_mul(weight)
}

fn provide_inject_fanout_score(count: usize) -> u32 {
    weighted(count.saturating_sub(1), 2)
}

fn cyclomatic_score(input: ComplexityInput) -> u32 {
    weighted(input.template_cyclomatic, 1)
}

fn cognitive_score(input: ComplexityInput) -> u32 {
    weighted(input.template_cognitive, 1)
}
