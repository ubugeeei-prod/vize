//! Stable overlay model for parser-driven reactivity analysis.
//!
//! The core reactivity tracker stores compact Rust facts. This module exposes
//! those facts as a deterministic, serializable contract for diagnostics,
//! reports, editor overlays, and Playground rendering.

use crate::effect_graph::{EffectGraph, EffectNodeId};
use crate::reactivity::{ReactiveKind, ReactivityLoss, ReactivityLossKind, ReactivityTracker};
use serde::Serialize;
use vize_carton::CompactString;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactivityOverlay {
    pub summary: ReactivityOverlaySummary,
    pub sources: Vec<ReactivitySourceOverlay>,
    pub losses: Vec<ReactivityLossOverlay>,
    pub effect_graph: ReactivityEffectGraphOverlay,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactivityOverlaySummary {
    pub source_count: usize,
    pub ref_source_count: usize,
    pub reactive_source_count: usize,
    pub computed_source_count: usize,
    pub readonly_source_count: usize,
    pub needs_value_access_count: usize,
    pub loss_count: usize,
    pub effect_edge_count: usize,
    pub effect_cycle_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactivitySourceOverlay {
    pub id: u32,
    pub name: CompactString,
    pub kind: &'static str,
    pub category: &'static str,
    pub needs_value_access: bool,
    pub declaration_offset: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactivityLossOverlay {
    pub kind: &'static str,
    pub category: &'static str,
    pub source_name: Option<CompactString>,
    pub target_name: Option<CompactString>,
    pub property_name: Option<CompactString>,
    pub argument_name: Option<CompactString>,
    pub callee_name: Option<CompactString>,
    pub getter_name: Option<CompactString>,
    pub alias_name: Option<CompactString>,
    pub extracted_props: Vec<CompactString>,
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactivityEffectGraphOverlay {
    pub edges: Vec<ReactivityEffectEdgeOverlay>,
    pub cycle: Option<Vec<EffectNodeId>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactivityEffectEdgeOverlay {
    pub from: EffectNodeId,
    pub to: EffectNodeId,
    pub category: &'static str,
}

impl ReactivityOverlay {
    pub fn from_tracker(tracker: &ReactivityTracker) -> Self {
        Self::from_tracker_and_effect_graph(tracker, None)
    }

    pub fn from_tracker_and_effect_graph(
        tracker: &ReactivityTracker,
        effect_graph: Option<&EffectGraph>,
    ) -> Self {
        let mut sources: Vec<_> = tracker
            .sources()
            .iter()
            .map(ReactivitySourceOverlay::from_source)
            .collect();
        sources.sort_by_key(|source| source.id);

        let mut losses: Vec<_> = tracker
            .losses()
            .iter()
            .map(ReactivityLossOverlay::from_loss)
            .collect();
        losses.sort_by_key(|loss| (loss.start, loss.end, loss.kind));

        let effect_graph = effect_graph
            .map(ReactivityEffectGraphOverlay::from_effect_graph)
            .unwrap_or_default();

        let mut summary = ReactivityOverlaySummary {
            source_count: sources.len(),
            loss_count: losses.len(),
            effect_edge_count: effect_graph.edges.len(),
            effect_cycle_count: usize::from(effect_graph.cycle.is_some()),
            ..ReactivityOverlaySummary::default()
        };

        for source in &sources {
            match source.category {
                "ref" => summary.ref_source_count += 1,
                "reactive" => summary.reactive_source_count += 1,
                "computed" => summary.computed_source_count += 1,
                "readonly" => summary.readonly_source_count += 1,
                _ => {}
            }
            if source.needs_value_access {
                summary.needs_value_access_count += 1;
            }
        }

        Self {
            summary,
            sources,
            losses,
            effect_graph,
        }
    }
}

impl ReactivityTracker {
    pub fn overlay(&self) -> ReactivityOverlay {
        ReactivityOverlay::from_tracker(self)
    }

    pub fn overlay_with_effect_graph(&self, effect_graph: &EffectGraph) -> ReactivityOverlay {
        ReactivityOverlay::from_tracker_and_effect_graph(self, Some(effect_graph))
    }
}

impl EffectGraph {
    pub fn overlay(&self) -> ReactivityEffectGraphOverlay {
        ReactivityEffectGraphOverlay::from_effect_graph(self)
    }
}

impl ReactivitySourceOverlay {
    fn from_source(source: &crate::reactivity::ReactiveSource) -> Self {
        Self {
            id: source.id.as_u32(),
            name: source.name.clone(),
            kind: reactive_kind_name(source.kind),
            category: reactive_kind_category(source.kind),
            needs_value_access: source.kind.needs_value_access(),
            declaration_offset: source.declaration_offset,
        }
    }
}

impl ReactivityLossOverlay {
    fn from_loss(loss: &ReactivityLoss) -> Self {
        let mut overlay = Self {
            kind: reactivity_loss_kind_name(&loss.kind),
            category: "loss",
            source_name: None,
            target_name: None,
            property_name: None,
            argument_name: None,
            callee_name: None,
            getter_name: None,
            alias_name: None,
            extracted_props: Vec::new(),
            start: loss.start,
            end: loss.end,
        };

        match &loss.kind {
            ReactivityLossKind::ReactiveDestructure {
                source_name,
                destructured_props,
            }
            | ReactivityLossKind::RefValueDestructure {
                source_name,
                destructured_props,
            } => {
                overlay.source_name = Some(source_name.clone());
                overlay.extracted_props = destructured_props.clone();
            }
            ReactivityLossKind::RefValueExtract {
                source_name,
                target_name,
            } => {
                overlay.source_name = Some(source_name.clone());
                overlay.target_name = Some(target_name.clone());
            }
            ReactivityLossKind::ReactivePropertyExtract {
                source_name,
                prop_name,
                target_name,
            } => {
                overlay.source_name = Some(source_name.clone());
                overlay.property_name = Some(prop_name.clone());
                overlay.target_name = Some(target_name.clone());
            }
            ReactivityLossKind::PropsDestructure { destructured_props } => {
                overlay.extracted_props = destructured_props.clone();
            }
            ReactivityLossKind::FunctionArgumentExtract {
                source_name,
                argument_name,
                callee_name,
            } => {
                overlay.source_name = Some(source_name.clone());
                overlay.argument_name = Some(argument_name.clone());
                overlay.callee_name = Some(callee_name.clone());
            }
            ReactivityLossKind::GetterCallExtract {
                context_name,
                getter_name,
                target_name,
                callee_name,
                source_name,
            } => {
                overlay.source_name = Some(source_name.clone());
                overlay.argument_name = Some(context_name.clone());
                overlay.getter_name = Some(getter_name.clone());
                overlay.target_name = Some(target_name.clone());
                overlay.callee_name = Some(callee_name.clone());
            }
            ReactivityLossKind::PlainValueAlias {
                source_name,
                alias_name,
                target_name,
            } => {
                overlay.source_name = Some(source_name.clone());
                overlay.alias_name = Some(alias_name.clone());
                overlay.target_name = Some(target_name.clone());
            }
            ReactivityLossKind::ReactiveSpread { source_name }
            | ReactivityLossKind::ReactiveReassign { source_name } => {
                overlay.source_name = Some(source_name.clone());
            }
        }

        overlay
    }
}

impl ReactivityEffectGraphOverlay {
    fn from_effect_graph(effect_graph: &EffectGraph) -> Self {
        let mut edges: Vec<_> = effect_graph
            .edges()
            .map(|edge| ReactivityEffectEdgeOverlay {
                from: edge.from.clone(),
                to: edge.to.clone(),
                category: "effectEdge",
            })
            .collect();
        edges.sort_by(|left, right| left.from.cmp(&right.from).then(left.to.cmp(&right.to)));

        Self {
            edges,
            cycle: effect_graph.find_cycle(),
        }
    }
}

const fn reactive_kind_name(kind: ReactiveKind) -> &'static str {
    match kind {
        ReactiveKind::Ref => "ref",
        ReactiveKind::ShallowRef => "shallowRef",
        ReactiveKind::Reactive => "reactive",
        ReactiveKind::ShallowReactive => "shallowReactive",
        ReactiveKind::Computed => "computed",
        ReactiveKind::Readonly => "readonly",
        ReactiveKind::ShallowReadonly => "shallowReadonly",
        ReactiveKind::ToRef => "toRef",
        ReactiveKind::ToRefs => "toRefs",
    }
}

const fn reactive_kind_category(kind: ReactiveKind) -> &'static str {
    match kind {
        ReactiveKind::Ref
        | ReactiveKind::ShallowRef
        | ReactiveKind::ToRef
        | ReactiveKind::ToRefs => "ref",
        ReactiveKind::Reactive | ReactiveKind::ShallowReactive => "reactive",
        ReactiveKind::Computed => "computed",
        ReactiveKind::Readonly | ReactiveKind::ShallowReadonly => "readonly",
    }
}

const fn reactivity_loss_kind_name(kind: &ReactivityLossKind) -> &'static str {
    match kind {
        ReactivityLossKind::ReactiveDestructure { .. } => "reactiveDestructure",
        ReactivityLossKind::RefValueDestructure { .. } => "refValueDestructure",
        ReactivityLossKind::RefValueExtract { .. } => "refValueExtract",
        ReactivityLossKind::ReactivePropertyExtract { .. } => "reactivePropertyExtract",
        ReactivityLossKind::PropsDestructure { .. } => "propsDestructure",
        ReactivityLossKind::FunctionArgumentExtract { .. } => "functionArgumentExtract",
        ReactivityLossKind::GetterCallExtract { .. } => "getterCallExtract",
        ReactivityLossKind::PlainValueAlias { .. } => "plainValueAlias",
        ReactivityLossKind::ReactiveSpread { .. } => "reactiveSpread",
        ReactivityLossKind::ReactiveReassign { .. } => "reactiveReassign",
    }
}
