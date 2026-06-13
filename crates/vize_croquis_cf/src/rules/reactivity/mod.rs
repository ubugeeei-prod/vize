//! Reactivity tracking and loss detection.

mod component;
mod diagnostics;
mod imports;
mod losses;
mod types;

use crate::diagnostics::CrossFileDiagnostic;
use crate::graph::DependencyGraph;
use crate::registry::ModuleRegistry;

pub use types::{ReactivityIssue, ReactivityIssueKind};

/// Analyze reactivity issues across components.
pub fn analyze_reactivity(
    registry: &ModuleRegistry,
    _graph: &DependencyGraph,
) -> (Vec<ReactivityIssue>, Vec<CrossFileDiagnostic>) {
    let mut issues = Vec::new();
    let mut diagnostics = Vec::new();

    for entry in registry.vue_components() {
        let analysis = &entry.analysis;
        let file_id = entry.id;
        let component_issues = component::analyze_component_reactivity(analysis);

        for issue in component_issues {
            let diag = diagnostics::create_diagnostic(file_id, &issue);
            diagnostics.push(diag);

            issues.push(ReactivityIssue {
                file_id,
                kind: issue.kind,
                offset: issue.offset,
                source: issue.source,
            });
        }
    }

    (issues, diagnostics)
}

#[cfg(test)]
mod tests;
