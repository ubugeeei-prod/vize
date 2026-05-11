//! Async race-condition analysis for reactive state.
//!
//! Uses parser-derived race facts and adds cross-file context for injected
//! state mutations.

use crate::cross_file::diagnostics::{
    CrossFileDiagnostic, CrossFileDiagnosticKind, DiagnosticSeverity,
};
use crate::cross_file::graph::DependencyGraph;
use crate::cross_file::registry::{FileId, ModuleEntry, ModuleRegistry};
use crate::provide::InjectEntry;
use crate::race::{RaceConditionRisk, RaceConditionRiskKind};
use vize_carton::{cstr, CompactString, FxHashMap};

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
struct InjectedMutation {
    consumer: FileId,
    provider: FileId,
    key: CompactString,
    key_identity: CompactString,
    target_name: CompactString,
    async_context: CompactString,
    offset: u32,
    end: u32,
    provide_offset: u32,
}

/// Analyze async race-condition risks across registered files.
pub fn analyze_race_conditions(
    registry: &ModuleRegistry,
    graph: &DependencyGraph,
) -> (Vec<RaceConditionIssue>, Vec<CrossFileDiagnostic>) {
    let provide_matches = super::provide_inject::analyze_provide_inject(registry, graph).0;
    let mut matches_by_consumer_key = FxHashMap::default();
    for provide_match in provide_matches {
        matches_by_consumer_key
            .entry((provide_match.consumer, provide_match.key_identity.clone()))
            .or_insert_with(Vec::new)
            .push(provide_match);
    }

    let mut issues = Vec::new();
    let mut diagnostics = Vec::new();
    let mut injected_mutations = Vec::new();

    for entry in registry.vue_components() {
        let injected_targets = injected_targets(entry);
        for risk in entry.analysis.race_conditions.risks() {
            for target in risk.kind.mutated_targets() {
                let Some(inject) = injected_targets.get(target.as_str()) else {
                    diagnostics.extend(local_diagnostics(entry.id, risk, target));
                    issues.push(RaceConditionIssue {
                        file_id: entry.id,
                        kind: local_issue_kind(risk, target),
                        offset: risk.start,
                        end: risk.end,
                    });
                    continue;
                };

                let key_identity = provide_key_identity(&inject.key);
                let Some(matches) = matches_by_consumer_key.get(&(entry.id, key_identity.clone()))
                else {
                    diagnostics.extend(local_diagnostics(entry.id, risk, target));
                    issues.push(RaceConditionIssue {
                        file_id: entry.id,
                        kind: local_issue_kind(risk, target),
                        offset: risk.start,
                        end: risk.end,
                    });
                    continue;
                };

                for provide_match in matches {
                    injected_mutations.push(InjectedMutation {
                        consumer: entry.id,
                        provider: provide_match.provider,
                        key: provide_match.key.clone(),
                        key_identity: key_identity.clone(),
                        target_name: target.clone(),
                        async_context: risk.kind.async_context(),
                        offset: risk.start,
                        end: risk.end,
                        provide_offset: provide_match.provide_offset,
                    });
                }
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

fn local_diagnostics(
    file_id: FileId,
    risk: &RaceConditionRisk,
    target: &CompactString,
) -> Vec<CrossFileDiagnostic> {
    if matches!(risk.kind, RaceConditionRiskKind::AsyncWatchEffect { .. }) {
        return vec![watch_effect_diagnostic(file_id, risk)];
    }

    vec![
        CrossFileDiagnostic::new(
            CrossFileDiagnosticKind::AsyncBoundaryCrossing {
                variable_name: target.clone(),
                async_context: risk.kind.async_context(),
            },
            DiagnosticSeverity::Error,
            file_id,
            risk.start,
            cstr!(
                "Reactive state '{}' is mutated from an async boundary; stale completions can overwrite newer state",
                target
            ),
        )
        .with_end_offset(risk.end)
        .with_suggestion(
            "Add cancellation/cleanup, guard stale requests, or keep async results in an owned request token",
        ),
    ]
}

fn watch_effect_diagnostic(file_id: FileId, risk: &RaceConditionRisk) -> CrossFileDiagnostic {
    let (async_operation, targets) = match &risk.kind {
        RaceConditionRiskKind::AsyncWatchEffect {
            async_operation,
            mutated_targets,
        } => (async_operation.clone(), mutated_targets.clone()),
        _ => (
            risk.kind.async_context(),
            risk.kind.mutated_targets().to_vec(),
        ),
    };
    let target_list = targets
        .iter()
        .map(|target| target.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    CrossFileDiagnostic::new(
        CrossFileDiagnosticKind::WatchEffectWithAsync { async_operation },
        DiagnosticSeverity::Error,
        file_id,
        risk.start,
        cstr!(
            "watchEffect async work mutates reactive state ({}) and can race with invalidation",
            target_list
        ),
    )
    .with_end_offset(risk.end)
    .with_suggestion(
        "Use watch() with onCleanup/onWatcherCleanup and cancel stale async work before mutating state",
    )
}

fn injected_mutation_diagnostic(
    mutation: &InjectedMutation,
    writer_count: usize,
    all_mutations: &[InjectedMutation],
) -> CrossFileDiagnostic {
    let mut diagnostic = CrossFileDiagnostic::new(
        CrossFileDiagnosticKind::InjectedAsyncMutationRace {
            key: mutation.key.clone(),
            target_name: mutation.target_name.clone(),
            async_context: mutation.async_context.clone(),
            writer_count,
        },
        DiagnosticSeverity::Error,
        mutation.consumer,
        mutation.offset,
        cstr!(
            "Injected state '{}' is mutated from {} in a consumer; provider-owned state can be overwritten by stale async completions",
            mutation.key,
            mutation.async_context
        ),
    )
    .with_end_offset(mutation.end)
    .with_related(
        mutation.provider,
        mutation.provide_offset,
        cstr!("provider for injected key '{}'", mutation.key),
    )
    .with_suggestion(
        "Move async writes behind a provider-owned action, or cancel/ignore stale consumer work with onCleanup/onWatcherCleanup",
    );

    for other in all_mutations {
        if other.consumer == mutation.consumer
            || other.provider != mutation.provider
            || other.key_identity != mutation.key_identity
        {
            continue;
        }
        diagnostic = diagnostic.with_related(
            other.consumer,
            other.offset,
            cstr!("another async writer for injected key '{}'", other.key),
        );
    }

    diagnostic
}

fn injected_writer_counts(
    mutations: &[InjectedMutation],
) -> FxHashMap<(FileId, CompactString), usize> {
    let mut writer_files: FxHashMap<(FileId, CompactString), Vec<FileId>> = FxHashMap::default();
    for mutation in mutations {
        let writers = writer_files
            .entry((mutation.provider, mutation.key_identity.clone()))
            .or_default();
        if !writers.contains(&mutation.consumer) {
            writers.push(mutation.consumer);
        }
    }

    writer_files
        .into_iter()
        .map(|(key, writers)| (key, writers.len()))
        .collect()
}

fn provide_key_identity(key: &crate::provide::ProvideKey) -> CompactString {
    match key {
        crate::provide::ProvideKey::String(s) => cstr!("string:{s}"),
        crate::provide::ProvideKey::Symbol(s) => cstr!("symbol:{s}"),
    }
}
