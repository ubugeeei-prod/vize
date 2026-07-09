use super::{ComplexityDimensionBreakdown, ComplexityDimensionScores, ComplexityInput};
use super::{ComplexityReport, CrossFileReactivityIssueKind};
use crate::analyzer::CrossFileResult;
use crate::registry::{FileId, ModuleEntry, ModuleRegistry};
use crate::rules::ReactivityIssueKind;
use vize_carton::{CompactString, FxHashMap};
use vize_croquis::{EffectGraphSummary, ScopeKind, TemplateExpressionKind};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComplexityHotspot {
    pub file_id: FileId,
    pub file_name: CompactString,
    pub component_name: Option<CompactString>,
    pub input: ComplexityInput,
    pub dimensions: ComplexityDimensionScores,
    pub total_score: u32,
    pub dominant_dimension: Option<ComplexityDimensionBreakdown>,
}

pub(crate) fn summarize_complexity_hotspots_with_effect_graphs(
    registry: &ModuleRegistry,
    effect_graphs: &FxHashMap<FileId, EffectGraphSummary>,
    result: &CrossFileResult,
) -> Vec<ComplexityHotspot> {
    let mut inputs = local_inputs(registry, effect_graphs);
    add_fallthrough_inputs(&mut inputs, result);
    add_reactivity_inputs(&mut inputs, result);
    add_provide_inject_inputs(&mut inputs, result);

    let mut hotspots: Vec<_> = registry
        .vue_components()
        .filter_map(|entry| hotspot_for_entry(entry, inputs.remove(&entry.id).unwrap_or_default()))
        .collect();
    hotspots.sort_by(|a, b| {
        b.total_score
            .cmp(&a.total_score)
            .then_with(|| a.file_name.cmp(&b.file_name))
    });
    hotspots
}

fn local_inputs(
    registry: &ModuleRegistry,
    effect_graphs: &FxHashMap<FileId, EffectGraphSummary>,
) -> FxHashMap<FileId, ComplexityInput> {
    registry
        .vue_components()
        .map(|entry| {
            let effect_graph = effect_graphs.get(&entry.id).copied().unwrap_or_default();
            (entry.id, local_input(entry, effect_graph))
        })
        .collect()
}

fn local_input(entry: &ModuleEntry, effect_graph: EffectGraphSummary) -> ComplexityInput {
    let analysis = &entry.analysis;

    ComplexityInput {
        component_count: 1,
        template_if_count: analysis
            .template_expressions
            .iter()
            .filter(|expr| expr.kind == TemplateExpressionKind::VIf)
            .count(),
        template_for_count: analysis
            .scopes
            .iter()
            .filter(|scope| scope.kind == ScopeKind::VFor)
            .count(),
        template_logical_operator_count: analysis
            .template_expressions
            .iter()
            .filter(|expr| expr.kind == TemplateExpressionKind::VIf)
            .map(|expr| logical_operator_count(expr.content.as_str()))
            .sum(),
        slot_count: analysis.macros.slots().len()
            + analysis
                .component_usages
                .iter()
                .map(|usage| usage.slots.len())
                .sum::<usize>(),
        prop_drilling_edge_count: analysis
            .component_usages
            .iter()
            .map(|usage| usage.props.len())
            .sum(),
        reactive_node_count: analysis.reactivity.count(),
        reactive_edge_count: effect_graph.edge_count,
        reactive_cycle_count: effect_graph.cycle_count,
        ..ComplexityInput::default()
    }
}

fn add_fallthrough_inputs(
    inputs: &mut FxHashMap<FileId, ComplexityInput>,
    result: &CrossFileResult,
) {
    for info in &result.fallthrough_info {
        inputs
            .entry(info.file_id)
            .or_default()
            .fallthrough_risk_count += usize::from(info.has_potential_issues())
            .saturating_add(info.risky_unconsumed_fallthrough_attr_count());
    }
}

fn add_reactivity_inputs(
    inputs: &mut FxHashMap<FileId, ComplexityInput>,
    result: &CrossFileResult,
) {
    for issue in &result.reactivity_issues {
        let input = inputs.entry(issue.file_id).or_default();
        if matches!(issue.kind, ReactivityIssueKind::ShouldUseStoreToRefs { .. }) {
            input.global_state_reference_count += 1;
        }
    }

    for issue in &result.cross_file_reactivity_issues {
        let input = inputs.entry(issue.file_id).or_default();
        if let CrossFileReactivityIssueKind::StoreDestructured { .. } = issue.kind {
            input.global_state_reference_count += 1;
        }
    }
}

fn add_provide_inject_inputs(
    inputs: &mut FxHashMap<FileId, ComplexityInput>,
    result: &CrossFileResult,
) {
    for matched in &result.provide_inject_matches {
        let provider = inputs.entry(matched.provider).or_default();
        provider.provide_inject_reference_count += 1;
        provider.provide_inject_fanout_count += 1;

        // A dependency injection edge is not inherently a reactive edge.
        let consumer = inputs.entry(matched.consumer).or_default();
        consumer.provide_inject_reference_count += 1;

        let depth = matched.path.len();
        for file_id in &matched.path {
            let input = inputs.entry(*file_id).or_default();
            input.provide_inject_max_depth = input.provide_inject_max_depth.max(depth);
        }
    }
}

fn hotspot_for_entry(entry: &ModuleEntry, input: ComplexityInput) -> Option<ComplexityHotspot> {
    let report = ComplexityReport::from_input(input);
    (report.total_score > 0).then(|| ComplexityHotspot {
        file_id: entry.id,
        file_name: entry.filename.clone(),
        component_name: entry.component_name.clone(),
        input,
        dimensions: report.dimensions,
        total_score: report.total_score,
        dominant_dimension: report.dominant_dimension(),
    })
}

fn logical_operator_count(content: &str) -> usize {
    content
        .matches("&&")
        .count()
        .saturating_add(content.matches("||").count())
}
