//! Props validation analyzer.
//!
//! Validates that props passed to child components match their declarations.

use crate::diagnostics::{CrossFileDiagnostic, CrossFileDiagnosticKind, DiagnosticSeverity};
use crate::graph::DependencyGraph;
use crate::registry::{FileId, ModuleEntry, ModuleRegistry};
use std::path::{Component, Path, PathBuf};
use vize_carton::{CompactString, FxHashMap, FxHashSet, String, cstr};

/// Information about a props validation issue.
#[derive(Debug, Clone)]
pub struct PropsValidationIssue {
    /// The file where the parent component is.
    pub parent_file: FileId,
    /// The file where the child component is.
    pub child_file: FileId,
    /// The component name.
    pub component_name: CompactString,
    /// Kind of issue.
    pub kind: PropsValidationIssueKind,
    /// Source offset in parent file.
    pub offset: u32,
}

/// Kind of props validation issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropsValidationIssueKind {
    /// Prop passed but not declared in child.
    UndeclaredProp { prop_name: CompactString },
    /// Required prop not passed.
    MissingRequiredProp { prop_name: CompactString },
    /// Type mismatch (if detectable statically).
    TypeMismatch {
        prop_name: CompactString,
        expected: CompactString,
        actual: CompactString,
    },
}

/// Information about a child component's props.
#[derive(Debug, Default)]
struct ComponentPropsInfo {
    /// Declared props with their required status.
    props: FxHashMap<CompactString, PropInfo>,
}

#[derive(Debug, Clone)]
struct PropInfo {
    required: bool,
    #[allow(dead_code)] // Will be used for type checking in future
    prop_type: Option<CompactString>,
}

/// Analyze props validation across component boundaries.
///
/// This analyzer checks:
/// 1. Props passed to children are declared in their defineProps
/// 2. Required props are always passed
pub fn analyze_props_validation(
    registry: &ModuleRegistry,
    graph: &DependencyGraph,
) -> (Vec<PropsValidationIssue>, Vec<CrossFileDiagnostic>) {
    let mut issues = Vec::new();
    let mut diagnostics = Vec::new();

    // Build a map of component name -> props info
    let mut component_props: FxHashMap<CompactString, (FileId, ComponentPropsInfo)> =
        FxHashMap::default();

    for entry in registry.iter() {
        if !entry.is_vue_sfc {
            continue;
        }

        let Some(ref component_name) = entry.component_name else {
            continue;
        };

        let mut props_info = ComponentPropsInfo::default();

        // Extract props from macros
        for prop in entry.analysis.macros.props() {
            props_info.props.insert(
                prop.name.clone(),
                PropInfo {
                    required: prop.required,
                    prop_type: prop.prop_type.clone(),
                },
            );
        }

        component_props.insert(component_name.clone(), (entry.id, props_info));
    }

    // Now check each component usage
    for (parent_id, child_id) in graph.component_usage() {
        let Some(parent_entry) = registry.get(parent_id) else {
            continue;
        };
        let Some(child_entry) = registry.get(child_id) else {
            continue;
        };
        let Some(ref child_component_name) = child_entry.component_name else {
            continue;
        };

        // Get the child's props info
        let Some((_, child_props_info)) = component_props.get(child_component_name) else {
            continue;
        };

        // Get props passed by parent
        // This requires parsing the template to find the actual props passed
        // For now, we focus on checking required props from the child's perspective
        let aliases = imported_aliases_for_child(parent_entry, child_entry);
        let passed_usages = extract_passed_props_for_component(
            &parent_entry.analysis,
            child_component_name.as_str(),
            &aliases,
        );

        for passed_usage in passed_usages {
            // Check for missing required props. A spread v-bind can provide any
            // required prop, so only validate required props for non-spread usages.
            if !passed_usage.has_spread_attrs {
                for (prop_name, prop_info) in &child_props_info.props {
                    if prop_info.required && !passed_usage.props.contains(prop_name.as_str()) {
                        let issue = PropsValidationIssue {
                            parent_file: parent_id,
                            child_file: child_id,
                            component_name: child_component_name.clone(),
                            kind: PropsValidationIssueKind::MissingRequiredProp {
                                prop_name: prop_name.clone(),
                            },
                            offset: passed_usage.offset,
                        };
                        issues.push(issue);

                        let diagnostic = CrossFileDiagnostic::new(
                            CrossFileDiagnosticKind::MissingRequiredProp {
                                prop_name: prop_name.clone(),
                                component_name: child_component_name.clone(),
                            },
                            DiagnosticSeverity::Error,
                            parent_id,
                            passed_usage.offset,
                            cstr!(
                                "**Missing Required Prop**: `{}` must be passed to `<{}>`\n\n\
                                This prop is declared as required in the component's `defineProps`.",
                                prop_name,
                                child_component_name
                            ),
                        )
                        .with_related(
                            child_id,
                            0,
                            cstr!("Prop `{prop_name}` is declared as required here"),
                        );

                        diagnostics.push(diagnostic);
                    }
                }
            }

            // Check for undeclared props (explicit props passed but not in defineProps)
            for passed_prop in &passed_usage.props {
                // Skip built-in attributes
                if is_builtin_attr(passed_prop) {
                    continue;
                }

                // Check if this prop is declared
                let is_declared = child_props_info.props.contains_key(*passed_prop);

                if !is_declared {
                    let issue = PropsValidationIssue {
                        parent_file: parent_id,
                        child_file: child_id,
                        component_name: child_component_name.clone(),
                        kind: PropsValidationIssueKind::UndeclaredProp {
                            prop_name: CompactString::new(*passed_prop),
                        },
                        offset: passed_usage.offset,
                    };
                    issues.push(issue);

                    let diagnostic = CrossFileDiagnostic::new(
                        CrossFileDiagnosticKind::UndeclaredProp {
                            prop_name: CompactString::new(*passed_prop),
                            component_name: child_component_name.clone(),
                        },
                        DiagnosticSeverity::Warning, // Warning since it might be intentional $attrs
                        parent_id,
                        passed_usage.offset,
                        cstr!(
                            "**Undeclared Prop**: `{}` is passed to `<{}>` but not declared\n\n\
                            The prop is not defined in the component's `defineProps`.\n\
                            If intentional, it will fall through to the root element via `$attrs`.",
                            passed_prop, child_component_name
                        ),
                    )
                    .with_suggestion(cstr!(
                        "Add to defineProps:\n```typescript\ndefineProps<{{\n  {}: unknown\n}}>()\n```\n\n\
                        Or use `v-bind=\"$attrs\"` in the child component for fallthrough.",
                        passed_prop
                    ));

                    diagnostics.push(diagnostic);
                }
            }
        }
    }

    (issues, diagnostics)
}

struct PassedComponentUsage<'a> {
    props: FxHashSet<&'a str>,
    has_spread_attrs: bool,
    offset: u32,
}

/// Extract matched usages and explicit props passed to a specific component from the analysis.
///
/// Uses component_usages to find props passed to the component.
fn extract_passed_props_for_component<'a>(
    analysis: &'a vize_croquis::Croquis,
    component_name: &str,
    aliases: &[CompactString],
) -> Vec<PassedComponentUsage<'a>> {
    let mut usages = Vec::new();

    for usage in &analysis.component_usages {
        // Match component name (case-insensitive for kebab-case vs PascalCase)
        if component_names_match(usage.name.as_str(), component_name)
            || aliases
                .iter()
                .any(|alias| component_names_match(usage.name.as_str(), alias.as_str()))
        {
            let mut props = FxHashSet::default();
            for prop in &usage.props {
                props.insert(prop.name.as_str());
            }

            usages.push(PassedComponentUsage {
                props,
                has_spread_attrs: usage.has_spread_attrs,
                offset: usage.start,
            });
        }
    }

    if usages.is_empty() {
        usages.push(PassedComponentUsage {
            props: FxHashSet::default(),
            has_spread_attrs: false,
            offset: 0,
        });
    }

    usages
}

fn imported_aliases_for_child(
    parent_entry: &ModuleEntry,
    child_entry: &ModuleEntry,
) -> Vec<CompactString> {
    let parent_dir = parent_entry.path.parent();
    let mut aliases = Vec::new();

    for scope in parent_entry.analysis.scopes.iter() {
        let vize_croquis::ScopeData::ExternalModule(data) = scope.data() else {
            continue;
        };

        if !import_targets_path(data.source.as_str(), parent_dir, child_entry.path.as_path()) {
            continue;
        }

        aliases.extend(scope.bindings().map(|(name, _)| CompactString::new(name)));
    }

    aliases
}

fn import_targets_path(specifier: &str, from_dir: Option<&Path>, target: &Path) -> bool {
    let normalized_target = normalize_logical_path(target.to_path_buf());
    import_candidates(specifier, from_dir)
        .into_iter()
        .any(|candidate| candidate == normalized_target || normalized_target.ends_with(&candidate))
}

fn import_candidates(specifier: &str, from_dir: Option<&Path>) -> Vec<PathBuf> {
    let mut bases = Vec::new();

    if let Some(relative) = specifier.strip_prefix("@/") {
        bases.push(PathBuf::from("src").join(relative));
    } else if specifier.starts_with('.') {
        let base = from_dir
            .filter(|dir| !dir.as_os_str().is_empty())
            .map_or_else(|| PathBuf::from(specifier), |dir| dir.join(specifier));
        bases.push(base);
    } else if let Some(stripped) = specifier.strip_prefix('/') {
        bases.push(PathBuf::from(stripped));
        bases.push(PathBuf::from(specifier));
    } else {
        bases.push(PathBuf::from(specifier));
    }

    let mut candidates = Vec::new();
    for base in bases {
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
    }

    candidates
}

fn path_with_suffix(base: &Path, suffix: &str) -> PathBuf {
    if let Some(index_file) = suffix.strip_prefix('/') {
        base.join(index_file)
    } else {
        let mut value = base.as_os_str().to_os_string();
        value.push(suffix);
        PathBuf::from(value)
    }
}

fn normalize_logical_path(path: PathBuf) -> PathBuf {
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

fn component_names_match(left: &str, right: &str) -> bool {
    left == right || to_pascal_case(left) == to_pascal_case(right)
}

/// Convert kebab-case to PascalCase.
#[inline]
fn to_pascal_case(s: &str) -> String {
    s.split('-')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::default(),
                Some(c) => c.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

/// Check if an attribute name is a built-in HTML/Vue attribute.
#[inline]
fn is_builtin_attr(name: &str) -> bool {
    matches!(
        name,
        "key"
            | "ref"
            | "is"
            | "class"
            | "style"
            | "id"
            | "slot"
            | "slot-scope"
            | "v-slot"
            | "v-if"
            | "v-else"
            | "v-else-if"
            | "v-for"
            | "v-show"
            | "v-bind"
            | "v-on"
            | "v-model"
            | "v-html"
            | "v-text"
            | "v-pre"
            | "v-cloak"
            | "v-once"
            | "v-memo"
    )
}

#[cfg(test)]
mod tests {
    use super::is_builtin_attr;

    #[test]
    fn test_is_builtin_attr() {
        assert!(is_builtin_attr("key"));
        assert!(is_builtin_attr("ref"));
        assert!(is_builtin_attr("v-model"));
        assert!(!is_builtin_attr("myProp"));
        assert!(!is_builtin_attr("customAttr"));
    }
}
