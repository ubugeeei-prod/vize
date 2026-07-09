use crate::graph::{DependencyEdge, DependencyGraph};
use crate::registry::{FileId, ModuleRegistry};
use serde::Serialize;
use vize_carton::{CompactString, FxHashSet, camelize, cstr};

use super::{is_declared_event, is_standard_html_attr};

/// One parent template usage of a child component and the attrs it passes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FallthroughUsageFact {
    pub parent_file_id: FileId,
    pub child_file_id: FileId,
    pub component_name: CompactString,
    pub usage_start: u32,
    pub usage_end: u32,
    pub has_spread_attrs: bool,
    pub attrs: Vec<FallthroughUsageAttrFact>,
}

/// One passed attr/listener on a component usage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FallthroughUsageAttrFact {
    pub name: CompactString,
    pub kind: FallthroughUsageAttrKind,
    pub source_start: u32,
    pub source_end: u32,
    pub dynamic: bool,
    pub declared_prop: bool,
    pub declared_event: bool,
    pub standard_html_attr: bool,
    pub fallthrough: bool,
}

/// Source category of a usage attr.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FallthroughUsageAttrKind {
    Prop,
    Listener,
}

pub(super) fn collect_fallthrough_usage_facts(
    registry: &ModuleRegistry,
    graph: &DependencyGraph,
) -> Vec<FallthroughUsageFact> {
    let mut facts = Vec::new();

    for node in graph.nodes() {
        for (child_id, edge_type) in &node.imports {
            if *edge_type != DependencyEdge::ComponentUsage {
                continue;
            }
            let Some(parent_entry) = registry.get(node.file_id) else {
                continue;
            };
            let declared_props = declared_props_for(registry, *child_id);
            let declared_events = declared_events_for(registry, *child_id);

            for usage in &parent_entry.analysis.component_usages {
                if graph.find_by_component(usage.name.as_str()) != Some(*child_id) {
                    continue;
                }

                let mut attrs = Vec::with_capacity(usage.props.len() + usage.events.len());
                attrs.extend(usage.props.iter().map(|prop| {
                    let name = prop.name.clone();
                    usage_attr_fact(
                        name,
                        FallthroughUsageAttrKind::Prop,
                        prop.start,
                        prop.end,
                        prop.is_dynamic,
                        &declared_props,
                        false,
                    )
                }));
                attrs.extend(usage.events.iter().map(|event| {
                    let declared_event = is_declared_event(&declared_events, event.name.as_str());
                    usage_attr_fact(
                        listener_attr_name(event.name.as_str()),
                        FallthroughUsageAttrKind::Listener,
                        event.start,
                        event.end,
                        true,
                        &declared_props,
                        declared_event,
                    )
                }));

                facts.push(FallthroughUsageFact {
                    parent_file_id: node.file_id,
                    child_file_id: *child_id,
                    component_name: usage.name.clone(),
                    usage_start: usage.start,
                    usage_end: usage.end,
                    has_spread_attrs: usage.has_spread_attrs,
                    attrs,
                });
            }
        }
    }

    facts.sort_by_key(|fact| {
        (
            fact.child_file_id.as_u32(),
            fact.parent_file_id.as_u32(),
            fact.usage_start,
            fact.usage_end,
        )
    });
    facts
}

fn usage_attr_fact(
    name: CompactString,
    kind: FallthroughUsageAttrKind,
    source_start: u32,
    source_end: u32,
    dynamic: bool,
    declared_props: &FxHashSet<CompactString>,
    declared_event: bool,
) -> FallthroughUsageAttrFact {
    let declared_prop = declared_props.contains(&name);
    FallthroughUsageAttrFact {
        standard_html_attr: is_standard_html_attr(name.as_str()),
        fallthrough: !declared_prop && !declared_event,
        name,
        kind,
        source_start,
        source_end,
        dynamic,
        declared_prop,
        declared_event,
    }
}

fn declared_props_for(registry: &ModuleRegistry, child_id: FileId) -> FxHashSet<CompactString> {
    registry
        .get(child_id)
        .map(|entry| {
            entry
                .analysis
                .macros
                .props()
                .iter()
                .map(|prop| prop.name.clone())
                .collect()
        })
        .unwrap_or_default()
}

fn declared_events_for(registry: &ModuleRegistry, child_id: FileId) -> FxHashSet<CompactString> {
    registry
        .get(child_id)
        .map(|entry| {
            entry
                .analysis
                .macros
                .emits()
                .iter()
                .map(|event| event.name.clone())
                .collect()
        })
        .unwrap_or_default()
}

fn listener_attr_name(event_name: &str) -> CompactString {
    let event_name = camelize(event_name);
    let mut chars = event_name.chars();
    let Some(first) = chars.next() else {
        return cstr!("on");
    };
    cstr!("on{}{}", first.to_ascii_uppercase(), chars.as_str())
}
