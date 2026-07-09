//! Cross-file complexity scoring.
//!
//! The model is intentionally explainable: callers can inspect both raw input
//! counts and weighted dimension scores before deciding how to surface the
//! result in reports or diagnostics.

mod nesting;

use crate::analyzer::CrossFileResult;
use crate::graph::DependencyGraph;
use crate::registry::ModuleRegistry;
use crate::rules::cross_file_reactivity::CrossFileReactivityIssueKind;
use vize_croquis::{ScopeKind, TemplateExpressionKind};

/// Raw cross-file signals used to score Vue component complexity.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ComplexityInput {
    pub component_count: usize,
    pub template_if_count: usize,
    pub template_for_count: usize,
    pub template_logical_operator_count: usize,
    pub component_tree_v_if_max_depth: usize,
    pub component_tree_v_for_max_depth: usize,
    pub component_tree_scoped_slot_max_depth: usize,
    pub component_tree_template_nesting_score: usize,
    pub slot_count: usize,
    pub prop_drilling_edge_count: usize,
    pub global_state_reference_count: usize,
    pub provide_inject_max_depth: usize,
    pub provide_inject_reference_count: usize,
    pub fallthrough_risk_count: usize,
    pub reactive_node_count: usize,
    pub reactive_edge_count: usize,
    pub reactive_cycle_count: usize,
}

/// Per-dimension complexity score.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ComplexityDimensionScores {
    pub template_control_flow: u32,
    pub slot_usage: u32,
    pub prop_drilling: u32,
    pub global_state: u32,
    pub provide_inject: u32,
    pub fallthrough_attrs: u32,
    pub reactive_graph: u32,
}

/// Named complexity dimension for explainable report output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// One scored dimension in a complexity report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            ComplexityDimensionBreakdown {
                dimension: ComplexityDimension::TemplateControlFlow,
                score: self.template_control_flow,
            },
            ComplexityDimensionBreakdown {
                dimension: ComplexityDimension::SlotUsage,
                score: self.slot_usage,
            },
            ComplexityDimensionBreakdown {
                dimension: ComplexityDimension::PropDrilling,
                score: self.prop_drilling,
            },
            ComplexityDimensionBreakdown {
                dimension: ComplexityDimension::GlobalState,
                score: self.global_state,
            },
            ComplexityDimensionBreakdown {
                dimension: ComplexityDimension::ProvideInject,
                score: self.provide_inject,
            },
            ComplexityDimensionBreakdown {
                dimension: ComplexityDimension::FallthroughAttrs,
                score: self.fallthrough_attrs,
            },
            ComplexityDimensionBreakdown {
                dimension: ComplexityDimension::ReactiveGraph,
                score: self.reactive_graph,
            },
        ]
    }

    pub fn dominant_dimension(self) -> Option<ComplexityDimensionBreakdown> {
        self.breakdown()
            .into_iter()
            .filter(|breakdown| breakdown.score > 0)
            .max_by_key(|breakdown| breakdown.score)
    }
}

/// Human-facing band for the total complexity score.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ComplexityBand {
    #[default]
    Low,
    Moderate,
    High,
    Extreme,
}

/// Explainable complexity score for one cross-file graph or component slice.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
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
            slot_usage: weighted(input.slot_count, 2).saturating_add(weighted(
                input.component_tree_scoped_slot_max_depth.saturating_sub(1),
                4,
            )),
            prop_drilling: weighted(input.prop_drilling_edge_count, 3),
            global_state: weighted(input.global_state_reference_count, 2),
            provide_inject: weighted(input.provide_inject_max_depth.saturating_sub(1), 2)
                .saturating_add(weighted(input.provide_inject_reference_count, 1)),
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

/// Summarize complexity from the analyzer's existing cross-file facts.
#[cfg(test)]
pub(super) fn summarize_complexity(
    registry: &ModuleRegistry,
    result: &CrossFileResult,
) -> ComplexityReport {
    ComplexityReport::from_input(complexity_input(registry, result))
}

/// Summarize complexity with component-tree template nesting signals.
pub fn summarize_complexity_with_graph(
    registry: &ModuleRegistry,
    graph: &DependencyGraph,
    result: &CrossFileResult,
) -> ComplexityReport {
    let mut input = complexity_input(registry, result);
    nesting::add_component_tree_template_nesting(&mut input, registry, graph);
    ComplexityReport::from_input(input)
}

fn complexity_input(registry: &ModuleRegistry, result: &CrossFileResult) -> ComplexityInput {
    let mut input = ComplexityInput {
        fallthrough_risk_count: fallthrough_risk_count(result),
        reactive_edge_count: result
            .reactivity_issues
            .len()
            .saturating_add(result.cross_file_reactivity_issues.len())
            .saturating_add(result.provide_inject_matches.len()),
        reactive_cycle_count: result
            .cross_file_reactivity_issues
            .iter()
            .filter(|issue| {
                matches!(
                    issue.kind,
                    CrossFileReactivityIssueKind::CircularReactiveDependency { .. }
                )
            })
            .count(),
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
    } else {
        input.provide_inject_reference_count = result.provide_inject_matches.len();
    }

    for entry in registry.vue_components() {
        let analysis = &entry.analysis;
        input.component_count = input.component_count.saturating_add(1);

        input.template_if_count += analysis
            .template_expressions
            .iter()
            .filter(|expr| expr.kind == TemplateExpressionKind::VIf)
            .count();
        input.template_logical_operator_count += analysis
            .template_expressions
            .iter()
            .filter(|expr| expr.kind == TemplateExpressionKind::VIf)
            .map(|expr| logical_operator_count(expr.content.as_str()))
            .sum::<usize>();
        input.template_for_count += analysis
            .scopes
            .iter()
            .filter(|scope| scope.kind == ScopeKind::VFor)
            .count();
        input.slot_count += analysis.macros.slots().len();
        input.slot_count += analysis
            .component_usages
            .iter()
            .map(|usage| usage.slots.len())
            .sum::<usize>();
        input.prop_drilling_edge_count += analysis
            .component_usages
            .iter()
            .map(|usage| usage.props.len())
            .sum::<usize>();
        input.reactive_node_count += analysis.reactivity.count();
    }

    input
}

fn fallthrough_risk_count(result: &CrossFileResult) -> usize {
    if let Some(summary) = result.fallthrough_summary {
        return summary
            .components_with_potential_issues
            .saturating_add(summary.risky_unconsumed_fallthrough_attr_count);
    }

    result
        .fallthrough_info
        .iter()
        .filter(|info| info.has_potential_issues())
        .count()
}

fn weighted(count: usize, weight: u32) -> u32 {
    u32::try_from(count)
        .unwrap_or(u32::MAX)
        .saturating_mul(weight)
}

fn cyclomatic_score(input: ComplexityInput) -> u32 {
    weighted(
        input
            .component_count
            .saturating_add(input.template_if_count)
            .saturating_add(input.template_for_count)
            .saturating_add(input.template_logical_operator_count),
        1,
    )
}

fn cognitive_score(input: ComplexityInput) -> u32 {
    weighted(input.component_tree_template_nesting_score, 1)
}

fn logical_operator_count(content: &str) -> usize {
    content
        .matches("&&")
        .count()
        .saturating_add(content.matches("||").count())
}
