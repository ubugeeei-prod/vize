use super::engine::CrossFileReactivityAnalyzer;
use super::types::{CrossFileReactivityIssue, CrossFileReactivityIssueKind};
use crate::diagnostics::DiagnosticSeverity;
use crate::registry::FileId;
use vize_carton::CompactString;

impl<'a> CrossFileReactivityAnalyzer<'a> {
    pub(super) fn track_composable_flows(&mut self) {
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
    pub(super) fn find_composable_source(&self, source_path: &str) -> Option<FileId> {
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
    pub(super) fn check_composable_usage(
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
}
