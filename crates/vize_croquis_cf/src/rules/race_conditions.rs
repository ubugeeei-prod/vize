//! Async race-condition analysis for reactive state.
//!
//! Uses parser-derived race facts and adds cross-file context for injected
//! state mutations.

use super::provide_inject::ProvideInjectIndex;
use crate::diagnostics::CrossFileDiagnostic;
use crate::graph::DependencyGraph;
use crate::registry::{FileId, ModuleEntry, ModuleRegistry};
use vize_carton::{CompactString, FxHashMap, cstr};
use vize_croquis::provide::{InjectEntry, ProvideKey};
use vize_croquis::race::{RaceConditionRisk, RaceConditionRiskKind};

mod diagnostics;

use diagnostics::{injected_mutation_diagnostic, injected_writer_counts, local_diagnostics};

/// Kind of race-condition issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RaceConditionIssueKind {
    /// Local reactive value is mutated from an async boundary.
    AsyncReactiveMutation {
        variable_name: CompactString,
        async_context: CompactString,
    },
    /// `watchEffect` contains async work and reactive mutation.
    AsyncWatchEffect {
        async_operation: CompactString,
        mutated_targets: Vec<CompactString>,
    },
    /// Injected state is mutated from an async boundary.
    InjectedAsyncMutation {
        key: CompactString,
        target_name: CompactString,
        async_context: CompactString,
        provider: FileId,
        writer_count: usize,
    },
}

/// A detected race-condition issue with file context.
#[derive(Debug, Clone)]
pub struct RaceConditionIssue {
    /// File where the issue occurs.
    pub file_id: FileId,
    /// Kind of race issue.
    pub kind: RaceConditionIssueKind,
    /// Start offset in script.
    pub offset: u32,
    /// End offset in script.
    pub end: u32,
}

#[derive(Debug, Clone)]
pub(super) struct InjectedMutation {
    pub(super) consumer: FileId,
    pub(super) provider: FileId,
    pub(super) key: CompactString,
    pub(super) key_identity: CompactString,
    pub(super) target_name: CompactString,
    pub(super) async_context: CompactString,
    pub(super) offset: u32,
    pub(super) end: u32,
    pub(super) provide_offset: u32,
}

struct PendingInjectedMutation<'a> {
    consumer: FileId,
    inject: InjectEntry,
    target_name: CompactString,
    risk: &'a RaceConditionRisk,
}

/// Analyze async race-condition risks across registered files.
#[allow(dead_code)]
pub fn analyze_race_conditions(
    registry: &ModuleRegistry,
    graph: &DependencyGraph,
) -> (Vec<RaceConditionIssue>, Vec<CrossFileDiagnostic>) {
    analyze_race_conditions_with_index(registry, graph, None)
}

pub(crate) fn analyze_race_conditions_with_index(
    registry: &ModuleRegistry,
    graph: &DependencyGraph,
    provide_inject_index: Option<&ProvideInjectIndex>,
) -> (Vec<RaceConditionIssue>, Vec<CrossFileDiagnostic>) {
    let mut issues = Vec::new();
    let mut diagnostics = Vec::new();
    let mut injected_mutations = Vec::new();
    let mut pending_injected_mutations = Vec::new();

    for entry in registry.vue_components() {
        let injected_targets = injected_targets(entry);
        for risk in entry.analysis.race_conditions.risks() {
            for target in risk.kind.mutated_targets() {
                let Some(&inject) = injected_targets.get(target.as_str()) else {
                    diagnostics.extend(local_diagnostics(entry.id, risk, target));
                    issues.push(RaceConditionIssue {
                        file_id: entry.id,
                        kind: local_issue_kind(risk, target),
                        offset: risk.start,
                        end: risk.end,
                    });
                    continue;
                };

                pending_injected_mutations.push(PendingInjectedMutation {
                    consumer: entry.id,
                    inject: inject.clone(),
                    target_name: target.clone(),
                    risk,
                });
            }
        }
    }

    if !pending_injected_mutations.is_empty() {
        let owned_index;
        let index = if let Some(index) = provide_inject_index {
            index
        } else {
            owned_index = ProvideInjectIndex::new(registry, graph);
            &owned_index
        };

        let mut provider_cache = FxHashMap::default();
        for pending in pending_injected_mutations {
            let key_identity = provide_key_identity(&pending.inject.key);
            let cache_key = (pending.consumer, pending.inject.key.clone());
            let matches = provider_cache
                .entry(cache_key)
                .or_insert_with(|| index.resolve_providers(pending.consumer, &pending.inject.key))
                .clone();

            if matches.is_empty() {
                diagnostics.extend(local_diagnostics(
                    pending.consumer,
                    pending.risk,
                    &pending.target_name,
                ));
                issues.push(RaceConditionIssue {
                    file_id: pending.consumer,
                    kind: local_issue_kind(pending.risk, &pending.target_name),
                    offset: pending.risk.start,
                    end: pending.risk.end,
                });
                continue;
            }

            for provider_match in matches {
                injected_mutations.push(InjectedMutation {
                    consumer: pending.consumer,
                    provider: provider_match.provider_id,
                    key: provide_key_display(&pending.inject.key),
                    key_identity: key_identity.clone(),
                    target_name: pending.target_name.clone(),
                    async_context: pending.risk.kind.async_context(),
                    offset: pending.risk.start,
                    end: pending.risk.end,
                    provide_offset: provider_match.provide.start,
                });
            }
        }
    }

    let writer_counts = injected_writer_counts(&injected_mutations);
    for mutation in &injected_mutations {
        let writer_count = writer_counts
            .get(&(mutation.provider, mutation.key_identity.clone()))
            .copied()
            .unwrap_or(1);
        let issue_kind = RaceConditionIssueKind::InjectedAsyncMutation {
            key: mutation.key.clone(),
            target_name: mutation.target_name.clone(),
            async_context: mutation.async_context.clone(),
            provider: mutation.provider,
            writer_count,
        };
        issues.push(RaceConditionIssue {
            file_id: mutation.consumer,
            kind: issue_kind.clone(),
            offset: mutation.offset,
            end: mutation.end,
        });
        diagnostics.push(injected_mutation_diagnostic(
            mutation,
            writer_count,
            &injected_mutations,
        ));
    }

    (issues, diagnostics)
}

fn injected_targets(entry: &ModuleEntry) -> FxHashMap<&str, &InjectEntry> {
    entry
        .analysis
        .provide_inject
        .injects()
        .iter()
        .filter(|inject| !inject.local_name.starts_with('('))
        .map(|inject| (inject.local_name.as_str(), inject))
        .collect()
}

fn local_issue_kind(risk: &RaceConditionRisk, target: &CompactString) -> RaceConditionIssueKind {
    match &risk.kind {
        RaceConditionRiskKind::AsyncWatchEffect {
            async_operation,
            mutated_targets,
        } => RaceConditionIssueKind::AsyncWatchEffect {
            async_operation: async_operation.clone(),
            mutated_targets: mutated_targets.clone(),
        },
        _ => RaceConditionIssueKind::AsyncReactiveMutation {
            variable_name: target.clone(),
            async_context: risk.kind.async_context(),
        },
    }
}

fn provide_key_display(key: &ProvideKey) -> CompactString {
    match key {
        ProvideKey::String(s) | ProvideKey::Symbol(s) => s.clone(),
    }
}

fn provide_key_identity(key: &vize_croquis::provide::ProvideKey) -> CompactString {
    match key {
        vize_croquis::provide::ProvideKey::String(s) => cstr!("string:{s}"),
        vize_croquis::provide::ProvideKey::Symbol(s) => cstr!("symbol:{s}"),
    }
}
