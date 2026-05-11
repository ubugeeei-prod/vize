//! Provide/Inject analysis.
//!
//! Matches provide() calls with inject() consumers across the component tree:
//! - Unmatched inject (no provider found in ancestors)
//! - Unused provide (no inject found in descendants)
//! - Type mismatches between provide and inject

use crate::cross_file::diagnostics::{
    CrossFileDiagnostic, CrossFileDiagnosticKind, DiagnosticSeverity,
};
use crate::cross_file::graph::{DependencyEdge, DependencyGraph};
use crate::cross_file::registry::{FileId, ModuleRegistry};
use crate::provide::{InjectEntry, InjectPattern, ProvideEntry, ProvideKey};
use vize_carton::{cstr, CompactString, FxHashMap, FxHashSet, String};

/// Information about a provide/inject match.
#[derive(Debug, Clone)]
pub struct ProvideInjectMatch {
    /// Component providing the value.
    pub provider: FileId,
    /// Component injecting the value.
    pub consumer: FileId,
    /// The provide/inject key.
    pub key: CompactString,
    /// Stable key identity including string/symbol namespace.
    pub key_identity: CompactString,
    /// Path from provider to consumer.
    pub path: Vec<FileId>,
    /// Whether types match (if available).
    pub type_match: Option<bool>,
    /// Provider offset in source.
    pub provide_offset: u32,
    /// Consumer offset in source.
    pub inject_offset: u32,
}

/// Tree representation of provide/inject relationships.
#[derive(Debug, Clone)]
pub struct ProvideInjectTree {
    /// Root nodes (components that provide but don't inject from ancestors).
    pub roots: Vec<ProvideNode>,
}

/// A node in the provide/inject tree.
#[derive(Debug, Clone)]
pub struct ProvideNode {
    /// File ID of this component.
    pub file_id: FileId,
    /// Component name (if available).
    pub component_name: Option<CompactString>,
    /// Keys provided by this component.
    pub provides: Vec<ProvideInfo>,
    /// Keys injected by this component.
    pub injects: Vec<InjectInfo>,
    /// Children components that inject from this provider.
    pub children: Vec<ProvideNode>,
}

/// Information about a provide call.
#[derive(Debug, Clone)]
pub struct ProvideInfo {
    /// The provide key.
    pub key: CompactString,
    /// The provided type (if available).
    pub value_type: Option<CompactString>,
    /// Source offset.
    pub offset: u32,
    /// Number of consumers.
    pub consumer_count: usize,
}

/// Information about an inject call.
#[derive(Debug, Clone)]
pub struct InjectInfo {
    /// The inject key.
    pub key: CompactString,
    /// Whether a default value is provided.
    pub has_default: bool,
    /// The provider file (if found).
    pub provider: Option<FileId>,
    /// Source offset.
    pub offset: u32,
}

impl ProvideInjectTree {
    /// Render the tree as a markdown string for visualization.
    pub fn to_markdown(&self, registry: &ModuleRegistry) -> String {
        let mut output = String::with_capacity(4096);
        output.push_str("## Provide/Inject Tree\n\n");

        if self.roots.is_empty() {
            output.push_str("_No provide/inject relationships found._\n");
            return output;
        }

        for root in &self.roots {
            Self::render_node(&mut output, root, registry, 0);
        }

        output
    }

    fn render_node(
        output: &mut String,
        node: &ProvideNode,
        registry: &ModuleRegistry,
        depth: usize,
    ) {
        use std::fmt::Write;

        let indent = "  ".repeat(depth);
        let name = node
            .component_name
            .as_deref()
            .or_else(|| {
                registry
                    .get(node.file_id)
                    .and_then(|e| e.path.file_stem()?.to_str())
            })
            .unwrap_or("<unknown>");

        // Component name
        writeln!(output, "{}📦 **{}**", indent, name).ok();

        // Provides
        if !node.provides.is_empty() {
            for p in &node.provides {
                let type_str = p
                    .value_type
                    .as_deref()
                    .map(|t| cstr!(": `{t}`"))
                    .unwrap_or_default();
                let consumers = if p.consumer_count > 0 {
                    cstr!(" → {} consumer(s)", p.consumer_count)
                } else {
                    CompactString::new(" ⚠️ _unused_")
                };
                writeln!(
                    output,
                    "{}  🔹 provide(`\"{}\"`){}{} ",
                    indent, p.key, type_str, consumers
                )
                .ok();
            }
        }

        // Injects
        if !node.injects.is_empty() {
            for i in &node.injects {
                let default_str = if i.has_default { " (has default)" } else { "" };
                let provider_str = if i.provider.is_some() {
                    " ✅"
                } else {
                    " ❌ _no provider_"
                };
                writeln!(
                    output,
                    "{}  🔸 inject(`\"{}\"`){}{} ",
                    indent, i.key, default_str, provider_str
                )
                .ok();
            }
        }

        // Children
        for child in &node.children {
            Self::render_node(output, child, registry, depth + 1);
        }
    }
}

/// Build the provide/inject tree from analysis results.
pub fn build_provide_inject_tree(
    registry: &ModuleRegistry,
    _graph: &DependencyGraph,
    matches: &[ProvideInjectMatch],
) -> ProvideInjectTree {
    // Collect all provides and injects
    let mut provides_map: FxHashMap<FileId, Vec<ProvideEntry>> = FxHashMap::default();
    let mut injects_map: FxHashMap<FileId, Vec<InjectEntry>> = FxHashMap::default();
    let mut consumer_counts: FxHashMap<(FileId, CompactString), usize> = FxHashMap::default();

    for entry in registry.vue_components() {
        let (p, i) = extract_provide_inject(&entry.analysis);
        if !p.is_empty() {
            provides_map.insert(entry.id, p);
        }
        if !i.is_empty() {
            injects_map.insert(entry.id, i);
        }
    }

    // Count consumers for each provide
    for m in matches {
        *consumer_counts
            .entry((m.provider, m.key_identity.clone()))
            .or_insert(0) += 1;
    }

    // Build the displayed tree from resolved provider -> ... -> consumer paths.
    // This keeps pass-through components visible even when they do not provide
    // or inject the key themselves.
    let mut included_nodes = FxHashSet::default();
    let mut child_map: FxHashMap<FileId, Vec<FileId>> = FxHashMap::default();
    let mut parent_map: FxHashMap<FileId, FileId> = FxHashMap::default();

    for m in matches {
        for file_id in &m.path {
            included_nodes.insert(*file_id);
        }
        for pair in m.path.windows(2) {
            let parent = pair[0];
            let child = pair[1];
            child_map.entry(parent).or_default().push(child);
            parent_map.entry(child).or_insert(parent);
        }
    }

    for &file_id in provides_map.keys() {
        included_nodes.insert(file_id);
    }
    for &file_id in injects_map.keys() {
        included_nodes.insert(file_id);
    }

    for children in child_map.values_mut() {
        children.sort_by_key(|id| id.as_u32());
        children.dedup();
    }

    let mut root_ids: Vec<_> = included_nodes
        .iter()
        .copied()
        .filter(|file_id| !parent_map.contains_key(file_id))
        .collect();
    root_ids.sort_by_key(|id| id.as_u32());

    let mut visited = FxHashSet::default();
    let roots = root_ids
        .into_iter()
        .map(|file_id| {
            build_node(
                file_id,
                registry,
                &child_map,
                &provides_map,
                &injects_map,
                &consumer_counts,
                matches,
                &mut visited,
            )
        })
        .collect();

    ProvideInjectTree { roots }
}

#[allow(unused, clippy::too_many_arguments)]
fn build_node(
    file_id: FileId,
    registry: &ModuleRegistry,
    child_map: &FxHashMap<FileId, Vec<FileId>>,
    provides_map: &FxHashMap<FileId, Vec<ProvideEntry>>,
    injects_map: &FxHashMap<FileId, Vec<InjectEntry>>,
    consumer_counts: &FxHashMap<(FileId, CompactString), usize>,
    matches: &[ProvideInjectMatch],
    visited: &mut FxHashSet<FileId>,
) -> ProvideNode {
    visited.insert(file_id);

    let component_name = registry.get(file_id).and_then(|e| e.component_name.clone());

    // Build provides info
    let provides: Vec<ProvideInfo> = provides_map
        .get(&file_id)
        .map(|ps| {
            ps.iter()
                .map(|p| {
                    let key = match &p.key {
                        ProvideKey::String(s) => s.clone(),
                        ProvideKey::Symbol(s) => s.clone(),
                    };
                    let key_identity = provide_key_identity(&p.key);
                    let count = *consumer_counts.get(&(file_id, key_identity)).unwrap_or(&0);
                    ProvideInfo {
                        key,
                        value_type: p.value_type.clone(),
                        offset: p.start,
                        consumer_count: count,
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    // Build injects info
    let injects = injects_map
        .get(&file_id)
        .map(|is| {
            is.iter()
                .map(|i| {
                    let key = match &i.key {
                        ProvideKey::String(s) => s.clone(),
                        ProvideKey::Symbol(s) => s.clone(),
                    };
                    let key_identity = provide_key_identity(&i.key);
                    let provider = matches
                        .iter()
                        .find(|m| m.consumer == file_id && m.key_identity == key_identity)
                        .map(|m| m.provider);
                    InjectInfo {
                        key,
                        has_default: i.default_value.is_some(),
                        provider,
                        offset: i.start,
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    // Find children (components that inject from this provider)
    let mut children = Vec::new();
    if let Some(child_ids) = child_map.get(&file_id) {
        for &child_id in child_ids {
            if visited.contains(&child_id) {
                continue;
            }
            let child_node = build_node(
                child_id,
                registry,
                child_map,
                provides_map,
                injects_map,
                consumer_counts,
                matches,
                visited,
            );
            children.push(child_node);
        }
    }

    ProvideNode {
        file_id,
        component_name,
        provides,
        injects,
        children,
    }
}

/// Analyze provide/inject relationships across the component tree.
pub fn analyze_provide_inject(
    registry: &ModuleRegistry,
    graph: &DependencyGraph,
) -> (Vec<ProvideInjectMatch>, Vec<CrossFileDiagnostic>) {
    let mut matches = Vec::new();
    let mut diagnostics = Vec::new();

    // Collect all provides and injects
    let mut provides: FxHashMap<FileId, Vec<ProvideEntry>> = FxHashMap::default();
    let mut injects: FxHashMap<FileId, Vec<InjectEntry>> = FxHashMap::default();

    for entry in registry.vue_components() {
        // Extract provide/inject from analysis
        // In a full implementation, this would come from script_parser
        let (p, i) = extract_provide_inject(&entry.analysis);
        for provide in &p {
            if let ProvideKey::String(key) = &provide.key {
                diagnostics.push(create_string_key_diagnostic(
                    entry.id,
                    key,
                    true,
                    provide.start,
                    provide.end,
                ));
            }
        }
        for inject in &i {
            if let ProvideKey::String(key) = &inject.key {
                diagnostics.push(create_string_key_diagnostic(
                    entry.id,
                    key,
                    false,
                    inject.start,
                    inject.end,
                ));
            }
        }
        if !p.is_empty() {
            provides.insert(entry.id, p);
        }
        if !i.is_empty() {
            injects.insert(entry.id, i);
        }
    }

    // Track which provides are used
    let mut used_provides: FxHashSet<(FileId, ProvideKey)> = FxHashSet::default();

    // For each inject, try to find a matching provide in ancestors
    for (&consumer_id, consumer_injects) in &injects {
        for inject in consumer_injects {
            let key_str = provide_key_display(&inject.key);
            let provider_match = find_provider(consumer_id, &inject.key, &provides, graph);
            let provider_related = provider_match
                .as_ref()
                .map(|(provider_id, provide_entry, _)| (*provider_id, provide_entry.start));

            // Check for destructured inject - this causes reactivity loss
            match &inject.pattern {
                InjectPattern::ObjectDestructure(props) => {
                    let diagnostic =
                        CrossFileDiagnostic::new(
                            CrossFileDiagnosticKind::DestructuringBreaksReactivity {
                                source_name: cstr!("inject('{key_str}')"),
                                destructured_keys: props.clone(),
                                suggestion: CompactString::new("toRefs"),
                            },
                            DiagnosticSeverity::Error,
                            consumer_id,
                            inject.start,
                            cstr!(
                                "Destructuring inject('{}') into {{ {} }} breaks reactivity connection",
                                key_str,
                                props.iter().map(|p| p.as_str()).collect::<Vec<_>>().join(", ")
                            ),
                        )
                        .with_end_offset(inject.end)
                        .with_suggestion(cstr!(
                            "Store inject result first: `const {} = inject('{}')`, then access properties",
                            inject.local_name,
                            key_str
                        ));
                    diagnostics.push(with_provider_related(
                        diagnostic,
                        provider_related,
                        &key_str,
                    ));
                }
                InjectPattern::ArrayDestructure(items) => {
                    let diagnostic =
                        CrossFileDiagnostic::new(
                            CrossFileDiagnosticKind::DestructuringBreaksReactivity {
                                source_name: cstr!("inject('{key_str}')"),
                                destructured_keys: items.clone(),
                                suggestion: CompactString::new("toRefs"),
                            },
                            DiagnosticSeverity::Error,
                            consumer_id,
                            inject.start,
                            cstr!(
                                "Array destructuring inject('{}') into [{}] breaks reactivity connection",
                                key_str,
                                items.iter().map(|p| p.as_str()).collect::<Vec<_>>().join(", ")
                            ),
                        )
                        .with_end_offset(inject.end)
                        .with_suggestion(cstr!(
                            "Store inject result first: `const {} = inject('{}')`, then access indices",
                            inject.local_name,
                            key_str
                        ));
                    diagnostics.push(with_provider_related(
                        diagnostic,
                        provider_related,
                        &key_str,
                    ));
                }
                InjectPattern::IndirectDestructure {
                    inject_var,
                    props,
                    offset,
                } => {
                    // Indirect destructuring also loses reactivity
                    let diagnostic =
                        CrossFileDiagnostic::new(
                            CrossFileDiagnosticKind::DestructuringBreaksReactivity {
                                source_name: inject_var.clone(),
                                destructured_keys: props.clone(),
                                suggestion: CompactString::new("toRefs"),
                            },
                            DiagnosticSeverity::Error,
                            consumer_id,
                            *offset,
                            cstr!(
                                "Destructuring '{}' (from inject('{}')) into {{ {} }} breaks reactivity connection",
                                inject_var,
                                key_str,
                                props.iter().map(|p| p.as_str()).collect::<Vec<_>>().join(", ")
                            ),
                        )
                        .with_suggestion(cstr!(
                            "Access properties directly: `{}.prop` instead of destructuring",
                            inject_var
                        ));
                    diagnostics.push(with_provider_related(
                        diagnostic,
                        provider_related,
                        &key_str,
                    ));
                }
                InjectPattern::Simple => {
                    // No reactivity loss issue
                }
            }

            match provider_match {
                Some((provider_id, provide_entry, path)) => {
                    // Found a match
                    used_provides.insert((provider_id, inject.key.clone()));

                    matches.push(ProvideInjectMatch {
                        provider: provider_id,
                        consumer: consumer_id,
                        key: key_str.clone(),
                        key_identity: provide_key_identity(&inject.key),
                        path,
                        type_match: None, // Would need type analysis
                        provide_offset: provide_entry.start,
                        inject_offset: inject.start,
                    });
                }
                None => {
                    // No provider found
                    if inject.default_value.is_none() {
                        diagnostics.push(
                            CrossFileDiagnostic::new(
                                CrossFileDiagnosticKind::UnmatchedInject {
                                    key: key_str.clone(),
                                },
                                DiagnosticSeverity::Error,
                                consumer_id,
                                inject.start,
                                cstr!(
                                    "**Unmatched Inject**: `inject('{}')` has no matching `provide()` in any ancestor component\n\n\
                                    This will return `undefined` at runtime and may cause errors.\n\n\
                                    ### Checklist:\n\
                                    - [ ] Add `provide('{}', value)` in a parent/ancestor component\n\
                                    - [ ] Or provide a default value: `inject('{}', defaultValue)`",
                                    key_str, key_str, key_str
                                ),
                            )
                            .with_end_offset(inject.end)
                            .with_suggestion(cstr!(
                                "```typescript\n// In parent component:\nprovide('{}', yourValue)\n\n// Or with default:\nconst {} = inject('{}', defaultValue)\n```",
                                key_str, inject.local_name, key_str
                            )),
                        );
                    } else {
                        diagnostics.push(
                            CrossFileDiagnostic::new(
                                CrossFileDiagnosticKind::UnmatchedInject {
                                    key: key_str.clone(),
                                },
                                DiagnosticSeverity::Warning,
                                consumer_id,
                                inject.start,
                                cstr!(
                                    "**Unmatched Inject Default**: `inject('{}')` falls back to its default value because no ancestor provides this key.\n\n\
                                    The runtime fallback is safe, but this can hide broken provider wiring.",
                                    key_str
                                ),
                            )
                            .with_end_offset(inject.end)
                            .with_suggestion(cstr!(
                                "Add `provide('{}', value)` in an ancestor, or keep the fallback only if it is intentional",
                                key_str
                            )),
                        );
                    }
                }
            }
        }
    }

    // Check for unused provides
    for (&provider_id, provider_provides) in &provides {
        for provide in provider_provides {
            let key_str = provide_key_display(&provide.key);

            if !used_provides.contains(&(provider_id, provide.key.clone())) {
                // Check if any descendant injects this key
                let has_descendant_inject =
                    has_inject_in_descendants(provider_id, &provide.key, &injects, graph);

                if !has_descendant_inject {
                    diagnostics.push(
                        CrossFileDiagnostic::new(
                            CrossFileDiagnosticKind::UnusedProvide {
                                key: key_str.clone(),
                            },
                            DiagnosticSeverity::Warning,
                            provider_id,
                            provide.start,
                            cstr!(
                                "provide('{}') is not used by any descendant component",
                                key_str
                            ),
                        )
                        .with_end_offset(provide.end)
                        .with_suggestion(
                            "Remove if not needed, or add inject() in a child component",
                        ),
                    );
                }
            }
        }
    }

    (matches, diagnostics)
}

fn with_provider_related(
    diagnostic: CrossFileDiagnostic,
    provider_related: Option<(FileId, u32)>,
    key: &CompactString,
) -> CrossFileDiagnostic {
    if let Some((provider_id, provider_offset)) = provider_related {
        diagnostic.with_related(
            provider_id,
            provider_offset,
            cstr!("provide('{key}') source"),
        )
    } else {
        diagnostic
    }
}

/// Extract provide/inject calls from a component's analysis.
/// Uses the ProvideInjectTracker for precise static analysis - no heuristics.
#[inline]
fn extract_provide_inject(analysis: &crate::Croquis) -> (Vec<ProvideEntry>, Vec<InjectEntry>) {
    // Use the actual provide/inject tracker data - precise static analysis
    let provides = analysis.provide_inject.provides().to_vec();
    let injects = analysis.provide_inject.injects().to_vec();
    (provides, injects)
}

fn provide_key_display(key: &ProvideKey) -> CompactString {
    match key {
        ProvideKey::String(s) | ProvideKey::Symbol(s) => s.clone(),
    }
}

fn provide_key_identity(key: &ProvideKey) -> CompactString {
    match key {
        ProvideKey::String(s) => cstr!("string:{s}"),
        ProvideKey::Symbol(s) => cstr!("symbol:{s}"),
    }
}

fn create_string_key_diagnostic(
    file_id: FileId,
    key: &CompactString,
    is_provide: bool,
    start: u32,
    end: u32,
) -> CrossFileDiagnostic {
    let api_name = if is_provide { "provide" } else { "inject" };
    CrossFileDiagnostic::new(
        CrossFileDiagnosticKind::ProvideInjectWithoutSymbol {
            key: key.clone(),
            is_provide,
        },
        DiagnosticSeverity::Warning,
        file_id,
        start,
        cstr!(
            "{}('{}') uses a string injection key; prefer Symbol/InjectionKey for typed, collision-safe dependency flow",
            api_name,
            key
        ),
    )
    .with_end_offset(end)
    .with_suggestion(cstr!(
        "Define an InjectionKey, for example `const {}Key: InjectionKey<...> = Symbol('{}')`, then use it in provide() and inject()",
        key,
        key
    ))
}

/// Find a provider for a given key in ancestor components.
fn find_provider(
    consumer: FileId,
    key: &ProvideKey,
    provides: &FxHashMap<FileId, Vec<ProvideEntry>>,
    graph: &DependencyGraph,
) -> Option<(FileId, ProvideEntry, Vec<FileId>)> {
    let mut visited = FxHashSet::default();
    let mut queue = vec![(consumer, vec![consumer])];
    let mut cursor = 0;

    while cursor < queue.len() {
        let (current, path) = queue[cursor].clone();
        cursor += 1;

        if visited.contains(&current) {
            continue;
        }
        visited.insert(current);

        // Check if current component provides this key
        if current != consumer {
            if let Some(component_provides) = provides.get(&current) {
                for provide in component_provides {
                    if provide.key == *key {
                        let mut provider_to_consumer = path;
                        provider_to_consumer.reverse();
                        return Some((current, provide.clone(), provider_to_consumer));
                    }
                }
            }
        }

        // Add parents (components that use this one) to queue
        let mut parents: Vec<_> = graph
            .dependents(current)
            .filter(|(parent_id, edge_type)| {
                *edge_type == DependencyEdge::ComponentUsage && !visited.contains(parent_id)
            })
            .collect();
        parents.sort_by_key(|(parent_id, _)| parent_id.as_u32());

        for (parent_id, _) in parents {
            let mut new_path = path.clone();
            new_path.push(parent_id);
            queue.push((parent_id, new_path));
        }
    }

    None
}

/// Check if any descendant component injects a given key.
fn has_inject_in_descendants(
    provider: FileId,
    key: &ProvideKey,
    injects: &FxHashMap<FileId, Vec<InjectEntry>>,
    graph: &DependencyGraph,
) -> bool {
    let mut visited = FxHashSet::default();
    let mut queue = vec![provider];

    while let Some(current) = queue.pop() {
        if visited.contains(&current) {
            continue;
        }
        visited.insert(current);

        // Check descendants (components used by this one)
        for (child_id, edge_type) in graph.dependencies(current) {
            if edge_type == DependencyEdge::ComponentUsage {
                // Check if child injects this key
                if let Some(child_injects) = injects.get(&child_id) {
                    for inject in child_injects {
                        if inject.key == *key {
                            return true;
                        }
                    }
                }

                if !visited.contains(&child_id) {
                    queue.push(child_id);
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use crate::provide::ProvideKey;
    use vize_carton::CompactString;

    #[test]
    fn test_provide_key_match() {
        let key1 = ProvideKey::String(CompactString::new("theme"));
        let key2 = ProvideKey::String(CompactString::new("theme"));

        let s1 = match &key1 {
            ProvideKey::String(s) => s.as_str(),
            ProvideKey::Symbol(s) => s.as_str(),
        };
        let s2 = match &key2 {
            ProvideKey::String(s) => s.as_str(),
            ProvideKey::Symbol(s) => s.as_str(),
        };

        assert_eq!(s1, s2);
    }
}
