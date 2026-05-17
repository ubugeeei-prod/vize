use super::types::{InternalIssue, ReactivityIssueKind};
use crate::diagnostics::{CrossFileDiagnostic, CrossFileDiagnosticKind, DiagnosticSeverity};
use crate::registry::FileId;
use vize_carton::{CompactString, cstr};

pub(super) fn create_diagnostic(file_id: FileId, issue: &InternalIssue) -> CrossFileDiagnostic {
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
