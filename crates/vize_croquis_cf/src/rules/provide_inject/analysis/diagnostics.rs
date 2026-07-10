use super::super::index::{ResolvedProvider, ResolvedProviderBranch};
use crate::diagnostics::{CrossFileDiagnostic, CrossFileDiagnosticKind, DiagnosticSeverity};
use crate::registry::FileId;
use vize_carton::{CompactString, FxHashSet, cstr};
use vize_croquis::provide::InjectEntry;

pub(super) fn provider_relateds(branches: &[ResolvedProviderBranch]) -> Vec<(FileId, u32)> {
    let mut seen = FxHashSet::default();
    branches
        .iter()
        .filter_map(|branch| match branch {
            ResolvedProviderBranch::Matched(provider) => {
                let related = (provider.provider_id, provider.provide.start);
                seen.insert(related).then_some(related)
            }
            ResolvedProviderBranch::Unmatched { .. } => None,
        })
        .collect()
}

pub(super) fn unmatched_inject_diagnostic(
    consumer_id: FileId,
    inject: &InjectEntry,
    key: &CompactString,
    unmatched_count: usize,
    branch_count: usize,
) -> CrossFileDiagnostic {
    let partial = unmatched_count < branch_count;
    let has_default = inject.default_value.is_some();
    let severity = if has_default {
        DiagnosticSeverity::Warning
    } else {
        DiagnosticSeverity::Error
    };
    let message = match (partial, has_default) {
        (true, false) => cstr!(
            "**Conditionally Unmatched Inject**: `inject('{}')` has no matching `provide()` in {} of {} ancestor branches. It returns `undefined` in those render contexts.",
            key,
            unmatched_count,
            branch_count
        ),
        (true, true) => cstr!(
            "**Conditionally Defaulted Inject**: `inject('{}')` falls back to its default value in {} of {} ancestor branches.",
            key,
            unmatched_count,
            branch_count
        ),
        (false, false) => cstr!(
            "**Unmatched Inject**: `inject('{}')` has no matching `provide()` in any ancestor component and returns `undefined` at runtime.",
            key
        ),
        (false, true) => cstr!(
            "**Unmatched Inject Default**: `inject('{}')` falls back to its default value because no ancestor provides this key.",
            key
        ),
    };
    CrossFileDiagnostic::new(
        CrossFileDiagnosticKind::UnmatchedInject { key: key.clone() },
        severity,
        consumer_id,
        inject.start,
        message,
    )
    .with_end_offset(inject.end)
    .with_suggestion(cstr!(
        "Add `provide('{}', value)` on every render branch, or keep a default only if fallback is intentional",
        key
    ))
}

pub(super) fn type_mismatch_diagnostic(
    consumer_id: FileId,
    inject: &InjectEntry,
    key: &CompactString,
    providers: &[&ResolvedProvider],
) -> CrossFileDiagnostic {
    let mut seen_types = FxHashSet::default();
    let provided_type = CompactString::new(
        providers
            .iter()
            .filter_map(|provider| provider.provide.value_type.as_deref())
            .filter(|provided_type| seen_types.insert(CompactString::new(*provided_type)))
            .collect::<Vec<_>>()
            .join(" | "),
    );
    let injected_type = inject
        .expected_type
        .clone()
        .expect("mismatched type requires inject type");
    let message = if providers.len() == 1 {
        cstr!(
            "inject('{}') expects a different type than its nearest provide()",
            key
        )
    } else {
        cstr!(
            "inject('{}') expects a different type than its nearest provide() branches ({})",
            key,
            provided_type
        )
    };
    let mut diagnostic = CrossFileDiagnostic::new(
        CrossFileDiagnosticKind::ProvideInjectTypeMismatch {
            key: key.clone(),
            provided_type,
            injected_type,
        },
        DiagnosticSeverity::Warning,
        consumer_id,
        inject.start,
        message,
    )
    .with_end_offset(inject.end);
    for provider in providers {
        diagnostic = diagnostic.with_related(
            provider.provider_id,
            provider.provide.start,
            cstr!("provide('{key}') source"),
        );
    }
    diagnostic
}

pub(super) fn with_provider_relateds(
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
