//! Diagnostics generation and reporting for cross-file reactivity analysis.

use super::engine::CrossFileReactivityAnalyzer;
use super::types::CrossFileReactivityIssueKind;
use crate::diagnostics::{CrossFileDiagnostic, CrossFileDiagnosticKind};
use vize_carton::CompactString;
use vize_carton::cstr;

impl<'a> CrossFileReactivityAnalyzer<'a> {
    /// Generate diagnostics from detected issues.
    pub(super) fn generate_diagnostics(&self) -> Vec<CrossFileDiagnostic> {
        let mut diagnostics = Vec::new();

        for issue in &self.issues {
            let mut diag = match &issue.kind {
                CrossFileReactivityIssueKind::ComposableReturnDestructured {
                    composable_name,
                    destructured_props,
                } => CrossFileDiagnostic::new(
                    CrossFileDiagnosticKind::DestructuringBreaksReactivity {
                        source_name: composable_name.clone(),
                        destructured_keys: destructured_props.clone(),
                        suggestion: CompactString::new("toRefs"),
                    },
                    issue.severity,
                    issue.file_id,
                    issue.offset,
                    cstr!(
                        "Destructuring {} return loses reactivity for: {}",
                        composable_name,
                        destructured_props.join(", ")
                    ),
                )
                .with_suggestion(cstr!(
                    "const result = {}(); then access result.prop or use toRefs(result)",
                    composable_name
                )),

                CrossFileReactivityIssueKind::InjectValueDestructured {
                    key,
                    destructured_props,
                } => CrossFileDiagnostic::new(
                    CrossFileDiagnosticKind::DestructuringBreaksReactivity {
                        source_name: key.clone(),
                        destructured_keys: destructured_props.clone(),
                        suggestion: CompactString::new("toRefs"),
                    },
                    issue.severity,
                    issue.file_id,
                    issue.offset,
                    cstr!(
                        "Destructuring inject('{}') loses reactivity for: {}",
                        key,
                        destructured_props.join(", ")
                    ),
                )
                .with_suggestion(cstr!(
                    "const injected = inject('{}'); access injected.prop directly",
                    key
                )),

                CrossFileReactivityIssueKind::StoreDestructured {
                    store_name,
                    destructured_props,
                } => CrossFileDiagnostic::new(
                    CrossFileDiagnosticKind::DestructuringBreaksReactivity {
                        source_name: store_name.clone(),
                        destructured_keys: destructured_props.clone(),
                        suggestion: CompactString::new("storeToRefs"),
                    },
                    issue.severity,
                    issue.file_id,
                    issue.offset,
                    cstr!(
                        "Destructuring {} loses reactivity for state/getters",
                        store_name
                    ),
                )
                .with_suggestion(cstr!(
                    "const {{ ... }} = storeToRefs({})",
                    store_name
                )),

                CrossFileReactivityIssueKind::PropsDestructured { destructured_props } => {
                    CrossFileDiagnostic::new(
                        CrossFileDiagnosticKind::DestructuringBreaksReactivity {
                            source_name: CompactString::new("props"),
                            destructured_keys: destructured_props.clone(),
                            suggestion: CompactString::new("toRefs"),
                        },
                        issue.severity,
                        issue.file_id,
                        issue.offset,
                        cstr!(
                            "Destructuring props: {} (Vue compiler handles this, but explicit toRefs is clearer)",
                            destructured_props.join(", ")
                        ),
                    )
                    .with_suggestion("const { ... } = toRefs(props) for explicit reactivity")
                }

                CrossFileReactivityIssueKind::NonReactiveProvide { key } => {
                    CrossFileDiagnostic::new(
                        CrossFileDiagnosticKind::NonReactiveProvideValue {
                            key: key.clone(),
                        },
                        issue.severity,
                        issue.file_id,
                        issue.offset,
                        cstr!(
                            "provide('{}') value is not reactive - consumers won't see updates",
                            key
                        ),
                    )
                    .with_suggestion("provide('key', ref(value)) or provide('key', computed(() => value))")
                }

                CrossFileReactivityIssueKind::ReactivityLostInPropChain {
                    prop_name,
                    parent_component,
                } => CrossFileDiagnostic::new(
                    CrossFileDiagnosticKind::DestructuringBreaksReactivity {
                        source_name: prop_name.clone(),
                        destructured_keys: vec![prop_name.clone()],
                        suggestion: CompactString::new("toRef"),
                    },
                    issue.severity,
                    issue.file_id,
                    issue.offset,
                    cstr!(
                        "Reactive prop '{}' from <{}> loses reactivity in the child",
                        prop_name,
                        parent_component
                    ),
                )
                .with_suggestion(cstr!(
                    "Use toRef(props, '{}'), toRefs(props), or access props.{} directly",
                    prop_name,
                    prop_name
                )),

                CrossFileReactivityIssueKind::CircularReactiveDependency { cycle } => {
                    CrossFileDiagnostic::new(
                        CrossFileDiagnosticKind::CircularReactiveDependency {
                            cycle: cycle.clone(),
                        },
                        issue.severity,
                        issue.file_id,
                        issue.offset,
                        cstr!("Circular reactive dependency: {} \u{2192} ...", cycle.join(" \u{2192} ")),
                    )
                    .with_suggestion("Break the cycle by using computed() or reorganizing data flow")
                }

                _ => CrossFileDiagnostic::new(
                    CrossFileDiagnosticKind::HydrationMismatchRisk {
                        reason: CompactString::new("cross-file reactivity issue"),
                    },
                    issue.severity,
                    issue.file_id,
                    issue.offset,
                    cstr!("{:?}", issue.kind),
                ),
            };

            if let (
                CrossFileReactivityIssueKind::ReactivityLostInPropChain { .. },
                Some(related_file),
            ) = (&issue.kind, issue.related_file)
            {
                diag = diag.with_related(
                    related_file,
                    0,
                    CompactString::new("Reactive value flows from here"),
                );
            }

            diagnostics.push(diag);
        }

        diagnostics
    }
}
