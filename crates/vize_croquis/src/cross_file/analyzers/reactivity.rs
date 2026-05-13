//! Reactivity tracking and loss detection.
//!
//! Detects issues where reactivity is accidentally broken:
//! - Destructuring reactive objects or refs
//! - Passing reactive values to non-reactive contexts
//! - Reactivity loss through function calls
//! - Ref unwrapping issues
//!
//! ## Performance Optimizations
//!
//! - Uses FxHashSet for O(1) lookups
//! - Early termination when possible
//! - Minimal allocations during analysis

use crate::cross_file::diagnostics::{
    CrossFileDiagnostic, CrossFileDiagnosticKind, DiagnosticSeverity,
};
use crate::cross_file::graph::DependencyGraph;
use crate::cross_file::registry::{FileId, ModuleRegistry};
use crate::reactivity::ReactiveKind;
use vize_carton::{cstr, CompactString, FxHashSet};

/// Kind of reactivity issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReactivityIssueKind {
    /// Destructuring a reactive object loses reactivity.
    DestructuredReactive {
        source_name: CompactString,
        destructured_props: Vec<CompactString>,
    },
    /// Destructuring a ref without .value loses reactivity.
    DestructuredRef { ref_name: CompactString },
    /// Reactive value passed to non-reactive context.
    ReactivityLost {
        value_name: CompactString,
        context: CompactString,
    },
    /// Ref used without .value in script.
    MissingValueAccess { ref_name: CompactString },
    /// toRef/toRefs should be used instead of destructuring.
    ShouldUseToRefs { source_name: CompactString },
    /// Reactive value assigned to plain variable.
    ReactiveToPlain {
        source_name: CompactString,
        target_name: CompactString,
    },
    /// Plain reactive snapshot passed through a function boundary.
    ReactiveSnapshotPassedToCall {
        source_name: CompactString,
        argument_name: CompactString,
        callee_name: CompactString,
    },
    /// Getter-backed context method extracted to a plain binding.
    GetterCallToPlain {
        context_name: CompactString,
        getter_name: CompactString,
        target_name: CompactString,
        callee_name: CompactString,
        source_name: CompactString,
    },
    /// storeToRefs should be used for Pinia store.
    ShouldUseStoreToRefs { store_name: CompactString },
    /// Computed without return statement.
    ComputedWithoutReturn { computed_name: CompactString },
    /// Watch source is not reactive.
    NonReactiveWatchSource { source_expression: CompactString },
    /// Prop passed to ref() which creates a copy.
    PropPassedToRef { prop_name: CompactString },
}

/// Information about a reactivity issue.
#[derive(Debug, Clone)]
pub struct ReactivityIssue {
    /// File where the issue occurs.
    pub file_id: FileId,
    /// Kind of issue.
    pub kind: ReactivityIssueKind,
    /// Offset in source.
    pub offset: u32,
    /// The reactive source involved.
    pub source: Option<CompactString>,
}

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

        // Analyze each component for reactivity issues
        let component_issues = analyze_component_reactivity(analysis);

        for issue in component_issues {
            let diag = create_diagnostic(file_id, &issue);
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

/// Internal issue representation during analysis.
struct InternalIssue {
    kind: ReactivityIssueKind,
    offset: u32,
    end_offset: Option<u32>,
    source: Option<CompactString>,
}

/// Analyze a single component for reactivity issues.
/// Uses precise static analysis from the Croquis data - no heuristics.
#[inline]
fn analyze_component_reactivity(analysis: &crate::Croquis) -> Vec<InternalIssue> {
    let mut issues = Vec::new();

    // Track which identifiers come from 'vue' imports (ref, reactive, toRefs, etc.)
    let vue_imports = extract_vue_imports(analysis);

    // Check for destructured inject() calls - these lose reactivity
    // This is precise: we check the actual InjectPattern from the tracker
    for inject in analysis.provide_inject.injects() {
        use crate::provide::InjectPattern;
        match &inject.pattern {
            InjectPattern::ObjectDestructure(props) => {
                issues.push(InternalIssue {
                    kind: ReactivityIssueKind::DestructuredReactive {
                        source_name: inject.local_name.clone(),
                        destructured_props: props.clone(),
                    },
                    offset: inject.start,
                    end_offset: None,
                    source: Some(inject.local_name.clone()),
                });
            }
            InjectPattern::ArrayDestructure(_items) => {
                issues.push(InternalIssue {
                    kind: ReactivityIssueKind::DestructuredReactive {
                        source_name: inject.local_name.clone(),
                        destructured_props: vec![CompactString::new("(array items)")],
                    },
                    offset: inject.start,
                    end_offset: None,
                    source: Some(inject.local_name.clone()),
                });
            }
            InjectPattern::IndirectDestructure {
                inject_var,
                props,
                offset,
            } => {
                // Indirect destructuring also loses reactivity
                // e.g., const state = inject('state'); const { count } = state;
                issues.push(InternalIssue {
                    kind: ReactivityIssueKind::DestructuredReactive {
                        source_name: inject_var.clone(),
                        destructured_props: props.clone(),
                    },
                    offset: *offset,
                    end_offset: None,
                    source: Some(inject_var.clone()),
                });
            }
            InjectPattern::Simple => {
                // No issue - inject is stored properly
            }
        }
    }

    // Check for toRefs usage - this is the correct pattern, no warning needed
    // Check for reactive sources that indicate proper usage
    let torefs_sources: FxHashSet<&str> = analysis
        .reactivity
        .sources()
        .iter()
        .filter(|s| matches!(s.kind, ReactiveKind::ToRef | ReactiveKind::ToRefs))
        .map(|s| s.name.as_str())
        .collect();

    // Build a set of all reactive sources (from vue imports)
    let _reactive_sources: FxHashSet<&str> = analysis
        .reactivity
        .sources()
        .iter()
        .map(|s| s.name.as_str())
        .collect();

    // Track props defined via defineProps
    let props: FxHashSet<&str> = analysis
        .macros
        .props()
        .iter()
        .map(|p| p.name.as_str())
        .collect();

    // Check if props are properly wrapped with toRef/toRefs when destructured
    if let Some(props_destructure) = analysis.macros.props_destructure() {
        for (key, _binding) in props_destructure.bindings.iter() {
            // Check if this destructured prop has a corresponding toRef
            if !torefs_sources.contains(key.as_str()) {
                // This prop is destructured without toRefs - Vue handles this with
                // reactive props destructure transform, so this is actually OK in modern Vue
                // We don't warn here as it's handled by the compiler
            }
        }
    }

    // Check for reactivity loss patterns detected by the parser
    // These are strict, AST-based detections
    for loss in analysis.reactivity.losses() {
        use crate::reactivity::ReactivityLossKind;
        match &loss.kind {
            ReactivityLossKind::ReactiveDestructure {
                source_name,
                destructured_props,
            } => {
                issues.push(InternalIssue {
                    kind: ReactivityIssueKind::DestructuredReactive {
                        source_name: source_name.clone(),
                        destructured_props: destructured_props.clone(),
                    },
                    offset: loss.start,
                    end_offset: Some(loss.end),
                    source: Some(source_name.clone()),
                });
            }
            ReactivityLossKind::RefValueDestructure {
                source_name,
                destructured_props,
            } => {
                issues.push(InternalIssue {
                    kind: ReactivityIssueKind::DestructuredRef {
                        ref_name: source_name.clone(),
                    },
                    offset: loss.start,
                    end_offset: Some(loss.end),
                    source: Some(cstr!(
                        "{}.value (destructured: {})",
                        source_name,
                        destructured_props.join(", ")
                    )),
                });
            }
            ReactivityLossKind::RefValueExtract {
                source_name,
                target_name,
            } => {
                issues.push(InternalIssue {
                    kind: ReactivityIssueKind::ReactiveToPlain {
                        source_name: cstr!("{source_name}.value"),
                        target_name: target_name.clone(),
                    },
                    offset: loss.start,
                    end_offset: Some(loss.end),
                    source: Some(source_name.clone()),
                });
            }
            ReactivityLossKind::ReactivePropertyExtract {
                source_name,
                prop_name,
                target_name,
            } => {
                issues.push(InternalIssue {
                    kind: ReactivityIssueKind::ReactiveToPlain {
                        source_name: cstr!("{source_name}.{prop_name}"),
                        target_name: target_name.clone(),
                    },
                    offset: loss.start,
                    end_offset: Some(loss.end),
                    source: Some(source_name.clone()),
                });
            }
            ReactivityLossKind::PropsDestructure { .. } => {}
            ReactivityLossKind::FunctionArgumentExtract {
                source_name,
                argument_name,
                callee_name,
            } => {
                issues.push(InternalIssue {
                    kind: ReactivityIssueKind::ReactiveSnapshotPassedToCall {
                        source_name: source_name.clone(),
                        argument_name: argument_name.clone(),
                        callee_name: callee_name.clone(),
                    },
                    offset: loss.start,
                    end_offset: Some(loss.end),
                    source: Some(source_name.clone()),
                });
            }
            ReactivityLossKind::GetterCallExtract {
                context_name,
                getter_name,
                target_name,
                callee_name,
                source_name,
            } => {
                issues.push(InternalIssue {
                    kind: ReactivityIssueKind::GetterCallToPlain {
                        context_name: context_name.clone(),
                        getter_name: getter_name.clone(),
                        target_name: target_name.clone(),
                        callee_name: callee_name.clone(),
                        source_name: source_name.clone(),
                    },
                    offset: loss.start,
                    end_offset: Some(loss.end),
                    source: Some(source_name.clone()),
                });
            }
            ReactivityLossKind::PlainValueAlias {
                source_name,
                alias_name,
                target_name,
            } => {
                issues.push(InternalIssue {
                    kind: ReactivityIssueKind::ReactiveToPlain {
                        source_name: cstr!("{source_name} via {alias_name}"),
                        target_name: target_name.clone(),
                    },
                    offset: loss.start,
                    end_offset: Some(loss.end),
                    source: Some(source_name.clone()),
                });
            }
            ReactivityLossKind::ReactiveSpread { source_name } => {
                issues.push(InternalIssue {
                    kind: ReactivityIssueKind::ShouldUseToRefs {
                        source_name: source_name.clone(),
                    },
                    offset: loss.start,
                    end_offset: Some(loss.end),
                    source: Some(source_name.clone()),
                });
            }
            ReactivityLossKind::ReactiveReassign { source_name } => {
                // Get the original reactive type for better diagnostics
                let original_type = analysis
                    .reactivity
                    .lookup(source_name.as_str())
                    .map(|s| match s.kind {
                        ReactiveKind::Ref => "ref",
                        ReactiveKind::ShallowRef => "shallowRef",
                        ReactiveKind::Reactive => "reactive",
                        ReactiveKind::ShallowReactive => "shallowReactive",
                        ReactiveKind::Computed => "computed",
                        ReactiveKind::Readonly => "readonly",
                        ReactiveKind::ShallowReadonly => "shallowReadonly",
                        ReactiveKind::ToRef => "toRef",
                        ReactiveKind::ToRefs => "toRefs",
                    })
                    .unwrap_or("reactive");

                issues.push(InternalIssue {
                    kind: ReactivityIssueKind::ReactivityLost {
                        value_name: source_name.clone(),
                        context: CompactString::new(original_type),
                    },
                    offset: loss.start,
                    end_offset: Some(loss.end),
                    source: Some(source_name.clone()),
                });
            }
        }
    }

    // Report if vue imports are present but not used properly
    if !vue_imports.is_empty() {
        // Check if reactive sources are actually used
        for source in analysis.reactivity.sources() {
            // Verify the reactive function was imported from 'vue'
            let function_name = match source.kind {
                ReactiveKind::Ref => "ref",
                ReactiveKind::ShallowRef => "shallowRef",
                ReactiveKind::Reactive => "reactive",
                ReactiveKind::ShallowReactive => "shallowReactive",
                ReactiveKind::Computed => "computed",
                ReactiveKind::Readonly => "readonly",
                ReactiveKind::ShallowReadonly => "shallowReadonly",
                ReactiveKind::ToRef => "toRef",
                ReactiveKind::ToRefs => "toRefs",
            };

            // Verify it comes from vue
            if !vue_imports.contains(function_name) {
                // The reactive function might be a local implementation or from another library
                // This is a potential issue but not necessarily an error
            }
        }
    }

    // Check for prop passed to ref() which creates a copy
    for source in analysis.reactivity.sources() {
        if source.kind == ReactiveKind::Ref {
            // Check if this ref is initialized with a prop
            if props.contains(source.name.as_str()) {
                issues.push(InternalIssue {
                    kind: ReactivityIssueKind::PropPassedToRef {
                        prop_name: source.name.clone(),
                    },
                    offset: source.declaration_offset,
                    end_offset: None,
                    source: Some(source.name.clone()),
                });
            }
        }
    }

    issues
}

/// Extract identifiers imported from 'vue'.
fn extract_vue_imports(analysis: &crate::Croquis) -> FxHashSet<&str> {
    use crate::scope::ScopeKind;

    let mut vue_imports = FxHashSet::default();

    for scope in analysis.scopes.iter() {
        if scope.kind == ScopeKind::ExternalModule {
            if let crate::scope::ScopeData::ExternalModule(data) = scope.data() {
                // Check if this is a vue import
                if data.source.as_str() == "vue" || data.source.starts_with("vue/") {
                    // Collect all bindings from this import
                    for (name, _) in scope.bindings() {
                        vue_imports.insert(name);
                    }
                }
            }
        }
    }

    vue_imports
}

/// Create a diagnostic from an internal issue.
fn create_diagnostic(file_id: FileId, issue: &InternalIssue) -> CrossFileDiagnostic {
    match &issue.kind {
        ReactivityIssueKind::DestructuredReactive {
            source_name,
            destructured_props,
        } => {
            let mut diag = CrossFileDiagnostic::new(
                CrossFileDiagnosticKind::DestructuringBreaksReactivity {
                    source_name: source_name.clone(),
                    destructured_keys: destructured_props.clone(),
                    suggestion: CompactString::new("toRefs"),
                },
                DiagnosticSeverity::Error,
                file_id,
                issue.offset,
                cstr!(
                    "Destructuring reactive object '{}' breaks reactivity connection",
                    source_name
                ),
            )
            .with_suggestion(cstr!(
                "Use toRefs({}) or access properties directly as {}.prop",
                source_name,
                source_name
            ));
            if let Some(end) = issue.end_offset {
                diag = diag.with_end_offset(end);
            }
            diag
        }

        ReactivityIssueKind::DestructuredRef { ref_name } => {
            let mut diag = CrossFileDiagnostic::new(
                CrossFileDiagnosticKind::DestructuringBreaksReactivity {
                    source_name: ref_name.clone(),
                    destructured_keys: vec![CompactString::new("value")],
                    suggestion: CompactString::new("computed"),
                },
                DiagnosticSeverity::Error,
                file_id,
                issue.offset,
                cstr!(
                    "Destructuring ref '{}' creates a non-reactive copy",
                    ref_name
                ),
            )
            .with_suggestion(cstr!(
                "Access {}.value directly or use computed(() => {}.value.prop)",
                ref_name,
                ref_name
            ));
            if let Some(end) = issue.end_offset {
                diag = diag.with_end_offset(end);
            }
            diag
        }

        ReactivityIssueKind::ReactivityLost {
            value_name,
            context,
        } => {
            // Check if this is a reassignment (context is a reactive type name)
            let is_reassignment = matches!(
                context.as_str(),
                "ref"
                    | "shallowRef"
                    | "reactive"
                    | "shallowReactive"
                    | "computed"
                    | "readonly"
                    | "shallowReadonly"
                    | "toRef"
                    | "toRefs"
            );

            if is_reassignment {
                let mut diag = CrossFileDiagnostic::new(
                    CrossFileDiagnosticKind::ReassignmentBreaksReactivity {
                        variable_name: value_name.clone(),
                        original_type: context.clone(),
                    },
                    DiagnosticSeverity::Error,
                    file_id,
                    issue.offset,
                    cstr!("Reassigning '{value_name}' breaks reactivity tracking",),
                )
                .with_suggestion(
                    "Mutate the object's properties instead, or use ref() for replaceable values",
                );
                if let Some(end) = issue.end_offset {
                    diag = diag.with_end_offset(end);
                }
                diag
            } else {
                CrossFileDiagnostic::new(
                    CrossFileDiagnosticKind::HydrationMismatchRisk {
                        reason: cstr!("'{value_name}' loses reactivity in {context}",),
                    },
                    DiagnosticSeverity::Error,
                    file_id,
                    issue.offset,
                    cstr!(
                        "Reactive value '{value_name}' loses reactivity when passed to {context}",
                    ),
                )
            }
        }

        ReactivityIssueKind::MissingValueAccess { ref_name } => CrossFileDiagnostic::new(
            CrossFileDiagnosticKind::HydrationMismatchRisk {
                reason: cstr!("Ref '{ref_name}' used without .value"),
            },
            DiagnosticSeverity::Error,
            file_id,
            issue.offset,
            cstr!("Ref '{ref_name}' should be accessed with .value in script context",),
        )
        .with_suggestion(cstr!("Use {ref_name}.value instead of {ref_name}",)),

        ReactivityIssueKind::ShouldUseToRefs { source_name } => {
            let mut diag = CrossFileDiagnostic::new(
                CrossFileDiagnosticKind::SpreadBreaksReactivity {
                    source_name: source_name.clone(),
                    source_type: CompactString::new("reactive"),
                },
                DiagnosticSeverity::Error,
                file_id,
                issue.offset,
                cstr!("Spreading '{source_name}' creates a non-reactive copy"),
            )
            .with_suggestion(cstr!(
                "Use toRefs({source_name}) to maintain reactivity, or toRaw({source_name}) for intentional copy",
            ));
            if let Some(end) = issue.end_offset {
                diag = diag.with_end_offset(end);
            }
            diag
        }

        ReactivityIssueKind::ReactiveToPlain {
            source_name,
            target_name,
        } => {
            let mut diag = CrossFileDiagnostic::new(
                CrossFileDiagnosticKind::ValueExtractionBreaksReactivity {
                    source_name: source_name.clone(),
                    extracted_value: target_name.clone(),
                },
                DiagnosticSeverity::Error,
                file_id,
                issue.offset,
                cstr!(
                    "Assigning reactive '{}' to '{}' creates a non-reactive copy",
                    source_name,
                    target_name
                ),
            )
            .with_suggestion("Use computed() or keep the reactive reference");
            if let Some(end) = issue.end_offset {
                diag = diag.with_end_offset(end);
            }
            diag
        }

        ReactivityIssueKind::ReactiveSnapshotPassedToCall {
            source_name,
            argument_name,
            callee_name,
        } => {
            let mut diag = CrossFileDiagnostic::new(
                CrossFileDiagnosticKind::ValueExtractionBreaksReactivity {
                    source_name: source_name.clone(),
                    extracted_value: argument_name.clone(),
                },
                DiagnosticSeverity::Error,
                file_id,
                issue.offset,
                cstr!(
                    "Passing '{}' to '{}' captures a non-reactive snapshot",
                    argument_name,
                    callee_name
                ),
            )
            .with_suggestion(cstr!(
                "Pass a getter like () => {argument_name}, or pass a ref/computed value explicitly"
            ));
            if let Some(end) = issue.end_offset {
                diag = diag.with_end_offset(end);
            }
            diag
        }

        ReactivityIssueKind::GetterCallToPlain {
            context_name,
            getter_name,
            target_name,
            callee_name,
            source_name,
        } => {
            let mut diag = CrossFileDiagnostic::new(
                CrossFileDiagnosticKind::ValueExtractionBreaksReactivity {
                    source_name: cstr!("{context_name}.{getter_name}()"),
                    extracted_value: target_name.clone(),
                },
                DiagnosticSeverity::Error,
                file_id,
                issue.offset,
                cstr!(
                    "Assigning '{}.{}()' to '{}' extracts the getter-backed value from '{}'",
                    context_name,
                    getter_name,
                    target_name,
                    source_name
                ),
            )
            .with_suggestion(cstr!(
                "Keep the getter from {callee_name} lazy, or wrap {target_name} with computed()"
            ));
            if let Some(end) = issue.end_offset {
                diag = diag.with_end_offset(end);
            }
            diag
        }

        ReactivityIssueKind::ShouldUseStoreToRefs { store_name } => CrossFileDiagnostic::new(
            CrossFileDiagnosticKind::DestructuringBreaksReactivity {
                source_name: store_name.clone(),
                destructured_keys: vec![],
                suggestion: CompactString::new("storeToRefs"),
            },
            DiagnosticSeverity::Error,
            file_id,
            issue.offset,
            cstr!(
                "Destructuring Pinia store '{store_name}' - use storeToRefs() for state/getters"
            ),
        )
        .with_suggestion(cstr!(
            "const {{ state, getter }} = storeToRefs({store_name})"
        )),

        ReactivityIssueKind::ComputedWithoutReturn { computed_name } => CrossFileDiagnostic::new(
            CrossFileDiagnosticKind::HydrationMismatchRisk {
                reason: cstr!("Computed '{computed_name}' may not return value"),
            },
            DiagnosticSeverity::Warning,
            file_id,
            issue.offset,
            cstr!("Computed property '{computed_name}' should return a value"),
        ),

        ReactivityIssueKind::NonReactiveWatchSource { source_expression } => {
            CrossFileDiagnostic::new(
                CrossFileDiagnosticKind::HydrationMismatchRisk {
                    reason: cstr!("Watch source '{source_expression}' is not reactive"),
                },
                DiagnosticSeverity::Warning,
                file_id,
                issue.offset,
                cstr!(
                    "Watch source '{source_expression}' is not reactive, changes won't trigger the callback"
                ),
            )
            .with_suggestion("Use () => value or a ref/reactive object as the watch source")
        }

        ReactivityIssueKind::PropPassedToRef { prop_name } => CrossFileDiagnostic::new(
            CrossFileDiagnosticKind::HydrationMismatchRisk {
                reason: cstr!("Prop '{prop_name}' passed to ref() creates a copy"),
            },
            DiagnosticSeverity::Error,
            file_id,
            issue.offset,
            cstr!("Passing prop '{prop_name}' to ref() creates a non-reactive copy"),
        )
        .with_suggestion(cstr!(
            "Use toRef(props, '{prop_name}') or computed(() => props.{prop_name})"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::ReactivityIssueKind;
    use vize_carton::CompactString;

    #[test]
    fn test_reactivity_issue_kind() {
        let kind = ReactivityIssueKind::DestructuredReactive {
            source_name: CompactString::new("state"),
            destructured_props: vec![CompactString::new("count")],
        };

        match kind {
            ReactivityIssueKind::DestructuredReactive { source_name, .. } => {
                assert_eq!(source_name.as_str(), "state");
            }
            _ => panic!("Wrong kind"),
        }
    }
}
