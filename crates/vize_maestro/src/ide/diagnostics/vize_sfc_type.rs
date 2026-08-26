#[cfg(feature = "native")]
use tower_lsp::lsp_types::{CodeDescription, DiagnosticSeverity, NumberOrString, Position, Range};
use tower_lsp::lsp_types::{Diagnostic, Url};
#[cfg(feature = "native")]
use vize_s0::cstr;

#[cfg(feature = "native")]
use super::sources;
use super::{DiagnosticService, LineIndex};
use crate::server::ServerState;

impl DiagnosticService {
    pub(super) fn extend_vize_sfc_type_diagnostics(
        state: &ServerState,
        uri: &Url,
        content: &str,
        line_index: &LineIndex<'_>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        #[cfg(feature = "native")]
        {
            if !state.is_lsp_typecheck_enabled() {
                return;
            }

            let config = state.get_type_checker_config();
            if !config.check_fallthrough_attrs {
                return;
            }

            let vize_type_diags =
                Self::collect_vize_sfc_type_diagnostics(uri, content, line_index, config.strict);
            tracing::info!(
                "collect: vize-owned SFC type diagnostics: {}",
                vize_type_diags.len()
            );
            diagnostics.extend(vize_type_diags);
        }

        #[cfg(not(feature = "native"))]
        let _ = (state, uri, content, line_index, diagnostics);
    }

    /// Collect Vize-owned SFC type diagnostics that are not produced by
    /// TypeScript/Corsa itself.
    ///
    /// Native editor builds use Corsa for TS diagnostics, so the synchronous
    /// legacy type checker must stay out of the hot path. The fallthrough-attrs
    /// warning is different: it is a Vize semantic diagnostic with authored SFC
    /// byte offsets, and Corsa has no equivalent. Keep this collector narrow so
    /// sync diagnostics do not reintroduce old type false positives.
    #[cfg(feature = "native")]
    fn collect_vize_sfc_type_diagnostics(
        uri: &Url,
        content: &str,
        line_index: &LineIndex<'_>,
        strict: bool,
    ) -> Vec<Diagnostic> {
        let options = vize_canon::SfcTypeCheckOptions {
            filename: uri.path().to_string().into(),
            include_virtual_ts: false,
            check_props: false,
            check_emits: false,
            check_template_bindings: false,
            check_reactivity: false,
            check_setup_context: false,
            check_invalid_exports: false,
            check_fallthrough_attrs: true,
            strict,
        };

        vize_canon::type_check_sfc(content, &options)
            .diagnostics
            .into_iter()
            .filter(|diagnostic| diagnostic.code.as_deref() == Some("fallthrough-attrs"))
            .map(|diagnostic| {
                let (start_line, start_character) = line_index.line_col(diagnostic.start as usize);
                let (end_line, end_character) = line_index.line_col(diagnostic.end as usize);
                let message = if let Some(help) = diagnostic.help.as_deref() {
                    cstr!("{}\n\nHelp: {}", diagnostic.message, help).to_string()
                } else {
                    diagnostic.message.to_string()
                };

                Diagnostic {
                    range: Range {
                        start: Position {
                            line: start_line,
                            character: start_character,
                        },
                        end: Position {
                            line: end_line,
                            character: end_character,
                        },
                    },
                    severity: Some(match diagnostic.severity {
                        vize_canon::SfcTypeSeverity::Error => DiagnosticSeverity::ERROR,
                        vize_canon::SfcTypeSeverity::Warning => DiagnosticSeverity::WARNING,
                        vize_canon::SfcTypeSeverity::Info => DiagnosticSeverity::INFORMATION,
                        vize_canon::SfcTypeSeverity::Hint => DiagnosticSeverity::HINT,
                    }),
                    code: diagnostic
                        .code
                        .map(|code| NumberOrString::String(code.to_string())),
                    code_description: Some(CodeDescription {
                        href: Url::parse(
                            "https://github.com/ubugeeei-prod/vize/wiki/type-errors#fallthrough-attrs",
                        )
                        .unwrap_or_else(|_| {
                            Url::parse("https://github.com/ubugeeei-prod/vize").unwrap()
                        }),
                    }),
                    source: Some(sources::TYPE_CHECKER.to_string()),
                    message,
                    ..Default::default()
                }
            })
            .collect()
    }
}
