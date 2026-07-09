use super::index::{ProvideInjectIndex, ResolvedProvider, ResolvedProviderBranch};
use super::keys::{provide_key_display, provide_key_identity};
use super::types::{ProvideInjectBranch, ProvideInjectMatch};
use crate::diagnostics::{CrossFileDiagnostic, CrossFileDiagnosticKind, DiagnosticSeverity};
use crate::graph::DependencyGraph;
use crate::registry::{FileId, ModuleRegistry};
use vize_carton::{CompactString, FxHashSet, cstr};
use vize_croquis::provide::InjectPattern;

mod diagnostics;
use self::diagnostics::{
    provider_relateds, type_mismatch_diagnostic, unmatched_inject_diagnostic,
    with_provider_relateds,
};

#[allow(dead_code)]
pub fn analyze_provide_inject(
    registry: &ModuleRegistry,
    graph: &DependencyGraph,
) -> (Vec<ProvideInjectMatch>, Vec<CrossFileDiagnostic>) {
    let index = ProvideInjectIndex::new(registry, graph);
    let (matches, _, diagnostics) = analyze_provide_inject_with_index(&index);
    (matches, diagnostics)
}

pub(crate) fn analyze_provide_inject_with_index(
    index: &ProvideInjectIndex,
) -> (
    Vec<ProvideInjectMatch>,
    Vec<ProvideInjectBranch>,
    Vec<CrossFileDiagnostic>,
) {
    let mut matches = Vec::new();
    let mut branches = Vec::new();
    let mut diagnostics = index.string_key_diagnostics();

    // Track which provides are used
    let mut used_provides: FxHashSet<(FileId, u32)> = FxHashSet::default();
    let mut recorded_matches: FxHashSet<(FileId, u32, FileId, u32)> = FxHashSet::default();

    // For each inject, try to find a matching provide in ancestors
    let mut consumer_ids = index.injects().keys().copied().collect::<Vec<_>>();
    index.sort_file_ids(&mut consumer_ids);
    for consumer_id in consumer_ids {
        let consumer_injects = &index.injects()[&consumer_id];
        for inject in consumer_injects {
            let key_str = provide_key_display(&inject.key);
            let key_identity = provide_key_identity(&inject.key);
            let provider_branches = index.resolve_provider_branches(consumer_id, &inject.key);
            let provider_related = provider_relateds(&provider_branches);

            // Check for destructured inject - this causes reactivity loss
            match &inject.pattern {
                InjectPattern::ObjectDestructure(props) => {
                    let diagnostic =
                        CrossFileDiagnostic::new(
                            CrossFileDiagnosticKind::DestructuringBreaksReactivity {
                                source_name: cstr!("inject('{key_str}')"),
                                destructured_keys: props.clone(),
                                suggestion: CompactString::new("toRefs"),
                            },
                            DiagnosticSeverity::Error,
                            consumer_id,
                            inject.start,
                            cstr!(
                                "Destructuring inject('{}') into {{ {} }} breaks reactivity connection",
                                key_str,
                                props.iter().map(|p| p.as_str()).collect::<Vec<_>>().join(", ")
                            ),
                        )
                        .with_end_offset(inject.end)
                        .with_suggestion(cstr!(
                            "Store inject result first: `const {} = inject('{}')`, then access properties",
                            inject.local_name,
                            key_str
                        ));
                    diagnostics.push(with_provider_relateds(
                        diagnostic,
                        &provider_related,
                        &key_str,
                    ));
                }
                InjectPattern::ArrayDestructure(items) => {
                    let diagnostic =
                        CrossFileDiagnostic::new(
                            CrossFileDiagnosticKind::DestructuringBreaksReactivity {
                                source_name: cstr!("inject('{key_str}')"),
                                destructured_keys: items.clone(),
                                suggestion: CompactString::new("toRefs"),
                            },
                            DiagnosticSeverity::Error,
                            consumer_id,
                            inject.start,
                            cstr!(
                                "Array destructuring inject('{}') into [{}] breaks reactivity connection",
                                key_str,
                                items.iter().map(|p| p.as_str()).collect::<Vec<_>>().join(", ")
                            ),
                        )
                        .with_end_offset(inject.end)
                        .with_suggestion(cstr!(
                            "Store inject result first: `const {} = inject('{}')`, then access indices",
                            inject.local_name,
                            key_str
                        ));
                    diagnostics.push(with_provider_relateds(
                        diagnostic,
                        &provider_related,
                        &key_str,
                    ));
                }
                InjectPattern::IndirectDestructure {
                    inject_var,
                    props,
                    offset,
                } => {
                    // Indirect destructuring also loses reactivity
                    let diagnostic =
                        CrossFileDiagnostic::new(
                            CrossFileDiagnosticKind::DestructuringBreaksReactivity {
                                source_name: inject_var.clone(),
                                destructured_keys: props.clone(),
                                suggestion: CompactString::new("toRefs"),
                            },
                            DiagnosticSeverity::Error,
                            consumer_id,
                            *offset,
                            cstr!(
                                "Destructuring '{}' (from inject('{}')) into {{ {} }} breaks reactivity connection",
                                inject_var,
                                key_str,
                                props.iter().map(|p| p.as_str()).collect::<Vec<_>>().join(", ")
                            ),
                        )
                        .with_suggestion(cstr!(
                            "Access properties directly: `{}.prop` instead of destructuring",
                            inject_var
                        ));
                    diagnostics.push(with_provider_relateds(
                        diagnostic,
                        &provider_related,
                        &key_str,
                    ));
                }
                InjectPattern::Simple => {
                    // No reactivity loss issue
                }
            }

            let unmatched_count = provider_branches
                .iter()
                .filter(|branch| matches!(branch, ResolvedProviderBranch::Unmatched { .. }))
                .count();
            if unmatched_count > 0 {
                let diagnostic = unmatched_inject_diagnostic(
                    consumer_id,
                    inject,
                    &key_str,
                    unmatched_count,
                    provider_branches.len(),
                );
                diagnostics.push(with_provider_relateds(
                    diagnostic,
                    &provider_related,
                    &key_str,
                ));
            }

            let mismatch_providers =
                mismatched_providers(&provider_branches, inject.expected_type.as_ref());
            if !mismatch_providers.is_empty() {
                diagnostics.push(type_mismatch_diagnostic(
                    consumer_id,
                    inject,
                    &key_str,
                    &mismatch_providers,
                ));
            }

            for provider_branch in provider_branches {
                let (path, provider) = match provider_branch {
                    ResolvedProviderBranch::Matched(provider_match) => {
                        used_provides.insert((
                            provider_match.provider_id,
                            provider_match.provide.id.as_u32(),
                        ));
                        let type_match = provide_inject_type_match(
                            provider_match.provide.value_type.as_ref(),
                            inject.expected_type.as_ref(),
                        );
                        let path = provider_match.path;
                        let provider_id = provider_match.provider_id;
                        let provide_offset = provider_match.provide.start;
                        if recorded_matches.insert((
                            provider_id,
                            provider_match.provide.id.as_u32(),
                            consumer_id,
                            inject.start,
                        )) {
                            matches.push(ProvideInjectMatch {
                                provider: provider_id,
                                consumer: consumer_id,
                                key: key_str.clone(),
                                key_identity: key_identity.clone(),
                                path: path.clone(),
                                type_match,
                                provide_offset,
                                inject_offset: inject.start,
                            });
                        }
                        (path, Some((provider_id, provide_offset)))
                    }
                    ResolvedProviderBranch::Unmatched { path } => (path, None),
                };
                branches.push(ProvideInjectBranch {
                    consumer: consumer_id,
                    key_identity: key_identity.clone(),
                    path,
                    provider: provider.map(|(provider, _)| provider),
                    provide_offset: provider.map(|(_, offset)| offset),
                    inject_offset: inject.start,
                });
            }
        }
    }

    // Check for unused provides
    for (&provider_id, provider_provides) in index.provides() {
        for provide in provider_provides {
            let key_str = provide_key_display(&provide.key);

            if !used_provides.contains(&(provider_id, provide.id.as_u32())) {
                diagnostics.push(
                    CrossFileDiagnostic::new(
                        CrossFileDiagnosticKind::UnusedProvide {
                            key: key_str.clone(),
                        },
                        DiagnosticSeverity::Warning,
                        provider_id,
                        provide.start,
                        cstr!(
                            "provide('{}') is not used by any descendant component",
                            key_str
                        ),
                    )
                    .with_end_offset(provide.end)
                    .with_suggestion("Remove if not needed, or add inject() in a child component"),
                );
            }
        }
    }

    (matches, branches, diagnostics)
}

fn mismatched_providers<'a>(
    branches: &'a [ResolvedProviderBranch],
    expected_type: Option<&CompactString>,
) -> Vec<&'a ResolvedProvider> {
    let mut seen = FxHashSet::default();
    branches
        .iter()
        .filter_map(|branch| match branch {
            ResolvedProviderBranch::Matched(provider)
                if provide_inject_type_match(
                    provider.provide.value_type.as_ref(),
                    expected_type,
                ) == Some(false)
                    && seen.insert((provider.provider_id, provider.provide.id.as_u32())) =>
            {
                Some(provider)
            }
            _ => None,
        })
        .collect()
}

fn provide_inject_type_match(
    provided_type: Option<&CompactString>,
    injected_type: Option<&CompactString>,
) -> Option<bool> {
    Some(types_equal_ignoring_ascii_whitespace(
        provided_type?.as_str(),
        injected_type?.as_str(),
    ))
}

fn types_equal_ignoring_ascii_whitespace(left: &str, right: &str) -> bool {
    left.chars()
        .filter(|ch| !ch.is_ascii_whitespace())
        .eq(right.chars().filter(|ch| !ch.is_ascii_whitespace()))
}
