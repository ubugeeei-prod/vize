//! JSX diagnostics use the same native document and mapping as navigation.

use super::{position::virtual_range_to_source, service::JsxService};
use crate::ide::{IdeContext, diagnostics::sources};
use std::sync::Arc;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};
use vize_canon::CorsaBridge;
use vize_l0::line_index::LineBreaks;

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
        let Some((virtual_ts, uri)) =
            super::service_project::open_virtual_project(ctx, &bridge).await
        else {
            return vec![];
        };

        let Ok(corsa_diags) = bridge.get_diagnostics(&uri).await else {
            return vec![];
        };

        corsa_diags
            .into_iter()
            .filter_map(|diag| {
                if is_generated_project_import_extension(&virtual_ts.code, &diag) {
                    return None;
                }
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

fn is_generated_project_import_extension(
    generated: &str,
    diagnostic: &vize_canon::corsa_bridge::LspDiagnostic,
) -> bool {
    let code = diagnostic.code.as_ref().and_then(|code| match code {
        serde_json::Value::Number(number) => {
            number.as_u64().and_then(|value| value.try_into().ok())
        }
        serde_json::Value::String(text) => text.trim_start_matches("TS").parse().ok(),
        _ => None,
    });
    let Some(start) = LineBreaks::Lsp.position_to_offset(
        generated,
        diagnostic.range.start.line,
        diagnostic.range.start.character,
    ) else {
        return false;
    };
    let end = LineBreaks::Lsp
        .position_to_offset(
            generated,
            diagnostic.range.end.line,
            diagnostic.range.end.character,
        )
        .unwrap_or(start);
    vize_canon::projection::assemble::is_generated_project_import_extension_at(
        generated,
        code,
        &diagnostic.message,
        start,
        end,
    )
}

#[cfg(test)]
mod tests {
    use super::is_generated_project_import_extension;
    use vize_canon::corsa_bridge::{LspDiagnostic, LspPosition, LspRange};

    const MESSAGE: &str = "An import path can only end with a '.ts' extension when 'allowImportingTsExtensions' is enabled.";

    fn diagnostic(source: &str, specifier: &str) -> LspDiagnostic {
        let start = source.find(specifier).unwrap() as u32;
        LspDiagnostic {
            range: LspRange {
                start: LspPosition {
                    line: 0,
                    character: start,
                },
                end: LspPosition {
                    line: 0,
                    character: start + specifier.len() as u32,
                },
            },
            severity: Some(1),
            code: Some(serde_json::json!(5097)),
            source: Some("ts".into()),
            message: MESSAGE.into(),
            related_information: None,
        }
    }

    #[test]
    fn jsx_diagnostics_drop_only_generated_vue_typescript_routes() {
        let generated = "import Counter from './Counter.vue.ts';\n";
        assert!(is_generated_project_import_extension(
            generated,
            &diagnostic(generated, "'./Counter.vue.ts'")
        ));

        let authored = "import value from './value.ts';\n";
        assert!(!is_generated_project_import_extension(
            authored,
            &diagnostic(authored, "'./value.ts'")
        ));
    }
}
