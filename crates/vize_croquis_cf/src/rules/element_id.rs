//! Unique element ID analysis.
//!
//! Detects issues with element IDs across components:
//! - Duplicate IDs across different components
//! - Non-unique IDs generated in loops
//!
//! Uses the `element_ids` field from Croquis analysis for accurate tracking.

use crate::diagnostics::{CrossFileDiagnostic, CrossFileDiagnosticKind, DiagnosticSeverity};
use crate::registry::{FileId, ModuleRegistry};
use vize_carton::{CompactString, FxHashMap, cstr};
use vize_croquis::analysis::ElementIdKind;

/// Information about a unique ID issue.
#[derive(Debug, Clone)]
pub struct UniqueIdIssue {
    /// The ID value or pattern.
    pub id: CompactString,
    /// Files where this ID appears.
    pub locations: Vec<(FileId, u32)>,
    /// Whether this is in a loop.
    pub in_loop: bool,
    /// Whether this is a dynamic ID expression.
    pub is_dynamic: bool,
    /// Kind of ID (id, for, aria reference, etc.)
    pub kind: ElementIdKind,
}

#[derive(Debug, Clone, Copy)]
struct IdLocation {
    file_id: FileId,
    start: u32,
    end: u32,
    in_loop: bool,
    kind: ElementIdKind,
}

/// Analyze element IDs across all components.
///
/// Uses the pre-collected `element_ids` from each file's Croquis analysis
/// to detect duplicate IDs and non-unique IDs in loops.
pub fn analyze_element_ids(
    registry: &ModuleRegistry,
) -> (Vec<UniqueIdIssue>, Vec<CrossFileDiagnostic>) {
    let mut issues = Vec::new();
    let mut diagnostics = Vec::new();

    // Collect all static IDs with their locations (grouped by ID value)
    let mut static_ids: FxHashMap<CompactString, Vec<IdLocation>> = FxHashMap::default();

    // Collect dynamic IDs that might not be unique
    let mut dynamic_ids_in_loops: Vec<(FileId, CompactString, u32, u32, ElementIdKind)> =
        Vec::new();

    for entry in registry.vue_components() {
        // Use the pre-collected element_ids from Croquis analysis
        for id_info in &entry.analysis.element_ids {
            // Only track ID definitions (not references like `for`, `aria-labelledby`)
            // References will be checked for matching definitions separately
            if id_info.kind.is_definition() {
                if id_info.is_static {
                    static_ids
                        .entry(id_info.value.clone())
                        .or_default()
                        .push(IdLocation {
                            file_id: entry.id,
                            start: id_info.start,
                            end: id_info.end,
                            in_loop: id_info.in_loop,
                            kind: id_info.kind,
                        });
                } else if id_info.in_loop {
                    // Dynamic ID in a loop - might not be unique
                    dynamic_ids_in_loops.push((
                        entry.id,
                        id_info.value.clone(),
                        id_info.start,
                        id_info.end,
                        id_info.kind,
                    ));
                }
            }
        }
    }

    // Check for duplicate static IDs across files
    for (id, locations) in &static_ids {
        if locations.len() > 1 {
            // Same ID used in multiple places (cross-file duplicate)
            let loc_list: Vec<_> = locations
                .iter()
                .map(|location| (location.file_id, location.start))
                .collect();

            let primary = locations[0];
            let kind = primary.kind;

            issues.push(UniqueIdIssue {
                id: id.clone(),
                locations: loc_list.clone(),
                in_loop: locations.iter().any(|location| location.in_loop),
                is_dynamic: false,
                kind,
            });

            let mut diagnostic = CrossFileDiagnostic::new(
                CrossFileDiagnosticKind::DuplicateElementId {
                    id: id.clone(),
                    locations: loc_list,
                },
                DiagnosticSeverity::Warning,
                primary.file_id,
                primary.start,
                cstr!(
                    "Element ID '{}' is used in {} different locations across files",
                    id,
                    locations.len()
                ),
            )
            .with_end_offset(primary.end)
            .with_suggestion("Use useId() to generate unique IDs for each component instance");

            for related in locations.iter().skip(1) {
                diagnostic = diagnostic.with_related(
                    related.file_id,
                    related.start,
                    cstr!("duplicate id '{id}' is also defined here"),
                );
            }

            diagnostics.push(diagnostic);
        }

        // Check for static IDs inside loops (will create duplicate IDs)
        for location in locations {
            if location.in_loop {
                issues.push(UniqueIdIssue {
                    id: id.clone(),
                    locations: vec![(location.file_id, location.start)],
                    in_loop: true,
                    is_dynamic: false,
                    kind: location.kind,
                });

                diagnostics.push(
                    CrossFileDiagnostic::new(
                        CrossFileDiagnosticKind::NonUniqueIdInLoop {
                            id_expression: id.clone(),
                        },
                        DiagnosticSeverity::Error,
                        location.file_id,
                        location.start,
                        cstr!("Static ID '{id}' inside v-for will create duplicate IDs",),
                    )
                    .with_end_offset(location.end)
                    .with_suggestion("Use a dynamic ID like `:id=\"`item-${index}`\"` or useId()"),
                );
            }
        }
    }

    // Check dynamic IDs in loops that might not be unique
    for (file_id, id_expr, start, end, kind) in dynamic_ids_in_loops {
        // Check if the expression likely produces unique values
        if !looks_unique(&id_expr) {
            issues.push(UniqueIdIssue {
                id: id_expr.clone(),
                locations: vec![(file_id, start)],
                in_loop: true,
                is_dynamic: true,
                kind,
            });

            diagnostics.push(
                CrossFileDiagnostic::new(
                    CrossFileDiagnosticKind::NonUniqueIdInLoop {
                        id_expression: id_expr.clone(),
                    },
                    DiagnosticSeverity::Warning,
                    file_id,
                    start,
                    cstr!("Dynamic ID '{id_expr}' may not produce unique values",),
                )
                .with_end_offset(end)
                .with_suggestion("Include a unique identifier like index or item.id"),
            );
        }
    }

    (issues, diagnostics)
}

/// Check if an ID expression looks like it would produce unique values.
fn looks_unique(expr: &str) -> bool {
    // Common patterns that indicate uniqueness (all lowercase for case-insensitive matching)
    let unique_patterns = [
        "index",
        ".id",
        ".uuid",
        ".key",
        "getid",
        "uniqueid",
        "nanoid",
        "uuid",
        "math.random",
        "date.now",
        "generateid",
    ];

    let expr_lower = expr.to_lowercase();
    unique_patterns.iter().any(|p| expr_lower.contains(p))
}

#[cfg(test)]
mod tests {
    use super::looks_unique;

    #[test]
    fn test_looks_unique() {
        assert!(looks_unique("`item-${index}`"));
        assert!(looks_unique("item.id"));
        assert!(looks_unique("generateId()"));
        assert!(!looks_unique("'static-id'"));
        assert!(!looks_unique("item.name"));
    }
}
