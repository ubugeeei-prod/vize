//! Diagnostics generation and reporting for cross-file reactivity analysis.

use super::analyzer::CrossFileReactivityAnalyzer;
use super::types::{CrossFileReactivityIssueKind, ReactivityFlowKind};
use crate::cross_file::diagnostics::{
    CrossFileDiagnostic, CrossFileDiagnosticKind, DiagnosticSeverity,
};
use vize_carton::append;
use vize_carton::cstr;
use vize_carton::CompactString;
use vize_carton::String;

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

    /// Generate a markdown report of cross-file reactivity flows.
    pub fn to_markdown(&self) -> String {
        let mut md = String::default();

        md.push_str("# Cross-File Reactivity Report\n\n");

        // Summary
        md.push_str("## Summary\n\n");
        append!(
            md,
            "- **Tracked Reactive Values**: {}\n",
            self.reactive_values.len()
        );
        append!(md, "- **Cross-File Flows**: {}\n", self.flows.len());
        append!(md, "- **Issues Detected**: {}\n\n", self.issues.len());

        // Flows
        if !self.flows.is_empty() {
            md.push_str("## Reactivity Flows\n\n");
            md.push_str("```\n");

            for flow in &self.flows {
                let status = if flow.preserved {
                    "\u{2713}"
                } else {
                    "\u{2717}"
                };
                let flow_type = match flow.flow_kind {
                    ReactivityFlowKind::ComposableExport => "composable",
                    ReactivityFlowKind::ProvideInject => "provide/inject",
                    ReactivityFlowKind::PropsFlow => "props",
                    ReactivityFlowKind::StoreFlow => "store",
                    ReactivityFlowKind::ModuleImport => "import",
                };

                append!(
                    md,
                    "{status} [{flow_type}] {} \u{2192} {}\n",
                    flow.source.name,
                    flow.target.name
                );

                if let Some(ref reason) = flow.loss_reason {
                    append!(md, "   \u{2514}\u{2500} Loss: {:?}\n", reason);
                }
            }

            md.push_str("```\n\n");
        }

        // Issues
        if !self.issues.is_empty() {
            md.push_str("## Issues\n\n");

            for issue in &self.issues {
                let icon = match issue.severity {
                    DiagnosticSeverity::Error => "\u{274c}",
                    DiagnosticSeverity::Warning => "\u{26a0}\u{fe0f}",
                    DiagnosticSeverity::Info => "\u{2139}\u{fe0f}",
                    DiagnosticSeverity::Hint => "\u{1f4a1}",
                };

                append!(md, "### {icon} {:?}\n\n", issue.kind);
                append!(md, "- **File**: {:?}\n", issue.file_id);
                append!(md, "- **Offset**: {}\n", issue.offset);
                if let Some(related) = issue.related_file {
                    append!(md, "- **Related File**: {:?}\n", related);
                }
                md.push('\n');
            }
        }

        md
    }
}
