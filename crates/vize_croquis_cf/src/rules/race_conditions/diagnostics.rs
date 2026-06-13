use super::InjectedMutation;
use crate::diagnostics::{CrossFileDiagnostic, CrossFileDiagnosticKind, DiagnosticSeverity};
use crate::registry::FileId;
use vize_carton::{CompactString, FxHashMap, cstr};
use vize_croquis::race::{RaceConditionRisk, RaceConditionRiskKind};

pub(super) fn local_diagnostics(
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

pub(super) fn injected_mutation_diagnostic(
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

pub(super) fn injected_writer_counts(
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
