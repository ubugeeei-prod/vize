use super::types::{InternalIssue, ReactivityIssueKind};
use vize_carton::{CompactString, cstr};
use vize_croquis::reactivity::ReactiveKind;

pub(super) fn append_reactivity_losses(
    analysis: &vize_croquis::Croquis,
    issues: &mut Vec<InternalIssue>,
) {
    // Check for reactivity loss patterns detected by the parser
    // These are strict, AST-based detections
    for loss in analysis.reactivity.losses() {
        use vize_croquis::reactivity::ReactivityLossKind;
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
}
