use super::{DependencyEdge, DependencyGraph, FileId, FxHashSet};

/// Find potential error sources in a component.
pub(super) fn find_error_sources(analysis: &vize_croquis::Croquis) -> Vec<u32> {
    let mut sources = Vec::new();

    // Look for common error patterns
    let error_patterns = [
        "throw",
        "Error(",
        "reject(",
        "JSON.parse",
        "fetch(",
        "axios",
        "await ",
    ];

    for expr in &analysis.template_expressions {
        for pattern in &error_patterns {
            if expr.content.contains(pattern) {
                sources.push(expr.start);
                break;
            }
        }
    }

    sources
}

/// Check if a component has an ancestor with a boundary.
pub(super) fn has_ancestor_with_boundary(
    file_id: FileId,
    boundaries: &FxHashSet<FileId>,
    graph: &DependencyGraph,
) -> bool {
    let mut visited = FxHashSet::default();
    let mut queue = vec![file_id];

    while let Some(current) = queue.pop() {
        if visited.contains(&current) {
            continue;
        }
        visited.insert(current);

        // Check if current is a boundary
        if current != file_id && boundaries.contains(&current) {
            return true;
        }

        // Add parents to queue
        for (parent_id, edge_type) in graph.dependents(current) {
            if edge_type == DependencyEdge::ComponentUsage && !visited.contains(&parent_id) {
                queue.push(parent_id);
            }
        }
    }

    false
}

/// Find all components protected by a boundary.
pub(super) fn find_protected_components(
    boundary_id: FileId,
    graph: &DependencyGraph,
) -> Vec<FileId> {
    let mut protected = Vec::new();
    let mut visited = FxHashSet::default();
    let mut queue = vec![boundary_id];

    while let Some(current) = queue.pop() {
        if visited.contains(&current) {
            continue;
        }
        visited.insert(current);

        // Add children (components used by this one)
        for (child_id, edge_type) in graph.dependencies(current) {
            if edge_type == DependencyEdge::ComponentUsage {
                protected.push(child_id);
                if !visited.contains(&child_id) {
                    queue.push(child_id);
                }
            }
        }
    }

    protected
}
