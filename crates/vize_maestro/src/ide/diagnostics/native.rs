//! Native diagnostic ownership and parser fallback after backend failures.

use super::{
    DiagnosticService, corsa::collect::CorsaDiagnostics, service::typecheck_unavailable_hint,
    sources,
};
use crate::{
    server::ServerState,
    utils::{is_jsx_path, is_standalone_html_path},
};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Url};

mod provenance;
pub(super) mod timeout;
pub(crate) use provenance::NativeDiagnostic;

/// The native part of one result, kept to combine with a later sync set.
pub(crate) struct NativeFamily {
    pub(crate) diagnostics: Vec<NativeDiagnostic>,
    /// Corsa answered, so its syntax errors replace script parser ones.
    pub(crate) owns_script_syntax: bool,
}

/// A publication and where its native family starts.
pub(crate) struct CombinedDiagnostics {
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) native: Option<NativeFamily>,
}

impl CombinedDiagnostics {
    pub(crate) fn into_parts(self) -> (Option<NativeFamily>, Vec<Diagnostic>) {
        (self.native, self.diagnostics)
    }
}

/// The one rule for every publication: a sync set plus the latest native
/// family, which a blocking parser error suppresses.
pub(super) fn combine(
    mut sync: Vec<Diagnostic>,
    native_script_syntax: bool,
    native: Option<NativeFamily>,
) -> CombinedDiagnostics {
    let Some(native) = native.filter(|_| !has_blocking_parser_error(&sync, native_script_syntax))
    else {
        return CombinedDiagnostics {
            diagnostics: sync,
            native: None,
        };
    };
    if native.owns_script_syntax && native_script_syntax {
        sync.retain(|diagnostic| diagnostic.source.as_deref() != Some(sources::SCRIPT_PARSER));
    }
    let native_diagnostics = if native.owns_script_syntax {
        native.diagnostics
    } else {
        without_carried_parser_duplicates(native.diagnostics, &sync)
    };
    let family = NativeFamily {
        diagnostics: without_duplicate_required_props(native_diagnostics, &sync),
        owns_script_syntax: native.owns_script_syntax,
    };
    sync.extend(
        family
            .diagnostics
            .iter()
            .map(|record| record.diagnostic.clone()),
    );
    CombinedDiagnostics {
        diagnostics: sync,
        native: Some(family),
    }
}

/// A carried family describes an earlier revision. When an unphased diagnostic
/// lands on the exact range of the current script parser error, the parser owns
/// that finding until the native pass for the current revision completes.
/// Other unknown diagnostics and every producer-owned phase still move with
/// unrelated edits.
fn without_carried_parser_duplicates(
    native: Vec<NativeDiagnostic>,
    authored: &[Diagnostic],
) -> Vec<NativeDiagnostic> {
    let parser_ranges = authored
        .iter()
        .filter(|diagnostic| diagnostic.source.as_deref() == Some(sources::SCRIPT_PARSER))
        .map(|diagnostic| diagnostic.range)
        .collect::<Vec<_>>();
    native
        .into_iter()
        .filter(|diagnostic| {
            diagnostic.phase != vize_canon::corsa_bridge::DiagnosticPhase::Unknown
                || !parser_ranges.contains(&diagnostic.range)
        })
        .collect()
}

impl DiagnosticService {
    /// Collect diagnostics asynchronously (includes Corsa diagnostics when available).
    pub async fn collect_async(state: &ServerState, uri: &Url) -> Vec<Diagnostic> {
        Self::collect_async_from_sync(state, uri, Self::collect(state, uri))
            .await
            .diagnostics
    }

    /// Combine a current sync set with an earlier native family.
    pub(crate) fn combine_native(
        state: &ServerState,
        uri: &Url,
        sync: Vec<Diagnostic>,
        native: Option<NativeFamily>,
    ) -> Vec<Diagnostic> {
        let Some(native) = native else {
            return sync;
        };
        let native_script_syntax = native_script_syntax(state, uri, &sync);
        combine(sync, native_script_syntax, Some(native)).diagnostics
    }

    /// Merge native diagnostics into an already collected, current sync set.
    /// The server validates its source stamp before handing this set over.
    pub(crate) async fn collect_async_from_sync(
        state: &ServerState,
        uri: &Url,
        diagnostics: Vec<Diagnostic>,
    ) -> CombinedDiagnostics {
        tracing::info!("collect_async: {}", uri);
        tracing::info!("sync diagnostics count: {}", diagnostics.len());
        let native_script_syntax = native_script_syntax(state, uri, &diagnostics);
        let skipped = |diagnostics| CombinedDiagnostics {
            diagnostics,
            native: None,
        };
        if has_blocking_parser_error(&diagnostics, native_script_syntax) {
            tracing::info!("collect_async: Corsa diagnostics skipped after parser error");
            return skipped(diagnostics);
        }
        if is_standalone_html_path(uri.path()) {
            tracing::info!("collect_async: Corsa diagnostics skipped for standalone HTML");
            return skipped(diagnostics);
        }

        // JSX/TSX type diagnostics. The sync `collect` above already added the
        // JSX compiler diagnostics; here we add TypeScript type errors derived
        // from the JSX virtual TS, surfaced alongside them. Gated on the opt-in
        // `typeChecker.jsxTypecheck` so React `.tsx` is never Vue-JSX-checked.
        if is_jsx_path(uri.path()) {
            if state.jsx_typecheck_enabled()
                && let Some(ctx) = crate::ide::IdeContext::new(state, uri, 0)
            {
                let corsa_bridge = state.get_corsa_bridge().await;
                let jsx_future = crate::ide::JsxService::diagnostics(&ctx, corsa_bridge);
                let jsx_type_diags = jsx_future.await;
                tracing::info!("jsx type diagnostics count: {}", jsx_type_diags.len());
                let native = NativeFamily {
                    diagnostics: jsx_type_diags.into_iter().map(Into::into).collect(),
                    owns_script_syntax: false,
                };
                return combine(diagnostics, native_script_syntax, Some(native));
            }
            tracing::info!("collect_async: jsx type diagnostics skipped (disabled by config)");
            return skipped(diagnostics);
        }

        if !state.is_lsp_typecheck_enabled() {
            tracing::info!("collect_async: Corsa diagnostics skipped (disabled by config)");
            return skipped(diagnostics);
        }
        let corsa_future = Self::collect_corsa_diagnostics(state, uri);
        let native = match corsa_future.await {
            CorsaDiagnostics::Complete(mut corsa_diags) => {
                #[cfg(test)]
                super::assembly_parity_custody::record(state, uri, "native_complete", || {
                    serde_json::json!(corsa_diags)
                });
                tracing::info!("corsa diagnostics count: {}", corsa_diags.len());
                corsa_diags.extend(
                    super::corsa::collect::configuration_disclosures(state)
                        .into_iter()
                        .map(NativeDiagnostic::from),
                );
                NativeFamily {
                    diagnostics: corsa_diags,
                    owns_script_syntax: true,
                }
            }
            CorsaDiagnostics::Unavailable(hints) => {
                #[cfg(test)]
                super::assembly_parity_custody::record(state, uri, "native_unavailable", || {
                    serde_json::json!(hints)
                });
                NativeFamily {
                    diagnostics: hints.into_iter().map(Into::into).collect(),
                    owns_script_syntax: false,
                }
            }
        };
        let mut combined = combine(diagnostics, native_script_syntax, Some(native));
        with_unavailable_hint(state, &mut combined);
        combined
    }
}

/// When the user opted into typecheck but Corsa never came up (init failed,
/// timed out, or simply not yet attempted while we already produced zero
/// corsa diagnostics for an SFC), surface a hint diagnostic so the Problems
/// panel reflects what is happening. Without this, the editor goes silent and
/// users assume their project is clean. See #681.
pub(super) fn with_unavailable_hint(state: &ServerState, publication: &mut CombinedDiagnostics) {
    if !state.has_corsa_bridge()
        && !publication
            .diagnostics
            .iter()
            .any(|d| d.source.as_deref() == Some(super::sources::TYPE_CHECKER))
    {
        let hint = typecheck_unavailable_hint();
        if let Some(native) = &mut publication.native {
            native.diagnostics.push(hint.clone().into());
        }
        publication.diagnostics.push(hint);
    }
}

pub(super) fn native_script_syntax(
    state: &ServerState,
    uri: &Url,
    diagnostics: &[Diagnostic],
) -> bool {
    state.is_lsp_typecheck_enabled()
        && diagnostics
            .iter()
            .any(|diagnostic| diagnostic.source.as_deref() == Some(sources::SCRIPT_PARSER))
        && state.documents.text_snapshot(uri).is_some_and(|source| {
            DiagnosticService::descriptor_for_collect(state, uri, &source)
                .is_ok_and(|descriptor| vize_canon::supports_native_script_syntax(&descriptor))
        })
}

/// The authored component check already reports the missing prop on the tag.
/// A native TS2345 for the generated checker call repeats it and exposes
/// internal helper types, especially after an unsaved dependency edit.
fn without_duplicate_required_props(
    corsa: Vec<NativeDiagnostic>,
    authored: &[Diagnostic],
) -> Vec<NativeDiagnostic> {
    let required_tags = authored
        .iter()
        .filter(|diagnostic| {
            diagnostic.source.as_deref() == Some(sources::COMPONENTS)
                && diagnostic.code.as_ref()
                    == Some(&tower_lsp::lsp_types::NumberOrString::String(
                        "component-required-props".into(),
                    ))
        })
        .map(|diagnostic| diagnostic.range)
        .collect::<Vec<_>>();
    corsa
        .into_iter()
        .filter(|diagnostic| {
            let duplicate = diagnostic.source.as_deref() == Some(sources::TYPE_CHECKER)
                && matches!(
                    diagnostic.code.as_ref(),
                    Some(tower_lsp::lsp_types::NumberOrString::Number(2345))
                )
                && diagnostic.message.contains("$ComponentCheckProps")
                && diagnostic.message.contains("is missing")
                && required_tags.iter().any(|tag| {
                    diagnostic.range.start.line == tag.start.line
                        && diagnostic.range.start.character >= tag.start.character.saturating_sub(1)
                        && diagnostic.range.start.character <= tag.end.character
                });
            !duplicate
        })
        .collect()
}

#[cfg(test)]
mod tests;

pub(super) fn has_blocking_parser_error(
    diagnostics: &[Diagnostic],
    native_script_syntax: bool,
) -> bool {
    diagnostics.iter().any(|diagnostic| {
        if native_script_syntax && diagnostic.source.as_deref() == Some(sources::SCRIPT_PARSER) {
            return false;
        }
        matches!(
            diagnostic.source.as_deref(),
            // Parser-level errors plus SFC compile-time validation errors
            // both leave the script body in a state where Corsa would just
            // cascade — the user already has the actionable diagnostic.
            Some(
                super::sources::SFC_PARSER
                    | super::sources::SCRIPT_PARSER
                    | super::sources::TEMPLATE_PARSER
                    | super::sources::SFC_COMPILER
            )
        ) && diagnostic.severity == Some(DiagnosticSeverity::ERROR)
    })
}
