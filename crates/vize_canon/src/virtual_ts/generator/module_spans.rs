//! Module-scope span planning from shared frontend products.

use vize_croquis::{Croquis, ScopeKind};

use super::{script_module, spans::merge_overlapping_spans};

pub(super) fn collect_module_spans(
    summary: &Croquis,
    script: Option<&str>,
    modules: Option<&vize_module::ModuleDocument>,
    script_facts: Option<&vize_atelier_sfc::SfcScriptGeneratorFacts>,
    has_script_setup: bool,
) -> Vec<(u32, u32)> {
    let mut spans = summary
        .import_statements
        .iter()
        .map(|import| (import.start, import.end))
        .collect::<Vec<_>>();

    if let Some(script) = script {
        let cached = modules.and_then(|modules| {
            script_module::collect_cached_module_statement_spans(script, modules)
        });
        let projected = script_facts.and_then(|facts| {
            script_module::collect_projected_module_statement_spans(script, facts)
        });
        spans.extend(cached.or(projected).unwrap_or_default());
    }

    spans.extend(
        summary
            .re_exports
            .iter()
            .map(|re_export| (re_export.start, re_export.end)),
    );
    if has_script_setup {
        spans.extend(summary.scopes.iter().filter_map(|scope| {
            matches!(scope.kind, ScopeKind::NonScriptSetup)
                .then_some((scope.span.start, scope.span.end))
        }));
    }
    spans.extend(
        summary
            .type_exports
            .iter()
            .filter(|export| export.hoisted)
            .map(|export| (export.start, export.end)),
    );

    merge_overlapping_spans(spans)
}
