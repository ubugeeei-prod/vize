//! Component resolution analyzer.
//!
//! Detects unregistered components and unresolved imports.

use crate::diagnostics::{CrossFileDiagnostic, CrossFileDiagnosticKind, DiagnosticSeverity};
use crate::graph::DependencyGraph;
use crate::registry::{FileId, ModuleRegistry};
use vize_carton::{CompactString, FxHashSet, String, cstr};

/// Information about a component resolution issue.
#[derive(Debug, Clone)]
pub struct ComponentResolutionIssue {
    /// The file where the issue was found.
    pub file_id: FileId,
    /// The component name or import specifier.
    pub name: CompactString,
    /// Kind of issue.
    pub kind: ComponentResolutionIssueKind,
    /// Source offset.
    pub offset: u32,
}

/// Kind of component resolution issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentResolutionIssueKind {
    /// Component used in template but not imported/registered.
    UnregisteredComponent,
    /// Import specifier could not be resolved.
    UnresolvedImport,
}

/// Analyze component resolution across all files.
///
/// This analyzer checks:
/// 1. All components used in templates are properly imported/registered
/// 2. All import specifiers can be resolved to actual files
pub fn analyze_component_resolution(
    registry: &ModuleRegistry,
    _graph: &DependencyGraph,
) -> (Vec<ComponentResolutionIssue>, Vec<CrossFileDiagnostic>) {
    let mut issues = Vec::new();
    let mut diagnostics = Vec::new();

    // Check each file
    for entry in registry.iter() {
        let file_id = entry.id;
        let analysis = &entry.analysis;

        // Get all imported identifiers from this file
        let imported_identifiers: FxHashSet<&str> = analysis
            .scopes
            .iter()
            .flat_map(|scope| scope.bindings().map(|(name, _)| name))
            .collect();

        // Check used components
        for component_name in &analysis.used_components {
            // Skip built-in components
            if is_builtin_component(component_name.as_str()) {
                continue;
            }

            // Lowercase hyphenated tags can be native custom elements configured
            // through Vue's compilerOptions.isCustomElement.
            if is_custom_element_tag(component_name.as_str()) {
                continue;
            }

            // Check if component is imported as a binding. Vue templates can
            // use either PascalCase (`UserCard`) or kebab-case (`user-card`).
            let is_imported = imported_identifiers
                .iter()
                .any(|name| component_names_match(component_name.as_str(), name));

            // A component being present somewhere in the project is not enough:
            // local template usage must come from an import/local binding unless
            // a framework-specific global component registry is modeled.
            let is_available = is_imported || analysis.bindings.contains(component_name.as_str());

            if !is_available {
                let template_offset =
                    component_usage_offset(analysis, component_name.as_str()).unwrap_or(0);
                let issue = ComponentResolutionIssue {
                    file_id,
                    name: component_name.clone(),
                    kind: ComponentResolutionIssueKind::UnregisteredComponent,
                    offset: template_offset,
                };
                issues.push(issue);

                let diagnostic = CrossFileDiagnostic::new(
                    CrossFileDiagnosticKind::UnregisteredComponent {
                        component_name: component_name.clone(),
                        template_offset,
                    },
                    DiagnosticSeverity::Error,
                    file_id,
                    template_offset,
                    cstr!(
                        "**Unregistered Component**: `<{}>` is used in template but not imported\n\n\
                        The component must be imported in `<script setup>` or registered globally.",
                        component_name
                    ),
                )
                .with_suggestion(cstr!(
                    "```typescript\nimport {} from './{}.vue'\n```",
                    component_name, component_name
                ));

                diagnostics.push(diagnostic);
            }
        }

        // Check for unresolved imports
        for scope in analysis.scopes.iter() {
            if scope.kind == vize_croquis::ScopeKind::ExternalModule
                && let vize_croquis::ScopeData::ExternalModule(data) = scope.data()
            {
                let source = &data.source;

                // Skip node_modules imports (bare specifiers)
                if !source.starts_with('.') && !source.starts_with('/') && !source.starts_with('@')
                {
                    continue;
                }

                // Skip @-prefixed imports that are likely aliases
                if source.starts_with('@') && !source.starts_with("@/") {
                    continue;
                }

                // Check if the import resolves to a known file
                let resolved = resolve_import(source, registry, entry.path.parent());

                if !resolved {
                    let issue = ComponentResolutionIssue {
                        file_id,
                        name: source.clone(),
                        kind: ComponentResolutionIssueKind::UnresolvedImport,
                        offset: scope.span.start,
                    };
                    issues.push(issue);

                    let diagnostic = CrossFileDiagnostic::new(
                        CrossFileDiagnosticKind::UnresolvedImport {
                            specifier: source.clone(),
                            import_offset: scope.span.start,
                        },
                        DiagnosticSeverity::Error,
                        file_id,
                        scope.span.start,
                        cstr!(
                            "**Unresolved Import**: Cannot find module `{}`\n\n\
                                - Check if the file exists at the specified path\n\
                                - Verify the import path is correct (relative paths start with `./` or `../`)\n\
                                - For alias imports like `@/`, ensure tsconfig paths are configured",
                            source
                        ),
                    );

                    diagnostics.push(diagnostic);
                }
            }
        }
    }

    (issues, diagnostics)
}

fn component_usage_offset(analysis: &vize_croquis::Croquis, component_name: &str) -> Option<u32> {
    analysis
        .component_usages
        .iter()
        .find(|usage| component_names_match(usage.name.as_str(), component_name))
        .map(|usage| usage.start)
}

/// Check if a component name is a Vue built-in component.
#[inline]
fn is_builtin_component(name: &str) -> bool {
    let normalized = to_pascal_case(name);
    matches!(
        normalized.as_str(),
        "Transition"
            | "TransitionGroup"
            | "KeepAlive"
            | "Suspense"
            | "Teleport"
            | "Component"
            | "Slot"
            | "Template"
            // Nuxt built-ins
            | "NuxtPage"
            | "NuxtLayout"
            | "NuxtLink"
            | "NuxtLoadingIndicator"
            | "NuxtErrorBoundary"
            | "NuxtWelcome"
            | "NuxtIsland"
            | "ClientOnly"
            | "DevOnly"
            | "ServerPlaceholder"
            // Vue Router
            | "RouterView"
            | "RouterLink"
            // Head management
            | "Head"
            | "Html"
            | "Body"
            | "Title"
            | "Meta"
            | "Style"
            | "Link"
            | "Base"
            | "NoScript"
            | "Script"
    )
}

fn is_custom_element_tag(name: &str) -> bool {
    name.contains('-')
        && name.chars().next().is_some_and(|c| c.is_ascii_lowercase())
        && !is_reserved_custom_element_name(name)
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '-' | '_' | '.'))
}

fn is_reserved_custom_element_name(name: &str) -> bool {
    matches!(
        name,
        "annotation-xml"
            | "color-profile"
            | "font-face"
            | "font-face-src"
            | "font-face-uri"
            | "font-face-format"
            | "font-face-name"
            | "missing-glyph"
    )
}

fn component_names_match(left: &str, right: &str) -> bool {
    left == right || to_pascal_case(left) == to_pascal_case(right)
}

fn to_pascal_case(value: &str) -> String {
    value
        .split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::default(),
            }
        })
        .collect()
}

/// Try to resolve an import specifier to a file in the registry.
#[allow(clippy::disallowed_macros)]
fn resolve_import(
    specifier: &str,
    registry: &ModuleRegistry,
    from_dir: Option<&std::path::Path>,
) -> bool {
    // Handle @/ alias (common Vue project alias)
    if let Some(relative) = specifier.strip_prefix("@/") {
        // Check with common extensions
        for ext in &["", ".vue", ".ts", ".tsx", ".js", ".jsx"] {
            let path = format!("src/{}{}", relative, ext);
            if registry.get_by_path(&path).is_some() {
                return true;
            }
        }
        return false;
    }

    // Handle relative imports.
    //
    // Resolve the specifier against the importing file's directory and
    // normalize `.`/`..` segments to a canonical path. Matching only the
    // canonical path ensures `./Button.vue` resolves to the sibling file and
    // never to a different directory's same-named file (e.g. `admin/Button.vue`).
    if specifier.starts_with('.') {
        let candidates = import_candidates(specifier, from_dir);
        return registry.iter().any(|entry| {
            let entry_path = normalize_logical_path(entry.path.clone());
            candidates.contains(&entry_path)
        });
    }

    // For absolute or other paths, check directly
    registry.get_by_path(specifier).is_some()
}

/// Build the set of canonical candidate paths a relative import specifier may
/// resolve to, trying the common module extensions and `index` files.
fn import_candidates(
    specifier: &str,
    from_dir: Option<&std::path::Path>,
) -> Vec<std::path::PathBuf> {
    use std::path::PathBuf;

    let base = from_dir
        .filter(|dir| !dir.as_os_str().is_empty())
        .map_or_else(|| PathBuf::from(specifier), |dir| dir.join(specifier));

    let mut candidates = Vec::new();
    let has_extension = base.extension().is_some();
    candidates.push(normalize_logical_path(base.clone()));

    if !has_extension {
        for suffix in [
            ".vue",
            ".ts",
            ".tsx",
            ".js",
            ".jsx",
            "/index.vue",
            "/index.ts",
            "/index.tsx",
            "/index.js",
            "/index.jsx",
        ] {
            candidates.push(normalize_logical_path(path_with_suffix(&base, suffix)));
        }
    }

    candidates
}

fn path_with_suffix(base: &std::path::Path, suffix: &str) -> std::path::PathBuf {
    if let Some(index_file) = suffix.strip_prefix('/') {
        base.join(index_file)
    } else {
        let mut value = base.as_os_str().to_os_string();
        value.push(suffix);
        std::path::PathBuf::from(value)
    }
}

/// Normalize a path by collapsing `.`/`..` segments without touching the
/// filesystem, yielding a canonical logical path for comparison.
fn normalize_logical_path(path: std::path::PathBuf) -> std::path::PathBuf {
    use std::path::{Component, Path, PathBuf};

    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(part) => normalized.push(part),
            Component::RootDir => normalized.push(Path::new("/")),
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
        }
    }

    normalized
}

#[cfg(test)]
mod tests {
    use super::{
        ComponentResolutionIssueKind, analyze_component_resolution, is_builtin_component,
        is_custom_element_tag, resolve_import,
    };
    use crate::CrossFileDiagnosticKind;
    use crate::graph::DependencyGraph;
    use crate::registry::ModuleRegistry;
    use vize_carton::{CompactString, smallvec};
    use vize_croquis::analysis::ComponentUsage;
    use vize_croquis::{Croquis, ScopeId};

    #[test]
    fn test_is_builtin_component() {
        assert!(is_builtin_component("Transition"));
        assert!(is_builtin_component("transition"));
        assert!(is_builtin_component("transition-group"));
        assert!(is_builtin_component("KeepAlive"));
        assert!(is_builtin_component("keep-alive"));
        assert!(is_builtin_component("RouterView"));
        assert!(is_builtin_component("router-view"));
        assert!(is_builtin_component("NuxtPage"));
        assert!(is_builtin_component("nuxt-page"));
        assert!(is_builtin_component("nuxt-link"));
        assert!(is_builtin_component("client-only"));
        assert!(is_builtin_component("slot"));
        assert!(!is_builtin_component("MyComponent"));
        assert!(!is_builtin_component("UserCard"));
    }

    #[test]
    fn test_is_custom_element_tag() {
        assert!(is_custom_element_tag("my-widget"));
        assert!(is_custom_element_tag("ion-button"));
        assert!(is_custom_element_tag("sl-icon2"));
        assert!(is_custom_element_tag("my_widget-button"));
        assert!(!is_custom_element_tag("MyWidget"));
        assert!(!is_custom_element_tag("myWidget"));
        assert!(!is_custom_element_tag("ChildWidget"));
        assert!(!is_custom_element_tag("font-face"));
        assert!(!is_custom_element_tag("div"));
    }

    #[test]
    fn unregistered_component_uses_template_usage_offset() {
        let mut registry = ModuleRegistry::new();
        let graph = DependencyGraph::new();
        let mut analysis = Croquis::new();

        analysis
            .used_components
            .insert(CompactString::new("UnknownThing"));
        analysis.component_usages.push(ComponentUsage {
            name: CompactString::new("UnknownThing"),
            start: 12,
            end: 27,
            props: smallvec![],
            events: smallvec![],
            slots: smallvec![],
            has_spread_attrs: false,
            scope_id: ScopeId::ROOT,
            vif_guard: None,
        });

        let (file_id, _) = registry.register("Parent.vue", "", analysis);
        let (issues, diagnostics) = analyze_component_resolution(&registry, &graph);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].file_id, file_id);
        assert_eq!(
            issues[0].kind,
            ComponentResolutionIssueKind::UnregisteredComponent
        );
        assert_eq!(issues[0].offset, 12);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].primary_file, file_id);
        assert_eq!(diagnostics[0].primary_offset, 12);
        assert_eq!(
            diagnostics[0].kind,
            CrossFileDiagnosticKind::UnregisteredComponent {
                component_name: CompactString::new("UnknownThing"),
                template_offset: 12,
            }
        );
    }

    #[test]
    fn relative_import_resolves_to_sibling_not_same_named_file_elsewhere() {
        let mut registry = ModuleRegistry::new();

        // Two components named `Button.vue` in different directories.
        registry.register("pages/Button.vue", "", Croquis::new());
        registry.register("admin/Button.vue", "", Croquis::new());

        // `./Button.vue` imported from `pages/Home.vue` must resolve to the
        // sibling `pages/Button.vue`.
        let from_dir = std::path::Path::new("pages/Home.vue").parent();
        assert!(resolve_import("./Button.vue", &registry, from_dir));
    }

    #[test]
    fn relative_import_does_not_cross_directories() {
        let mut registry = ModuleRegistry::new();

        // Only the `admin/` variant exists; the sibling does not.
        registry.register("admin/Button.vue", "", Croquis::new());

        // `./Button.vue` imported from `pages/Home.vue` must NOT resolve to
        // `admin/Button.vue` via a suffix match.
        let from_dir = std::path::Path::new("pages/Home.vue").parent();
        assert!(!resolve_import("./Button.vue", &registry, from_dir));
    }

    #[test]
    fn relative_import_resolves_parent_directory() {
        let mut registry = ModuleRegistry::new();

        registry.register("components/Button.vue", "", Croquis::new());

        // `../Button.vue` from `components/forms/Field.vue` resolves to
        // `components/Button.vue`.
        let from_dir = std::path::Path::new("components/forms/Field.vue").parent();
        assert!(resolve_import("../Button.vue", &registry, from_dir));
    }
}
