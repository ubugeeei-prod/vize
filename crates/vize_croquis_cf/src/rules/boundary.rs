//! Server/client, error and Suspense boundary analysis.
//! Detects browser API usage, unprotected async components and missing boundaries.

#[path = "boundary_graph.rs"]
mod graph_walk;
use graph_walk::{find_error_sources, find_protected_components, has_ancestor_with_boundary};

use crate::diagnostics::{
    CrossFileDiagnostic, CrossFileDiagnosticKind, DiagnosticSeverity, DiagnosticSource,
};
use crate::facts::ErrorBoundaryRule;
use crate::graph::{DependencyEdge, DependencyGraph};
use crate::registry::{FileId, ModuleRegistry};
use vize_carton::{CompactString, FxHashSet};
use vize_croquis::facts::{Bindings, BindingsTable, CroquisFacts};

/// Kind of boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryKind {
    /// Server/Client boundary (SSR).
    ServerClient,
    /// Error boundary (onErrorCaptured).
    Error,
    /// Suspense boundary (async components).
    Suspense,
}

/// Information about a boundary.
#[derive(Debug, Clone)]
pub struct BoundaryInfo {
    /// Component that defines the boundary.
    pub file_id: FileId,
    /// Kind of boundary.
    pub kind: BoundaryKind,
    /// Offset in source.
    pub offset: u32,
    /// Components protected by this boundary.
    pub protects: Vec<FileId>,
}

/// Analyze boundaries across the component tree.
pub fn analyze_boundaries(
    registry: &ModuleRegistry,
    graph: &DependencyGraph,
) -> (Vec<BoundaryInfo>, Vec<CrossFileDiagnostic>) {
    let mut boundaries = Vec::new();
    let mut diagnostics = Vec::new();

    // Collect components with boundaries
    let mut error_boundaries: FxHashSet<FileId> = FxHashSet::default();
    let mut suspense_boundaries: FxHashSet<FileId> = FxHashSet::default();
    let mut client_only_apis: Vec<(FileId, CompactString, u32)> = Vec::new();
    let mut async_components: FxHashSet<FileId> = FxHashSet::default();
    let mut components_with_errors: Vec<(FileId, u32)> = Vec::new();

    for entry in registry.vue_components() {
        let analysis = &entry.analysis;

        // Check for error boundary (onErrorCaptured)
        if has_error_captured(analysis) {
            error_boundaries.insert(entry.id);
            boundaries.push(BoundaryInfo {
                file_id: entry.id,
                kind: BoundaryKind::Error,
                offset: 0,
                protects: Vec::new(),
            });
        }

        // Check for Suspense usage
        if uses_suspense(analysis) {
            suspense_boundaries.insert(entry.id);
            boundaries.push(BoundaryInfo {
                file_id: entry.id,
                kind: BoundaryKind::Suspense,
                offset: 0,
                protects: Vec::new(),
            });
        }

        // Check for async setup
        if analysis.macros.is_async() {
            async_components.insert(entry.id);
        }

        // Check for browser-only APIs used outside client-only hooks
        let browser_usages = find_browser_api_usage(analysis);
        for (api, offset, context, source) in browser_usages {
            if source != DiagnosticSource::Script || !is_in_client_only_context(analysis, offset) {
                client_only_apis.push((entry.id, api.clone(), offset));

                diagnostics.push(
                    CrossFileDiagnostic::new(
                        CrossFileDiagnosticKind::BrowserApiInSsr {
                            api,
                            context: CompactString::new(context),
                        },
                        DiagnosticSeverity::Warning,
                        entry.id,
                        offset,
                        "Browser API used in potentially SSR context",
                    )
                    .with_primary_source(source)
                    .with_suggestion("Wrap in onMounted() or use import.meta.client check"),
                );
            }
        }

        // Check for potential errors without boundaries
        let error_sources = find_error_sources(analysis);
        for offset in error_sources {
            components_with_errors.push((entry.id, offset));
        }
    }

    // Check async components for Suspense boundaries
    for async_id in &async_components {
        let has_suspense = has_ancestor_with_boundary(*async_id, &suspense_boundaries, graph);

        if !has_suspense {
            let component_name = registry
                .get(*async_id)
                .and_then(|e| e.component_name.clone())
                .unwrap_or_else(|| CompactString::new("Component"));

            diagnostics.push(
                CrossFileDiagnostic::new(
                    CrossFileDiagnosticKind::AsyncWithoutSuspense { component_name },
                    DiagnosticSeverity::Warning,
                    *async_id,
                    0,
                    "Async component without Suspense boundary",
                )
                .with_suggestion(
                    "Wrap in <Suspense> or use defineAsyncComponent with loading state",
                ),
            );
        }
    }

    // Check error sources for error boundaries
    for (file_id, offset) in &components_with_errors {
        let has_boundary = has_ancestor_with_boundary(*file_id, &error_boundaries, graph);

        if !has_boundary {
            diagnostics.push(
                CrossFileDiagnostic::new(
                    CrossFileDiagnosticKind::UncaughtErrorBoundary,
                    DiagnosticSeverity::Info,
                    *file_id,
                    *offset,
                    "Potential error without error boundary",
                )
                .with_suggestion("Add onErrorCaptured in a parent component"),
            );
        }
    }

    // Update boundary protections
    for boundary in &mut boundaries {
        boundary.protects = find_protected_components(boundary.file_id, graph);
    }

    (boundaries, diagnostics)
}

/// Check if a component has onErrorCaptured.
fn has_error_captured(analysis: &vize_croquis::Croquis) -> bool {
    let mut facts = CroquisFacts::new(analysis);
    let view = facts.prepare::<ErrorBoundaryRule>();
    // Check for onErrorCaptured in bindings or scope
    view.get::<Bindings>()
        .is_ok_and(|bindings| bindings.contains_binding("onErrorCaptured"))
        || analysis.scopes.is_defined("onErrorCaptured")
        || analysis
            .template_expressions
            .iter()
            .any(|e| e.content.contains("onErrorCaptured"))
}

/// Check if a component uses Suspense.
fn uses_suspense(analysis: &vize_croquis::Croquis) -> bool {
    vize_croquis::facts::used_component_contains(analysis, "Suspense")
        || vize_croquis::facts::used_component_name_list(analysis)
            .iter()
            .any(|c| c.as_str() == "Suspense")
}

/// Find browser-only API usage in a component.
fn find_browser_api_usage(
    analysis: &vize_croquis::Croquis,
) -> Vec<(CompactString, u32, &'static str, DiagnosticSource)> {
    let mut usages = Vec::new();

    let browser_apis = [
        ("window", "Browser global"),
        ("document", "DOM API"),
        ("navigator", "Browser API"),
        ("localStorage", "Web Storage"),
        ("sessionStorage", "Web Storage"),
        ("location", "Browser location"),
        ("history", "Browser history"),
        ("fetch", "Fetch API"), // Note: fetch is available in Node 18+, but behavior differs
        ("XMLHttpRequest", "XHR"),
        ("WebSocket", "WebSocket"),
        ("IntersectionObserver", "Intersection Observer"),
        ("ResizeObserver", "Resize Observer"),
        ("MutationObserver", "Mutation Observer"),
        ("requestAnimationFrame", "Animation API"),
        ("cancelAnimationFrame", "Animation API"),
        ("getComputedStyle", "CSSOM"),
        ("matchMedia", "Media Query"),
        ("alert", "Browser dialog"),
        ("confirm", "Browser dialog"),
        ("prompt", "Browser dialog"),
    ];

    // Check template expressions. Match a whole identifier so `confirmDeleting`
    // and `fetchItems` are not treated as `confirm` / `fetch`.
    for expr in &analysis.template_expressions {
        for (api, context) in &browser_apis {
            if contains_ident(expr.content.as_str(), api) {
                usages.push((
                    CompactString::new(*api),
                    expr.start,
                    *context,
                    DiagnosticSource::Template,
                ));
            }
        }
    }

    for (api, offset) in analysis.setup_context.browser_globals() {
        // Empty names are mount-resource markers, not browser globals.
        if api.is_empty() {
            continue;
        }
        let context = browser_apis
            .iter()
            .find(|(name, _)| *name == api.as_str())
            .map(|(_, context)| *context)
            .unwrap_or("Browser global");
        usages.push((api.clone(), *offset, context, DiagnosticSource::Script));
    }

    usages
}

fn contains_ident(haystack: &str, ident: &str) -> bool {
    let bytes = haystack.as_bytes();
    let needle = ident.as_bytes();
    if needle.is_empty() {
        return false;
    }
    let mut start = 0;
    while start + needle.len() <= bytes.len() {
        if bytes.get(start..start + needle.len()) == Some(needle) {
            let before_ok = start == 0
                || bytes
                    .get(start - 1)
                    .is_some_and(|byte| !is_ident_byte(*byte));
            let after = start + needle.len();
            let after_ok = bytes.get(after).is_none_or(|byte| !is_ident_byte(*byte));
            if before_ok && after_ok {
                return true;
            }
        }
        start += 1;
    }
    false
}

fn is_ident_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

/// Check if an offset is inside a client-only context.
fn is_in_client_only_context(analysis: &vize_croquis::Croquis, offset: u32) -> bool {
    // Find the scope at this offset
    for scope in analysis.scopes.iter() {
        if scope.span.start <= offset && offset <= scope.span.end {
            // Check if this scope or any parent is client-only
            if scope.kind == vize_croquis::ScopeKind::ClientOnly {
                return true;
            }

            // Check parents
            for &parent_id in &scope.parents {
                if let Some(parent) = analysis.scopes.get_scope(parent_id)
                    && parent.kind == vize_croquis::ScopeKind::ClientOnly
                {
                    return true;
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::{BoundaryKind, find_browser_api_usage};
    use vize_carton::CompactString;
    use vize_croquis::{Croquis, ScopeId, TemplateExpression, TemplateExpressionKind};

    #[test]
    fn test_boundary_kind() {
        let kind = BoundaryKind::Error;
        assert_eq!(kind, BoundaryKind::Error);
    }

    #[test]
    fn template_browser_api_does_not_match_name_prefixes() {
        let mut analysis = Croquis::new();
        for content in [
            "confirmationMessage",
            "confirmDeleting",
            "fetchItems",
            "window.innerWidth",
            "confirm()",
        ] {
            analysis.template_expressions.push(TemplateExpression {
                content: CompactString::new(content),
                kind: TemplateExpressionKind::VOn,
                start: 0,
                end: content.len() as u32,
                scope_id: ScopeId::ROOT,
                vif_guard: None,
            });
        }

        let names: Vec<_> = find_browser_api_usage(&analysis)
            .into_iter()
            .map(|(name, _, _, _)| name.clone())
            .collect();
        assert!(names.iter().any(|name| name == "window"), "{names:?}");
        assert!(names.iter().any(|name| name == "confirm"), "{names:?}");
        assert_eq!(
            names
                .iter()
                .filter(|name| name.as_str() == "confirm")
                .count(),
            1
        );
        assert!(!names.iter().any(|name| name == "fetch"), "{names:?}");
    }
}
