//! Fallthrough attribute analysis.
//!
//! Detects issues with attribute inheritance across component boundaries:
//! - Attributes passed to component but not used
//! - `inheritAttrs: false` without explicit $attrs binding
//! - Multiple root elements without explicit $attrs

use crate::diagnostics::{CrossFileDiagnostic, CrossFileDiagnosticKind, DiagnosticSeverity};
use crate::graph::DependencyGraph;
use crate::registry::{FileId, ModuleRegistry};
use vize_carton::{CompactString, FxHashMap, FxHashSet, camelize, cstr};

mod component;
mod info;
mod summary;
mod usage;

pub use component::{FallthroughComponentFact, collect_fallthrough_component_facts};
pub use info::FallthroughInfo;
pub use summary::{FallthroughSummary, summarize_fallthrough};
pub use usage::*;

/// Analyze fallthrough attributes across the component graph.
pub fn analyze_fallthrough(
    registry: &ModuleRegistry,
    graph: &DependencyGraph,
) -> (Vec<FallthroughInfo>, Vec<CrossFileDiagnostic>) {
    let mut infos = Vec::new();
    let mut diagnostics = Vec::new();

    // First pass: collect information from each component
    for entry in registry.vue_components() {
        let analysis = &entry.analysis;

        // Use precise template_info from static analysis
        let template_info = &analysis.template_info;

        // Check for inheritAttrs option (from defineOptions macro)
        let inherit_attrs_disabled = check_inherit_attrs_disabled(analysis);

        // Get declared props
        let declared_props: FxHashSet<_> = analysis
            .macros
            .props()
            .iter()
            .map(|p| p.name.clone())
            .collect();
        let declared_events: FxHashSet<_> = analysis
            .macros
            .emits()
            .iter()
            .map(|event| event.name.clone())
            .collect();

        let info = FallthroughInfo {
            file_id: entry.id,
            inherit_attrs_disabled,
            uses_attrs: template_info.uses_attrs,
            binds_attrs: template_info.binds_attrs_explicitly,
            root_element_count: template_info.root_element_count,
            passed_attrs: FxHashSet::default(), // Will be filled later
            fallthrough_attrs: FxHashSet::default(), // Will be filled later
            dynamic_name_fallthrough_attrs: FxHashSet::default(),
            declared_props,
            declared_events,
            template_start: template_info.content_start,
            template_end: template_info.content_end,
        };

        infos.push(info);
    }

    // Second pass: track attribute passing through each component usage.
    let usage_facts = collect_fallthrough_usage_facts(registry, graph);
    let mut passed_attrs_map: FxHashMap<FileId, FxHashMap<FileId, FxHashSet<CompactString>>> =
        FxHashMap::default();
    let mut fallthrough_attrs_map: FxHashMap<FileId, FxHashSet<CompactString>> =
        FxHashMap::default();
    let mut dynamic_name_attrs_map: FxHashMap<FileId, FxHashSet<CompactString>> =
        FxHashMap::default();
    let mut fallthrough_related_map: FxHashMap<FileId, Vec<FallthroughUsageRelated>> =
        FxHashMap::default();
    for fact in &usage_facts {
        let attrs = passed_attrs_map
            .entry(fact.child_file_id)
            .or_default()
            .entry(fact.parent_file_id)
            .or_default();
        attrs.extend(fact.attrs.iter().map(|attr| attr.name.clone()));
        fallthrough_attrs_map
            .entry(fact.child_file_id)
            .or_default()
            .extend(
                fact.attrs
                    .iter()
                    .filter(|attr| attr.fallthrough)
                    .map(|attr| attr.name.clone()),
            );
        dynamic_name_attrs_map
            .entry(fact.child_file_id)
            .or_default()
            .extend(
                fact.attrs
                    .iter()
                    .filter(|attr| attr.fallthrough && attr.name_is_dynamic)
                    .map(|attr| attr.name.clone()),
            );

        let related = fallthrough_related_map
            .entry(fact.child_file_id)
            .or_default();
        related.extend(
            fact.attrs
                .iter()
                .filter(|attr| attr.fallthrough)
                .map(|attr| FallthroughUsageRelated {
                    parent_file_id: fact.parent_file_id,
                    attr_name: attr.name.clone(),
                    source_start: attr.source_start,
                    component_name: fact.component_name.clone(),
                }),
        );
        if fact.has_spread_attrs {
            related.push(FallthroughUsageRelated {
                parent_file_id: fact.parent_file_id,
                attr_name: cstr!("v-bind spread"),
                source_start: fact.usage_start,
                component_name: fact.component_name.clone(),
            });
        }
    }

    // Merge passed attrs into infos
    for info in &mut infos {
        if let Some(parent_attrs) = passed_attrs_map.get(&info.file_id) {
            for attrs in parent_attrs.values() {
                info.passed_attrs.extend(attrs.iter().cloned());
            }
        }
        if let Some(attrs) = fallthrough_attrs_map.get(&info.file_id) {
            info.fallthrough_attrs.extend(attrs.iter().cloned());
        }
        if let Some(attrs) = dynamic_name_attrs_map.get(&info.file_id) {
            info.dynamic_name_fallthrough_attrs
                .extend(attrs.iter().cloned());
        }
    }

    // Generate diagnostics
    for info in &infos {
        // Check for multiple root elements without explicit $attrs binding
        if info.root_element_count > 1 && !info.binds_attrs {
            let has_fallthrough = fallthrough_related_map
                .get(&info.file_id)
                .is_some_and(|related| !related.is_empty());

            if has_fallthrough {
                // Use offset 0 to point to <template> tag start (wasm.rs adds tag_start offset)
                diagnostics.push(with_fallthrough_relateds(
                    CrossFileDiagnostic::with_span(
                        CrossFileDiagnosticKind::MultiRootMissingAttrs,
                        DiagnosticSeverity::Warning,
                        info.file_id,
                        0,
                        info.template_end - info.template_start,
                        "Component has multiple root elements but $attrs is not explicitly bound",
                    )
                    .with_suggestion(
                        "Add v-bind=\"$attrs\" to the intended root element or wrap in single root",
                    ),
                    fallthrough_related_map
                        .get(&info.file_id)
                        .map(Vec::as_slice),
                    None,
                ));
            }
        }

        // Check for inheritAttrs: false without $attrs usage
        if info.inherit_attrs_disabled && !info.uses_attrs && !info.binds_attrs {
            // Use offset 0 to point to <template> tag start (wasm.rs adds tag_start offset)
            diagnostics.push(with_fallthrough_relateds(
                CrossFileDiagnostic::with_span(
                    CrossFileDiagnosticKind::InheritAttrsDisabledUnused,
                    DiagnosticSeverity::Warning,
                    info.file_id,
                    0,
                    info.template_end - info.template_start,
                    "inheritAttrs is disabled but $attrs is not used anywhere",
                )
                .with_suggestion("Use v-bind=\"$attrs\" or $attrs.class/$attrs.style in template"),
                fallthrough_related_map
                    .get(&info.file_id)
                    .map(Vec::as_slice),
                None,
            ));
        }

        // Check for unused fallthrough attributes
        let mut unused_attrs: Vec<_> = info
            .fallthrough_attrs
            .iter()
            .filter(|attr| {
                !info.uses_attrs
                    && (info.dynamic_name_fallthrough_attrs.contains(*attr)
                        || !is_standard_html_attr(attr))
            })
            .cloned()
            .collect();
        unused_attrs.sort_unstable();

        if !unused_attrs.is_empty() && !info.binds_attrs && info.root_element_count > 1 {
            // Use offset 0 to point to <template> tag start (wasm.rs adds tag_start offset)
            diagnostics.push(with_fallthrough_relateds(
                CrossFileDiagnostic::with_span(
                    CrossFileDiagnosticKind::UnusedFallthroughAttrs {
                        passed_attrs: unused_attrs.clone(),
                    },
                    DiagnosticSeverity::Info,
                    info.file_id,
                    0,
                    info.template_end - info.template_start,
                    cstr!(
                        "Attributes {:?} are passed but not used (component has multiple roots)",
                        unused_attrs
                    ),
                )
                .with_suggestion("Bind $attrs explicitly or declare as props"),
                fallthrough_related_map
                    .get(&info.file_id)
                    .map(Vec::as_slice),
                Some(&unused_attrs),
            ));
        }
    }

    (infos, diagnostics)
}

/// Check if inheritAttrs: false is set in the component options.
fn check_inherit_attrs_disabled(analysis: &vize_croquis::Croquis) -> bool {
    // Look for defineOptions with inheritAttrs: false in runtime_args
    analysis.macros.all_calls().iter().any(|call| {
        if call.name != "defineOptions" {
            return false;
        }
        // Check if runtime_args contains "inheritAttrs: false" or "inheritAttrs:false"
        if let Some(ref args) = call.runtime_args {
            args.contains("inheritAttrs") && args.contains("false")
        } else {
            false
        }
    })
}

/// Collect durable per-usage fallthrough facts with parent-side source ranges.
pub fn collect_fallthrough_usage_facts(
    registry: &ModuleRegistry,
    graph: &DependencyGraph,
) -> Vec<FallthroughUsageFact> {
    usage::collect_fallthrough_usage_facts(registry, graph)
}

/// Check if an attribute is a standard HTML attribute.
fn is_standard_html_attr(attr: &str) -> bool {
    if attr.starts_with("data-") || attr.starts_with("aria-") || is_listener_attr(attr) {
        return true;
    }

    matches!(
        attr,
        "class"
            | "style"
            | "id"
            | "key"
            | "ref"
            | "role"
            | "tabindex"
            | "title"
            | "disabled"
            | "hidden"
    )
}

fn is_listener_attr(attr: &str) -> bool {
    attr.strip_prefix("on")
        .and_then(|suffix| suffix.chars().next())
        .is_some_and(|first| first.is_ascii_uppercase())
}

fn is_declared_event(declared_events: &FxHashSet<CompactString>, event_name: &str) -> bool {
    let normalized = camelize(event_name);
    declared_events.iter().any(|declared| {
        declared.as_str() == event_name || camelize(declared.as_str()) == normalized
    })
}

struct FallthroughUsageRelated {
    parent_file_id: FileId,
    attr_name: CompactString,
    source_start: u32,
    component_name: CompactString,
}

fn with_fallthrough_relateds(
    mut diagnostic: CrossFileDiagnostic,
    relateds: Option<&[FallthroughUsageRelated]>,
    attrs_filter: Option<&[CompactString]>,
) -> CrossFileDiagnostic {
    let Some(relateds) = relateds else {
        return diagnostic;
    };

    for related in relateds {
        if attrs_filter.is_some_and(|attrs| {
            !attrs
                .iter()
                .any(|attr| attr.as_str() == related.attr_name.as_str())
        }) {
            continue;
        }

        diagnostic = diagnostic.with_related(
            related.parent_file_id,
            related.source_start,
            cstr!(
                "{} passed to <{}>",
                related.attr_name,
                related.component_name
            ),
        );
    }

    diagnostic
}

#[cfg(test)]
mod diagnostics_tests;

#[cfg(test)]
mod tests;
