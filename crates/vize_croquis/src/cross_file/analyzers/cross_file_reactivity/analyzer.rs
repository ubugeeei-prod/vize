//! Core analysis phases for cross-file reactivity tracking.
//!
//! Contains the `CrossFileReactivityAnalyzer` struct and its collection,
//! tracking, and detection methods.

use super::types::{
    ComposableInfo, CrossFileReactiveValue, CrossFileReactivityIssue, CrossFileReactivityIssueKind,
    ProvideDefinition, ReactiveValueId, ReactivityFlow, ReactivityFlowKind, ReactivityLossReason,
};
use crate::cross_file::diagnostics::{CrossFileDiagnostic, DiagnosticSeverity};
use crate::cross_file::graph::{DependencyEdge, DependencyGraph};
use crate::cross_file::registry::{FileId, ModuleRegistry};
use crate::provide::ProvideKey;
use crate::reactivity::ReactiveKind;
use vize_carton::{cstr, CompactString, FxHashMap, FxHashSet, SmallVec};

/// The cross-file reactivity analyzer.
pub struct CrossFileReactivityAnalyzer<'a> {
    pub(super) registry: &'a ModuleRegistry,
    pub(super) graph: &'a DependencyGraph,
    /// All tracked reactive values.
    pub(super) reactive_values: FxHashMap<ReactiveValueId, CrossFileReactiveValue>,
    /// Reactivity flows between files.
    pub(super) flows: Vec<ReactivityFlow>,
    /// Detected issues.
    pub(super) issues: Vec<CrossFileReactivityIssue>,
    /// Composable definitions (file -> composable name -> return type info).
    pub(super) composables: FxHashMap<FileId, Vec<ComposableInfo>>,
    /// Provide definitions by component file.
    pub(super) provides: FxHashMap<FileId, Vec<ProvideDefinition>>,
}

impl<'a> CrossFileReactivityAnalyzer<'a> {
    /// Create a new analyzer.
    pub fn new(registry: &'a ModuleRegistry, graph: &'a DependencyGraph) -> Self {
        Self {
            registry,
            graph,
            reactive_values: FxHashMap::default(),
            flows: Vec::new(),
            issues: Vec::new(),
            composables: FxHashMap::default(),
            provides: FxHashMap::default(),
        }
    }

    /// Run the full analysis.
    pub fn analyze(mut self) -> (Vec<CrossFileReactivityIssue>, Vec<CrossFileDiagnostic>) {
        // Phase 1: Collect all reactive value definitions
        self.collect_reactive_definitions();

        // Phase 2: Collect composable definitions
        self.collect_composables();

        // Phase 3: Collect provide definitions
        self.collect_provides();

        // Phase 4: Track flows across file boundaries
        self.track_cross_file_flows();

        // Phase 5: Detect issues
        self.detect_issues();

        // Generate diagnostics
        let diagnostics = self.generate_diagnostics();

        (self.issues, diagnostics)
    }

    /// Phase 1: Collect all reactive value definitions from each file.
    fn collect_reactive_definitions(&mut self) {
        for entry in self.registry.vue_components() {
            let file_id = entry.id;
            let analysis = &entry.analysis;

            // Collect from reactivity sources
            for source in analysis.reactivity.sources() {
                let id = ReactiveValueId {
                    file_id,
                    name: source.name.clone(),
                    offset: source.declaration_offset,
                };

                self.reactive_values.insert(
                    id.clone(),
                    CrossFileReactiveValue {
                        id,
                        kind: source.kind,
                        exposures: SmallVec::new(),
                        consumptions: SmallVec::new(),
                        reactivity_preserved: true,
                    },
                );
            }
        }

        // Also collect from TypeScript/JavaScript modules
        for entry in self.registry.iter().filter(|e| !e.is_vue_sfc) {
            let file_id = entry.id;
            let analysis = &entry.analysis;

            for source in analysis.reactivity.sources() {
                let id = ReactiveValueId {
                    file_id,
                    name: source.name.clone(),
                    offset: source.declaration_offset,
                };

                self.reactive_values.insert(
                    id.clone(),
                    CrossFileReactiveValue {
                        id,
                        kind: source.kind,
                        exposures: SmallVec::new(),
                        consumptions: SmallVec::new(),
                        reactivity_preserved: true,
                    },
                );
            }
        }
    }

    /// Phase 2: Collect composable function definitions.
    fn collect_composables(&mut self) {
        // Composables are typically in .ts files with "use" prefix
        for entry in self.registry.iter().filter(|e| !e.is_vue_sfc) {
            let file_id = entry.id;
            let path = entry.path.to_string_lossy();
            let path_str = path.as_ref();

            // Check if this looks like a composable file
            let filename = path_str.rsplit('/').next().unwrap_or(path_str);
            if !filename.starts_with("use") && !path_str.contains("/composables/") {
                continue;
            }

            let analysis = &entry.analysis;
            let mut composable_infos = Vec::new();

            // Look for exported functions that start with "use"
            for scope in analysis.scopes.iter() {
                if let crate::scope::ScopeKind::Function = scope.kind {
                    for (name, _) in scope.bindings() {
                        if name.starts_with("use") {
                            // This is likely a composable
                            // Collect its reactive returns
                            let reactive_returns: Vec<(CompactString, ReactiveKind)> = analysis
                                .reactivity
                                .sources()
                                .iter()
                                .map(|s| (s.name.clone(), s.kind))
                                .collect();

                            composable_infos.push(ComposableInfo {
                                name: CompactString::new(name),
                                reactive_returns,
                                file_id,
                                offset: scope.span.start,
                            });
                        }
                    }
                }
            }

            if !composable_infos.is_empty() {
                self.composables.insert(file_id, composable_infos);
            }
        }
    }

    /// Phase 3: Collect provide() definitions.
    fn collect_provides(&mut self) {
        for entry in self.registry.vue_components() {
            let file_id = entry.id;
            let analysis = &entry.analysis;

            for provide in analysis.provide_inject.provides() {
                let key_str = provide_key_display(&provide.key);
                let key_identity = provide_key_identity(&provide.key);

                // Check if the provided value is reactive
                let reactive_kind = provided_value_reactive_kind(analysis, provide.value.as_str());
                let is_reactive = reactive_kind.is_some();

                self.provides
                    .entry(file_id)
                    .or_default()
                    .push(ProvideDefinition {
                        file_id,
                        key: key_str,
                        key_identity,
                        value_name: provide.value.clone(),
                        is_reactive,
                        reactive_kind,
                        offset: provide.start,
                    });
            }
        }
    }

    /// Phase 4: Track reactivity flows across file boundaries.
    fn track_cross_file_flows(&mut self) {
        // Track composable import flows
        self.track_composable_flows();

        // Track provide/inject flows
        self.track_provide_inject_flows();

        // Track props flows
        self.track_props_flows();
    }

    /// Track flows from composable exports to imports.
    fn track_composable_flows(&mut self) {
        for entry in self.registry.vue_components() {
            let consumer_file_id = entry.id;
            let analysis = &entry.analysis;

            // Check for composable calls
            for composable in analysis.provide_inject.composables() {
                // Find the source file for this composable
                let source_file = self.find_composable_source(&composable.source);

                // Record the consumption
                if let Some(source_id) = source_file {
                    // Check if the composable return is destructured
                    // This is a key reactivity loss pattern
                    self.check_composable_usage(
                        consumer_file_id,
                        &composable.name,
                        composable.local_name.as_ref(),
                        source_id,
                        composable.start,
                    );
                }
            }
        }
    }

    /// Find the source file for a composable import path.
    fn find_composable_source(&self, source_path: &str) -> Option<FileId> {
        // Try to resolve the import path to a file
        for node in self.graph.nodes() {
            if let Some(entry) = self.registry.get(node.file_id) {
                let path = entry.path.to_string_lossy();
                #[allow(clippy::disallowed_macros)]
                if path.ends_with(&format!("{}.ts", source_path))
                    || path.ends_with(&format!("{}/index.ts", source_path))
                    || path.contains(source_path)
                {
                    return Some(node.file_id);
                }
            }
        }
        None
    }

    /// Check how a composable is used and detect issues.
    fn check_composable_usage(
        &mut self,
        consumer_file_id: FileId,
        composable_name: &CompactString,
        local_name: Option<&CompactString>,
        _source_file_id: FileId,
        offset: u32,
    ) {
        // If the composable result is not assigned to a variable (destructured directly),
        // we need to check the pattern
        if local_name.is_none() {
            // The composable return was destructured
            // This is often a reactivity loss if the composable returns reactive values
            self.issues.push(CrossFileReactivityIssue {
                file_id: consumer_file_id,
                kind: CrossFileReactivityIssueKind::ComposableReturnDestructured {
                    composable_name: composable_name.clone(),
                    destructured_props: vec![CompactString::new("(unknown)")],
                },
                offset,
                related_file: None,
                severity: DiagnosticSeverity::Warning,
            });
        }
    }

    /// Track provide/inject flows.
    fn track_provide_inject_flows(&mut self) {
        for entry in self.registry.vue_components() {
            let consumer_file_id = entry.id;
            let analysis = &entry.analysis;

            for inject in analysis.provide_inject.injects() {
                let key_str = provide_key_display(&inject.key);
                let key_identity = provide_key_identity(&inject.key);

                // Find providers in every ancestor branch. A component can be reused
                // under multiple parents, so a single inject can have multiple
                // runtime provider contexts.
                for provider in self.find_nearest_providers(consumer_file_id, key_identity.as_str())
                {
                    // Check if inject result is destructured
                    use crate::provide::InjectPattern;
                    match &inject.pattern {
                        InjectPattern::ObjectDestructure(props) => {
                            self.issues.push(CrossFileReactivityIssue {
                                file_id: consumer_file_id,
                                kind: CrossFileReactivityIssueKind::InjectValueDestructured {
                                    key: key_str.clone(),
                                    destructured_props: props.clone(),
                                },
                                offset: inject.start,
                                related_file: Some(provider.file_id),
                                severity: DiagnosticSeverity::Error,
                            });
                        }
                        InjectPattern::ArrayDestructure(_) => {
                            self.issues.push(CrossFileReactivityIssue {
                                file_id: consumer_file_id,
                                kind: CrossFileReactivityIssueKind::InjectValueDestructured {
                                    key: key_str.clone(),
                                    destructured_props: vec![CompactString::new(
                                        "(array destructure)",
                                    )],
                                },
                                offset: inject.start,
                                related_file: Some(provider.file_id),
                                severity: DiagnosticSeverity::Error,
                            });
                        }
                        InjectPattern::IndirectDestructure { props, offset, .. } => {
                            // Indirect destructuring also loses reactivity
                            self.issues.push(CrossFileReactivityIssue {
                                file_id: consumer_file_id,
                                kind: CrossFileReactivityIssueKind::InjectValueDestructured {
                                    key: key_str.clone(),
                                    destructured_props: props.clone(),
                                },
                                offset: *offset,
                                related_file: Some(provider.file_id),
                                severity: DiagnosticSeverity::Error,
                            });
                        }
                        InjectPattern::Simple => {
                            // OK - inject is assigned to a variable
                        }
                    }

                    // Check if provider provides non-reactive value
                    if !provider.is_reactive {
                        self.issues.push(CrossFileReactivityIssue {
                            file_id: provider.file_id,
                            kind: CrossFileReactivityIssueKind::NonReactiveProvide {
                                key: provider.key.clone(),
                            },
                            offset: provider.offset,
                            related_file: Some(consumer_file_id),
                            severity: DiagnosticSeverity::Warning,
                        });
                    }

                    // Create a flow record
                    let source_id = ReactiveValueId {
                        file_id: provider.file_id,
                        name: provider.value_name.clone(),
                        offset: provider.offset,
                    };
                    let target_id = ReactiveValueId {
                        file_id: consumer_file_id,
                        name: inject.local_name.clone(),
                        offset: inject.start,
                    };

                    let (preserved, loss_reason) = match &inject.pattern {
                        InjectPattern::Simple => (true, None),
                        InjectPattern::ObjectDestructure(_props) => {
                            (false, Some(ReactivityLossReason::InjectDestructure))
                        }
                        InjectPattern::ArrayDestructure(_) => (
                            false,
                            Some(ReactivityLossReason::Destructured { props: vec![] }),
                        ),
                        InjectPattern::IndirectDestructure { .. } => {
                            (false, Some(ReactivityLossReason::InjectDestructure))
                        }
                    };

                    self.flows.push(ReactivityFlow {
                        source: source_id,
                        target: target_id,
                        flow_kind: ReactivityFlowKind::ProvideInject,
                        preserved,
                        loss_reason,
                    });
                }
            }
        }
    }

    fn find_nearest_providers(
        &self,
        consumer_file_id: FileId,
        key_identity: &str,
    ) -> Vec<ProvideDefinition> {
        let mut providers = Vec::new();
        let mut seen_providers = FxHashSet::default();
        let mut queue = vec![(consumer_file_id, vec![consumer_file_id])];
        let mut cursor = 0;

        while cursor < queue.len() {
            let (current, path) = queue[cursor].clone();
            cursor += 1;

            if current != consumer_file_id {
                if let Some(provides) = self.provides.get(&current) {
                    if let Some(provider) = provides
                        .iter()
                        .rev()
                        .find(|provider| provider.key_identity.as_str() == key_identity)
                    {
                        if seen_providers.insert((provider.file_id, provider.offset)) {
                            providers.push(provider.clone());
                        }
                        continue;
                    }
                }
            }

            let mut parents: Vec<_> = self
                .graph
                .dependents(current)
                .filter(|(parent_id, edge_type)| {
                    *edge_type == DependencyEdge::ComponentUsage && !path.contains(parent_id)
                })
                .collect();
            parents.sort_by_key(|(parent_id, _)| parent_id.as_u32());

            for (parent_id, _) in parents {
                let mut new_path = path.clone();
                new_path.push(parent_id);
                queue.push((parent_id, new_path));
            }
        }

        providers.sort_by_key(|provider| (provider.file_id.as_u32(), provider.offset));
        providers
    }

    /// Track props flows between parent and child components.
    fn track_props_flows(&mut self) {
        for node in self.graph.nodes() {
            let parent_file_id = node.file_id;

            // Check component usages from this file
            for (child_file_id, edge_type) in &node.imports {
                if *edge_type != DependencyEdge::ComponentUsage {
                    continue;
                }

                // Get the parent's component usages
                if let Some(parent_entry) = self.registry.get(parent_file_id) {
                    for usage in &parent_entry.analysis.component_usages {
                        // Check each prop passed
                        for prop in &usage.props {
                            // Skip if no value
                            let Some(value) = &prop.value else {
                                continue;
                            };

                            // Check if this prop is reactive in the parent
                            let is_reactive = parent_entry
                                .analysis
                                .reactivity
                                .sources()
                                .iter()
                                .any(|s| s.name == *value);

                            if is_reactive {
                                // Create a props flow
                                let source_id = ReactiveValueId {
                                    file_id: parent_file_id,
                                    name: value.clone(),
                                    offset: prop.start,
                                };
                                let target_id = ReactiveValueId {
                                    file_id: *child_file_id,
                                    name: prop.name.clone(),
                                    offset: 0, // We don't know child's offset here
                                };

                                self.flows.push(ReactivityFlow {
                                    source: source_id,
                                    target: target_id,
                                    flow_kind: ReactivityFlowKind::PropsFlow,
                                    preserved: true, // Props flow preserves reactivity
                                    loss_reason: None,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    /// Phase 5: Detect additional cross-file issues.
    fn detect_issues(&mut self) {
        // Check for Pinia store destructuring
        for entry in self.registry.vue_components() {
            let file_id = entry.id;
            let analysis = &entry.analysis;

            // Look for Pinia store usage patterns
            self.detect_pinia_issues(file_id, analysis);

            // Detect props destructuring
            self.detect_props_destructure_issues(file_id, analysis);
        }

        // Check for circular reactive dependencies
        self.detect_circular_dependencies();
    }

    /// Detect Pinia store usage issues.
    fn detect_pinia_issues(&mut self, file_id: FileId, analysis: &crate::Croquis) {
        // Look for imports from pinia
        for scope in analysis.scopes.iter() {
            if let crate::scope::ScopeKind::ExternalModule = scope.kind {
                if let crate::scope::ScopeData::ExternalModule(data) = scope.data() {
                    if data.source.as_str() == "pinia" {
                        // Check for storeToRefs usage
                        let has_store_to_refs =
                            scope.bindings().any(|(name, _)| name == "storeToRefs");

                        if !has_store_to_refs {
                            // Check if there are store calls that might be destructured
                            // This is a heuristic - stores are usually named `use*Store`
                            for composable in analysis.provide_inject.composables() {
                                if composable.name.ends_with("Store")
                                    && composable.local_name.is_none()
                                {
                                    self.issues.push(CrossFileReactivityIssue {
                                        file_id,
                                        kind: CrossFileReactivityIssueKind::StoreDestructured {
                                            store_name: composable.name.clone(),
                                            destructured_props: vec![],
                                        },
                                        offset: composable.start,
                                        related_file: None,
                                        severity: DiagnosticSeverity::Warning,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Detect props destructuring issues.
    fn detect_props_destructure_issues(&mut self, file_id: FileId, analysis: &crate::Croquis) {
        if let Some(destructure) = analysis.macros.props_destructure() {
            // Check if toRefs is used
            let has_to_refs = analysis
                .reactivity
                .sources()
                .iter()
                .any(|s| matches!(s.kind, ReactiveKind::ToRefs));

            if !has_to_refs && !destructure.bindings.is_empty() {
                let props: Vec<CompactString> = destructure.bindings.keys().cloned().collect();

                // Note: Modern Vue handles this via reactive props destructure transform
                // So we only emit an info-level diagnostic
                self.issues.push(CrossFileReactivityIssue {
                    file_id,
                    kind: CrossFileReactivityIssueKind::PropsDestructured {
                        destructured_props: props,
                    },
                    offset: 0, // Destructure location from macro analysis
                    related_file: None,
                    severity: DiagnosticSeverity::Info,
                });
            }
        }
    }

    /// Detect circular reactive dependencies.
    fn detect_circular_dependencies(&mut self) {
        // Build a graph of reactive value dependencies
        let mut visited: FxHashSet<ReactiveValueId> = FxHashSet::default();
        let mut rec_stack: FxHashSet<ReactiveValueId> = FxHashSet::default();
        let mut path: Vec<CompactString> = Vec::new();

        for flow in &self.flows {
            if self.dfs_cycle_detect(&flow.source, &mut visited, &mut rec_stack, &mut path) {
                // Found a cycle
                let file_id = flow.source.file_id;
                self.issues.push(CrossFileReactivityIssue {
                    file_id,
                    kind: CrossFileReactivityIssueKind::CircularReactiveDependency {
                        cycle: path.clone(),
                    },
                    offset: flow.source.offset,
                    related_file: Some(flow.target.file_id),
                    severity: DiagnosticSeverity::Warning,
                });
                break;
            }
        }
    }

    /// DFS for cycle detection.
    fn dfs_cycle_detect(
        &self,
        current: &ReactiveValueId,
        visited: &mut FxHashSet<ReactiveValueId>,
        rec_stack: &mut FxHashSet<ReactiveValueId>,
        path: &mut Vec<CompactString>,
    ) -> bool {
        if rec_stack.contains(current) {
            return true;
        }
        if visited.contains(current) {
            return false;
        }

        visited.insert(current.clone());
        rec_stack.insert(current.clone());
        path.push(current.name.clone());

        // Find outgoing edges
        for flow in &self.flows {
            if flow.source == *current
                && self.dfs_cycle_detect(&flow.target, visited, rec_stack, path)
            {
                return true;
            }
        }

        path.pop();
        rec_stack.remove(current);
        false
    }
}

fn provided_value_reactive_kind(analysis: &crate::Croquis, value: &str) -> Option<ReactiveKind> {
    let value = value.trim();

    if let Some(source) = analysis
        .reactivity
        .sources()
        .iter()
        .find(|source| source.name.as_str() == value)
    {
        return Some(source.kind);
    }

    let callee = value
        .split_once('(')
        .map(|(callee, _)| callee.trim())
        .unwrap_or_default();

    match callee {
        "ref" => Some(ReactiveKind::Ref),
        "shallowRef" => Some(ReactiveKind::ShallowRef),
        "reactive" => Some(ReactiveKind::Reactive),
        "shallowReactive" => Some(ReactiveKind::ShallowReactive),
        "computed" => Some(ReactiveKind::Computed),
        "readonly" => Some(ReactiveKind::Readonly),
        "shallowReadonly" => Some(ReactiveKind::ShallowReadonly),
        "toRef" => Some(ReactiveKind::ToRef),
        "toRefs" => Some(ReactiveKind::ToRefs),
        _ => None,
    }
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
