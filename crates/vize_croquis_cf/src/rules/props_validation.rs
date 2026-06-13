//! Props validation analyzer.
//!
//! Validates that props passed to child components match their declarations.

use crate::diagnostics::{CrossFileDiagnostic, CrossFileDiagnosticKind, DiagnosticSeverity};
use crate::graph::DependencyGraph;
use crate::registry::{FileId, ModuleRegistry};
use vize_carton::{CompactString, FxHashMap, cstr};

mod helpers;

use helpers::{
    actual_literal_type, declared_prop, define_props_offset, extract_passed_props_for_component,
    has_passed_prop, imported_aliases_for_child, is_builtin_attr, prop_type_accepts_actual,
};

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
                    if prop_info.required && !has_passed_prop(&passed_usage, prop_name.as_str()) {
                        let issue = PropsValidationIssue {
                            parent_file: parent_id,
                            child_file: child_id,
                            component_name: child_component_name.clone(),
                            kind: PropsValidationIssueKind::MissingRequiredProp {
                                prop_name: prop_name.clone(),
                            },
                            offset: passed_usage.start,
                        };
                        issues.push(issue);

                        let diagnostic = CrossFileDiagnostic::with_span(
                            CrossFileDiagnosticKind::MissingRequiredProp {
                                prop_name: prop_name.clone(),
                                component_name: child_component_name.clone(),
                            },
                            DiagnosticSeverity::Error,
                            parent_id,
                            passed_usage.start,
                            passed_usage.end,
                            cstr!(
                                "**Missing Required Prop**: `{}` must be passed to `<{}>`\n\n\
                                This prop is declared as required in the component's `defineProps`.",
                                prop_name,
                                child_component_name
                            ),
                        )
                        .with_related(
                            child_id,
                            define_props_offset(child_entry),
                            cstr!("Prop `{prop_name}` is declared as required here"),
                        );

                        diagnostics.push(diagnostic);
                    }
                }
            }

            // Check for undeclared props (explicit props passed but not in defineProps)
            for passed_prop in &passed_usage.props {
                // Skip built-in attributes
                if is_builtin_attr(passed_prop.name) {
                    continue;
                }

                // Check if this prop is declared
                let Some((declared_prop_name, prop_info)) =
                    declared_prop(&child_props_info.props, passed_prop.name)
                else {
                    let issue = PropsValidationIssue {
                        parent_file: parent_id,
                        child_file: child_id,
                        component_name: child_component_name.clone(),
                        kind: PropsValidationIssueKind::UndeclaredProp {
                            prop_name: CompactString::new(passed_prop.name),
                        },
                        offset: passed_prop.start,
                    };
                    issues.push(issue);

                    let diagnostic = CrossFileDiagnostic::with_span(
                        CrossFileDiagnosticKind::UndeclaredProp {
                            prop_name: CompactString::new(passed_prop.name),
                            component_name: child_component_name.clone(),
                        },
                        DiagnosticSeverity::Warning, // Warning since it might be intentional $attrs
                        parent_id,
                        passed_prop.start,
                        passed_prop.end,
                        cstr!(
                            "**Undeclared Prop**: `{}` is passed to `<{}>` but not declared\n\n\
                            The prop is not defined in the component's `defineProps`.\n\
                            If intentional, it will fall through to the root element via `$attrs`.",
                            passed_prop.name, child_component_name
                        ),
                    )
                    .with_suggestion(cstr!(
                        "Add to defineProps:\n```typescript\ndefineProps<{{\n  {}: unknown\n}}>()\n```\n\n\
                        Or use `v-bind=\"$attrs\"` in the child component for fallthrough.",
                        passed_prop.name
                    ));

                    diagnostics.push(diagnostic);
                    continue;
                };

                if let (Some(expected), Some(actual)) = (
                    prop_info.prop_type.as_ref(),
                    actual_literal_type(passed_prop),
                ) && !prop_type_accepts_actual(expected.as_str(), actual.as_str())
                {
                    let issue = PropsValidationIssue {
                        parent_file: parent_id,
                        child_file: child_id,
                        component_name: child_component_name.clone(),
                        kind: PropsValidationIssueKind::TypeMismatch {
                            prop_name: CompactString::new(passed_prop.name),
                            expected: expected.clone(),
                            actual: actual.clone(),
                        },
                        offset: passed_prop.start,
                    };
                    issues.push(issue);

                    let diagnostic = CrossFileDiagnostic::with_span(
                        CrossFileDiagnosticKind::PropTypeMismatch {
                            prop_name: CompactString::new(passed_prop.name),
                            expected_type: expected.clone(),
                            actual_type: actual.clone(),
                        },
                        DiagnosticSeverity::Error,
                        parent_id,
                        passed_prop.start,
                        passed_prop.end,
                        cstr!(
                            "**Prop Type Mismatch**: `{}` expects `{}` but received `{}`\n\n\
                            The static prop value does not match the child component's declared runtime prop type.",
                            declared_prop_name, expected, actual
                        ),
                    )
                    .with_related(
                        child_id,
                        define_props_offset(child_entry),
                        cstr!("Prop `{declared_prop_name}` is declared with type `{expected}` here"),
                    );

                    diagnostics.push(diagnostic);
                }
            }
        }
    }

    (issues, diagnostics)
}
