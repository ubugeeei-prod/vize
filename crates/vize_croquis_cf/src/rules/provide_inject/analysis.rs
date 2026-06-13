use super::index::ProvideInjectIndex;
use super::keys::{provide_key_display, provide_key_identity};
use super::types::ProvideInjectMatch;
use crate::diagnostics::{CrossFileDiagnostic, CrossFileDiagnosticKind, DiagnosticSeverity};
use crate::graph::DependencyGraph;
use crate::registry::{FileId, ModuleRegistry};
use vize_carton::{CompactString, FxHashSet, cstr};
use vize_croquis::provide::InjectPattern;

#[allow(dead_code)]
pub fn analyze_provide_inject(
    registry: &ModuleRegistry,
    graph: &DependencyGraph,
) -> (Vec<ProvideInjectMatch>, Vec<CrossFileDiagnostic>) {
    let index = ProvideInjectIndex::new(registry, graph);
    analyze_provide_inject_with_index(&index)
}

pub(crate) fn analyze_provide_inject_with_index(
    index: &ProvideInjectIndex,
) -> (Vec<ProvideInjectMatch>, Vec<CrossFileDiagnostic>) {
    let mut matches = Vec::new();
    let mut diagnostics = index.string_key_diagnostics();

    // Track which provides are used
    let mut used_provides: FxHashSet<(FileId, u32)> = FxHashSet::default();

    // For each inject, try to find a matching provide in ancestors
    for (&consumer_id, consumer_injects) in index.injects() {
        for inject in consumer_injects {
            let key_str = provide_key_display(&inject.key);
            let provider_matches = index.resolve_providers(consumer_id, &inject.key);
            let provider_related: Vec<_> = provider_matches
                .iter()
                .map(|provider| (provider.provider_id, provider.provide.start))
                .collect();

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

            if provider_matches.is_empty() {
                // No provider found
                if inject.default_value.is_none() {
                    diagnostics.push(
                        CrossFileDiagnostic::new(
                            CrossFileDiagnosticKind::UnmatchedInject {
                                key: key_str.clone(),
                            },
                            DiagnosticSeverity::Error,
                            consumer_id,
                            inject.start,
                            cstr!(
                                "**Unmatched Inject**: `inject('{}')` has no matching `provide()` in any ancestor component\n\n\
                                This will return `undefined` at runtime and may cause errors.\n\n\
                                ### Checklist:\n\
                                - [ ] Add `provide('{}', value)` in a parent/ancestor component\n\
                                - [ ] Or provide a default value: `inject('{}', defaultValue)`",
                                key_str, key_str, key_str
                            ),
                        )
                        .with_end_offset(inject.end)
                        .with_suggestion(cstr!(
                            "```typescript\n// In parent component:\nprovide('{}', yourValue)\n\n// Or with default:\nconst {} = inject('{}', defaultValue)\n```",
                            key_str, inject.local_name, key_str
                        )),
                    );
                } else {
                    diagnostics.push(
                        CrossFileDiagnostic::new(
                            CrossFileDiagnosticKind::UnmatchedInject {
                                key: key_str.clone(),
                            },
                            DiagnosticSeverity::Warning,
                            consumer_id,
                            inject.start,
                            cstr!(
                                "**Unmatched Inject Default**: `inject('{}')` falls back to its default value because no ancestor provides this key.\n\n\
                                The runtime fallback is safe, but this can hide broken provider wiring.",
                                key_str
                            ),
                        )
                        .with_end_offset(inject.end)
                        .with_suggestion(cstr!(
                            "Add `provide('{}', value)` in an ancestor, or keep the fallback only if it is intentional",
                            key_str
                        )),
                    );
                }
            } else {
                for provider_match in provider_matches {
                    // Found a match
                    used_provides.insert((
                        provider_match.provider_id,
                        provider_match.provide.id.as_u32(),
                    ));

                    matches.push(ProvideInjectMatch {
                        provider: provider_match.provider_id,
                        consumer: consumer_id,
                        key: key_str.clone(),
                        key_identity: provide_key_identity(&inject.key),
                        path: provider_match.path,
                        type_match: None, // Would need type analysis
                        provide_offset: provider_match.provide.start,
                        inject_offset: inject.start,
                    });
                }
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

    (matches, diagnostics)
}

fn with_provider_relateds(
    mut diagnostic: CrossFileDiagnostic,
    provider_related: &[(FileId, u32)],
    key: &CompactString,
) -> CrossFileDiagnostic {
    for (provider_id, provider_offset) in provider_related {
        diagnostic = diagnostic.with_related(
            *provider_id,
            *provider_offset,
            cstr!("provide('{key}') source"),
        );
    }
    diagnostic
}
