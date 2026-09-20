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

impl DiagnosticService {
    /// Collect diagnostics asynchronously (includes Corsa diagnostics when available).
    pub async fn collect_async(state: &ServerState, uri: &Url) -> Vec<Diagnostic> {
        tracing::info!("collect_async: {}", uri);

        // Start with sync diagnostics (patina, etc.)
        let mut diagnostics = Self::collect(state, uri);
        tracing::info!("sync diagnostics count: {}", diagnostics.len());
        let native_script_syntax = state.is_lsp_typecheck_enabled()
            && diagnostics
                .iter()
                .any(|diagnostic| diagnostic.source.as_deref() == Some(sources::SCRIPT_PARSER))
            && state.documents.text(uri).is_some_and(|source| {
                Self::parse_sfc_for_collect(uri, &source)
                    .is_ok_and(|descriptor| vize_canon::supports_native_script_syntax(&descriptor))
            });
        if has_blocking_parser_error(&diagnostics, native_script_syntax) {
            tracing::info!("collect_async: Corsa diagnostics skipped after parser error");
            return diagnostics;
        }
        if is_standalone_html_path(uri.path()) {
            tracing::info!("collect_async: Corsa diagnostics skipped for standalone HTML");
            return diagnostics;
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
                match crate::runtime::timeout(std::time::Duration::from_secs(10), jsx_future).await
                {
                    Ok(jsx_type_diags) => {
                        tracing::info!("jsx type diagnostics count: {}", jsx_type_diags.len());
                        diagnostics.extend(jsx_type_diags);
                    }
                    Err(_) => tracing::warn!("jsx type diagnostics timed out for {}", uri),
                }
            } else {
                tracing::info!("collect_async: jsx type diagnostics skipped (disabled by config)");
            }
            return diagnostics;
        }

        if state.is_lsp_typecheck_enabled() {
            // Try to get Corsa diagnostics (with timeout, skip on failure).
            // Use 10s timeout - polling for diagnostics internally uses 5s
            let corsa_future = Self::collect_corsa_diagnostics(state, uri);
            match crate::runtime::timeout(std::time::Duration::from_secs(10), corsa_future).await {
                Ok(CorsaDiagnostics::Complete(corsa_diags)) => {
                    if native_script_syntax {
                        diagnostics.retain(|diagnostic| {
                            diagnostic.source.as_deref() != Some(sources::SCRIPT_PARSER)
                        });
                    }
                    tracing::info!("corsa diagnostics count: {}", corsa_diags.len());
                    diagnostics.extend(corsa_diags);
                }
                Ok(CorsaDiagnostics::Unavailable(hints)) => diagnostics.extend(hints),
                Err(_) => {
                    tracing::warn!("corsa diagnostics timed out for {}", uri);
                }
            }

            // When the user opted into typecheck but Corsa never came up
            // (init failed, timed out, or simply not yet attempted while we
            // already produced zero corsa diagnostics for an SFC), surface a
            // hint diagnostic so the Problems panel reflects what is
            // happening. Without this, the editor goes silent and users
            // assume their project is clean. See #681.
            if !state.has_corsa_bridge()
                && !diagnostics
                    .iter()
                    .any(|d| d.source.as_deref() == Some(super::sources::TYPE_CHECKER))
            {
                diagnostics.push(typecheck_unavailable_hint());
            }
        } else {
            tracing::info!("collect_async: Corsa diagnostics skipped (disabled by config)");
        }

        diagnostics
    }
}

fn has_blocking_parser_error(diagnostics: &[Diagnostic], native_script_syntax: bool) -> bool {
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
