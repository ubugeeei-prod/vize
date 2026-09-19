//! JSX diagnostics use the same native document and mapping as navigation.

use super::{position::virtual_range_to_source, service::JsxService};
use crate::ide::{IdeContext, diagnostics::sources};
use std::sync::Arc;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};
use vize_canon::CorsaBridge;

impl JsxService {
    /// Type diagnostics surfaced from JSX virtual TS through Corsa. Diagnostics
    /// outside authored mappings (such as the ambient preamble) are dropped.
    pub async fn diagnostics(
        ctx: &IdeContext<'_>,
        corsa_bridge: Option<Arc<CorsaBridge>>,
    ) -> Vec<Diagnostic> {
        let Some(bridge) = corsa_bridge else {
            return vec![];
        };
        if !bridge.is_initialized() {
            return vec![];
        }
        let Some(mut virtual_ts) = Self::virtual_ts(ctx) else {
            return vec![];
        };

        let Some(uri) =
            super::service_project::open_virtual_project(ctx, &bridge, &mut virtual_ts).await
        else {
            return vec![];
        };

        let Ok(corsa_diags) = bridge.get_diagnostics(&uri).await else {
            return vec![];
        };

        corsa_diags
            .into_iter()
            .filter_map(|diag| {
                // Skip "declared but never used" noise on the synthesized sink
                // helper and any other internal `__vize_` symbol.
                let is_unused = diag.message.contains("is declared but")
                    && (diag.message.contains("never read") || diag.message.contains("never used"));
                if is_unused && diag.message.contains("'__vize") {
                    return None;
                }

                let (start_line, end_line, start_char, end_char) = virtual_range_to_source(
                    &virtual_ts,
                    &ctx.content,
                    diag.range.start.line,
                    diag.range.start.character,
                    diag.range.end.line,
                    diag.range.end.character,
                )?;

                Some(Diagnostic {
                    range: Range {
                        start: Position {
                            line: start_line,
                            character: start_char,
                        },
                        end: Position {
                            line: end_line,
                            character: end_char,
                        },
                    },
                    severity: diag.severity.map(|s| match s {
                        1 => DiagnosticSeverity::ERROR,
                        2 => DiagnosticSeverity::WARNING,
                        3 => DiagnosticSeverity::INFORMATION,
                        _ => DiagnosticSeverity::HINT,
                    }),
                    code: diag
                        .code
                        .map(crate::ide::diagnostics::corsa::corsa_diagnostic_code),
                    source: Some(sources::TYPE_CHECKER.to_string()),
                    message: crate::ide::diagnostics::corsa::rewrite_corsa_message(
                        &diag.message,
                        &ctx.content,
                    ),
                    ..Default::default()
                })
            })
            .collect()
    }
}
