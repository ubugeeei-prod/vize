use super::analyzer::CrossFileReactivityAnalyzer;
use super::types::{CrossFileReactivityIssue, CrossFileReactivityIssueKind, ReactiveValueId};
use crate::diagnostics::DiagnosticSeverity;
use crate::registry::FileId;
use vize_carton::{CompactString, FxHashSet};

impl<'a> CrossFileReactivityAnalyzer<'a> {
    pub(super) fn detect_issues(&mut self) {
        // Check for Pinia store destructuring
        for entry in self.registry.vue_components() {
            let file_id = entry.id;
            let analysis = &entry.analysis;

            // Look for Pinia store usage patterns
            self.detect_pinia_issues(file_id, analysis);

            // Direct `defineProps` destructure is reactive in modern Vue. Plain
            // aliases from those bindings are tracked by the parser instead.
        }

        // Check for circular reactive dependencies
        self.detect_circular_dependencies();
    }

    /// Detect Pinia store usage issues.
    pub(super) fn detect_pinia_issues(
        &mut self,
        file_id: FileId,
        analysis: &vize_croquis::Croquis,
    ) {
        // Look for imports from pinia
        for scope in analysis.scopes.iter() {
            if let vize_croquis::ScopeKind::ExternalModule = scope.kind
                && let vize_croquis::ScopeData::ExternalModule(data) = scope.data()
                && data.source.as_str() == "pinia"
            {
                // Check for storeToRefs usage
                let has_store_to_refs = scope.bindings().any(|(name, _)| name == "storeToRefs");

                if !has_store_to_refs {
                    // Check if there are store calls that might be destructured
                    // This is a heuristic - stores are usually named `use*Store`
                    for composable in analysis.provide_inject.composables() {
                        if composable.name.ends_with("Store") && composable.local_name.is_none() {
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

    /// Detect circular reactive dependencies.
    pub(super) fn detect_circular_dependencies(&mut self) {
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
    pub(super) fn dfs_cycle_detect(
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
