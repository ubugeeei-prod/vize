//! Fallthrough attribute analysis.
//!
//! Detects issues with attribute inheritance across component boundaries:
//! - Attributes passed to component but not used
//! - `inheritAttrs: false` without explicit $attrs binding
//! - Multiple root elements without explicit $attrs

use crate::diagnostics::{CrossFileDiagnostic, CrossFileDiagnosticKind, DiagnosticSeverity};
use crate::graph::DependencyGraph;
use crate::registry::{FileId, ModuleRegistry};
use vize_carton::{CompactString, FxHashMap, FxHashSet, cstr};

/// Information about fallthrough attributes for a component.
#[derive(Debug, Clone)]
pub struct FallthroughInfo {
    /// File ID of the component.
    pub file_id: FileId,
    /// Whether `inheritAttrs: false` is set.
    pub inherit_attrs_disabled: bool,
    /// Whether $attrs is used in template.
    pub uses_attrs: bool,
    /// Whether $attrs is explicitly bound (v-bind="$attrs").
    pub binds_attrs: bool,
    /// Number of root elements in template.
    pub root_element_count: usize,
    /// Attributes passed by parent components.
    pub passed_attrs: FxHashSet<CompactString>,
    /// Props declared by this component.
    pub declared_props: FxHashSet<CompactString>,
    /// Template content start offset (relative to template block).
    pub template_start: u32,
    /// Template content end offset (relative to template block).
    pub template_end: u32,
}

impl FallthroughInfo {
    /// Check if fallthrough may cause issues.
    pub fn has_potential_issues(&self) -> bool {
        // Multiple roots without explicit $attrs
        if self.root_element_count > 1 && !self.binds_attrs {
            return true;
        }

        // inheritAttrs: false but $attrs not used
        if self.inherit_attrs_disabled && !self.uses_attrs && !self.binds_attrs {
            return true;
        }

        // Attributes passed that aren't props
        let fallthrough_attrs: Vec<_> = self
            .passed_attrs
            .iter()
            .filter(|attr| !self.declared_props.contains(*attr))
            .collect();

        if !fallthrough_attrs.is_empty() && !self.uses_attrs && self.root_element_count > 1 {
            return true;
        }

        false
    }
}

/// Analyze fallthrough attributes across the component graph.
pub fn analyze_fallthrough(
    registry: &ModuleRegistry,
    graph: &DependencyGraph,
) -> (Vec<FallthroughInfo>, Vec<CrossFileDiagnostic>) {
    let mut infos = Vec::new();
    let mut diagnostics = Vec::new();

    // Build a map of what attributes each component passes to its children
    let mut passed_attrs_map: FxHashMap<FileId, FxHashMap<FileId, FxHashSet<CompactString>>> =
        FxHashMap::default();

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

        let info = FallthroughInfo {
            file_id: entry.id,
            inherit_attrs_disabled,
            uses_attrs: template_info.uses_attrs,
            binds_attrs: template_info.binds_attrs_explicitly,
            root_element_count: template_info.root_element_count,
            passed_attrs: FxHashSet::default(), // Will be filled later
            declared_props,
            template_start: template_info.content_start,
            template_end: template_info.content_end,
        };

        infos.push(info);
    }

    // Second pass: track attribute passing through component usage
    for node in graph.nodes() {
        // Look at component usage edges
        for (child_id, edge_type) in &node.imports {
            if *edge_type != crate::graph::DependencyEdge::ComponentUsage {
                continue;
            }

            // Get the parent's analysis to find what attrs are passed
            if let Some(parent_entry) = registry.get(node.file_id) {
                // Only collect the attributes from the usages that actually
                // target this specific child component, so two distinct usage
                // sites of different children are not conflated.
                let attrs = extract_passed_attrs(&parent_entry.analysis, *child_id, graph);

                passed_attrs_map
                    .entry(*child_id)
                    .or_default()
                    .entry(node.file_id)
                    .or_default()
                    .extend(attrs);
            }
        }
    }

    // Merge passed attrs into infos
    for info in &mut infos {
        if let Some(parent_attrs) = passed_attrs_map.get(&info.file_id) {
            for attrs in parent_attrs.values() {
                info.passed_attrs.extend(attrs.iter().cloned());
            }
        }
    }

    // Generate diagnostics
    for info in &infos {
        // Check for multiple root elements without explicit $attrs binding
        if info.root_element_count > 1 && !info.binds_attrs {
            let has_fallthrough = info
                .passed_attrs
                .iter()
                .any(|attr| !info.declared_props.contains(attr));

            if has_fallthrough {
                // Use offset 0 to point to <template> tag start (wasm.rs adds tag_start offset)
                diagnostics.push(
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
                );
            }
        }

        // Check for inheritAttrs: false without $attrs usage
        if info.inherit_attrs_disabled && !info.uses_attrs && !info.binds_attrs {
            // Use offset 0 to point to <template> tag start (wasm.rs adds tag_start offset)
            diagnostics.push(
                CrossFileDiagnostic::with_span(
                    CrossFileDiagnosticKind::InheritAttrsDisabledUnused,
                    DiagnosticSeverity::Warning,
                    info.file_id,
                    0,
                    info.template_end - info.template_start,
                    "inheritAttrs is disabled but $attrs is not used anywhere",
                )
                .with_suggestion("Use v-bind=\"$attrs\" or $attrs.class/$attrs.style in template"),
            );
        }

        // Check for unused fallthrough attributes
        let unused_attrs: Vec<_> = info
            .passed_attrs
            .iter()
            .filter(|attr| {
                !info.declared_props.contains(*attr)
                    && !is_standard_html_attr(attr)
                    && !info.uses_attrs
            })
            .cloned()
            .collect();

        if !unused_attrs.is_empty() && !info.binds_attrs && info.root_element_count > 1 {
            // Use offset 0 to point to <template> tag start (wasm.rs adds tag_start offset)
            diagnostics.push(
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
            );
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

/// Extract attributes passed to a specific child component.
///
/// Uses `component_usages` for precise static analysis. Each usage carries the
/// component name as written in the template; we resolve that name to a
/// `FileId` through the dependency graph and only keep the props from usages
/// that target `child_id`. This ensures distinct usage sites of different
/// children are attributed independently, rather than merging every prop the
/// parent passes to any child onto a single component.
fn extract_passed_attrs(
    analysis: &vize_croquis::Croquis,
    child_id: FileId,
    graph: &DependencyGraph,
) -> FxHashSet<CompactString> {
    let mut attrs = FxHashSet::default();

    for usage in &analysis.component_usages {
        // Resolve the usage's component name to the file it refers to and skip
        // usages that target a different child component.
        if graph.find_by_component(usage.name.as_str()) != Some(child_id) {
            continue;
        }

        for prop in &usage.props {
            attrs.insert(prop.name.clone());
        }
    }

    attrs
}

/// Check if an attribute is a standard HTML attribute.
fn is_standard_html_attr(attr: &str) -> bool {
    matches!(
        attr,
        "class"
            | "style"
            | "id"
            | "key"
            | "ref"
            | "data-*"
            | "aria-*"
            | "role"
            | "tabindex"
            | "title"
            | "disabled"
            | "hidden"
    )
}

#[cfg(test)]
mod tests {
    use super::{FallthroughInfo, analyze_fallthrough};
    use crate::graph::{DependencyEdge, DependencyGraph, ModuleNode};
    use crate::registry::{FileId, ModuleRegistry};
    use vize_carton::{CompactString, FxHashSet, smallvec};
    use vize_croquis::analysis::{ComponentUsage, PassedProp};
    use vize_croquis::{Croquis, ScopeId};

    fn passed_prop(name: &str) -> PassedProp {
        PassedProp {
            name: CompactString::new(name),
            value: None,
            start: 0,
            end: 0,
            is_dynamic: false,
        }
    }

    fn usage_with_prop(name: &str, prop: &str) -> ComponentUsage {
        ComponentUsage {
            name: CompactString::new(name),
            start: 0,
            end: 0,
            props: smallvec![passed_prop(prop)],
            events: smallvec![],
            slots: smallvec![],
            has_spread_attrs: false,
            scope_id: ScopeId::ROOT,
            vif_guard: None,
        }
    }

    fn graph_node(id: FileId, path: &str, component: &str) -> ModuleNode {
        let mut node = ModuleNode::new(id, path);
        node.component_name = Some(CompactString::new(component));
        node
    }

    /// A parent that uses two distinct child components, each receiving a
    /// different prop, must attribute each prop only to the child it was passed
    /// to. Previously `extract_passed_attrs` ignored the child identity and
    /// merged every passed prop onto every child.
    #[test]
    fn passed_attrs_are_attributed_per_child() {
        let mut registry = ModuleRegistry::new();

        let parent_analysis = {
            let mut analysis = Croquis::new();
            analysis
                .component_usages
                .push(usage_with_prop("ChildA", "foo"));
            analysis
                .component_usages
                .push(usage_with_prop("ChildB", "bar"));
            analysis
        };

        let (parent_id, _) = registry.register("Parent.vue", "", parent_analysis);
        let (child_a, _) = registry.register("ChildA.vue", "", Croquis::new());
        let (child_b, _) = registry.register("ChildB.vue", "", Croquis::new());

        let mut graph = DependencyGraph::new();
        graph.add_node(graph_node(parent_id, "Parent.vue", "Parent"));
        graph.add_node(graph_node(child_a, "ChildA.vue", "ChildA"));
        graph.add_node(graph_node(child_b, "ChildB.vue", "ChildB"));
        graph.add_edge(parent_id, child_a, DependencyEdge::ComponentUsage);
        graph.add_edge(parent_id, child_b, DependencyEdge::ComponentUsage);

        let (infos, _diags) = analyze_fallthrough(&registry, &graph);

        let attrs_for = |id: FileId| -> Vec<&str> {
            let info = infos.iter().find(|i| i.file_id == id).unwrap();
            let mut names: Vec<&str> = info.passed_attrs.iter().map(|s| s.as_str()).collect();
            names.sort_unstable();
            names
        };

        // Each child only sees the prop passed to its own usage site.
        assert_eq!(attrs_for(child_a), vec!["foo"]);
        assert_eq!(attrs_for(child_b), vec!["bar"]);
    }

    /// The same child component used at two sites with different props must not
    /// have its attributes conflated with a sibling child's attributes.
    #[test]
    fn same_child_used_twice_does_not_leak_sibling_attrs() {
        let mut registry = ModuleRegistry::new();

        let parent_analysis = {
            let mut analysis = Croquis::new();
            // Two usages of the same child, each passing its own prop.
            analysis
                .component_usages
                .push(usage_with_prop("Card", "title"));
            analysis
                .component_usages
                .push(usage_with_prop("Card", "subtitle"));
            // An unrelated sibling child receiving a different prop.
            analysis
                .component_usages
                .push(usage_with_prop("Banner", "color"));
            analysis
        };

        let (parent_id, _) = registry.register("Parent.vue", "", parent_analysis);
        let (card_id, _) = registry.register("Card.vue", "", Croquis::new());
        let (banner_id, _) = registry.register("Banner.vue", "", Croquis::new());

        let mut graph = DependencyGraph::new();
        graph.add_node(graph_node(parent_id, "Parent.vue", "Parent"));
        graph.add_node(graph_node(card_id, "Card.vue", "Card"));
        graph.add_node(graph_node(banner_id, "Banner.vue", "Banner"));
        graph.add_edge(parent_id, card_id, DependencyEdge::ComponentUsage);
        graph.add_edge(parent_id, banner_id, DependencyEdge::ComponentUsage);

        let (infos, _diags) = analyze_fallthrough(&registry, &graph);

        let attrs_for = |id: FileId| -> Vec<&str> {
            let info = infos.iter().find(|i| i.file_id == id).unwrap();
            let mut names: Vec<&str> = info.passed_attrs.iter().map(|s| s.as_str()).collect();
            names.sort_unstable();
            names
        };

        // Card aggregates props from both of its usage sites, but never picks
        // up `color`, which only the Banner sibling received.
        assert_eq!(attrs_for(card_id), vec!["subtitle", "title"]);
        assert_eq!(attrs_for(banner_id), vec!["color"]);
    }

    #[test]
    fn test_fallthrough_info_issues() {
        // Single root element - no issue
        let mut info = FallthroughInfo {
            file_id: FileId::new(0),
            inherit_attrs_disabled: false,
            uses_attrs: false,
            binds_attrs: false,
            root_element_count: 1,
            passed_attrs: FxHashSet::default(),
            declared_props: FxHashSet::default(),
            template_start: 0,
            template_end: 0,
        };
        assert!(!info.has_potential_issues());

        // Multiple roots without binds_attrs - this IS an issue
        info.root_element_count = 2;
        assert!(info.has_potential_issues());

        // Multiple roots WITH binds_attrs - no issue
        info.binds_attrs = true;
        assert!(!info.has_potential_issues());

        // Reset and test inheritAttrs disabled without using $attrs
        info.binds_attrs = false;
        info.root_element_count = 1;
        info.inherit_attrs_disabled = true;
        assert!(info.has_potential_issues());

        // inheritAttrs disabled but $attrs is used - no issue
        info.uses_attrs = true;
        assert!(!info.has_potential_issues());
    }
}
