//! JSX diagnostics use the same native document and mapping as navigation.

use super::{position::virtual_range_to_source, service::JsxService};
use crate::ide::{IdeContext, diagnostics::sources};
use std::{ops::Range as ByteRange, sync::Arc};
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
                if is_rewritten_project_import_extension(
                    &virtual_ts.code,
                    &ctx.content,
                    &diag,
                    (start_line, end_line, start_char, end_char),
                ) {
                    return None;
                }

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
    let Some(range) = diagnostic_byte_range(generated, diagnostic) else {
        return false;
    };
    vize_canon::projection::assemble::is_generated_project_import_extension_at(
        generated,
        diagnostic_code(diagnostic),
        &diagnostic.message,
        range.start,
        range.end,
    )
}

fn is_rewritten_project_import_extension(
    generated: &str,
    authored: &str,
    diagnostic: &vize_canon::corsa_bridge::LspDiagnostic,
    authored_range: super::position::LspRangeParts,
) -> bool {
    let Some(generated_range) = diagnostic_byte_range(generated, diagnostic) else {
        return false;
    };
    let (start_line, end_line, start_character, end_character) = authored_range;
    let Some(authored_start) =
        LineBreaks::Lsp.position_to_offset(authored, start_line, start_character)
    else {
        return false;
    };
    let authored_end = LineBreaks::Lsp
        .position_to_offset(authored, end_line, end_character)
        .unwrap_or(authored_start);
    if diagnostic_code(diagnostic) != Some(5097)
        || !diagnostic.message.contains("allowImportingTsExtensions")
    {
        return false;
    }
    let Some(generated) = quoted_specifier_at(generated, generated_range) else {
        return false;
    };
    let Some(authored) = quoted_specifier_at(authored, authored_start..authored_end) else {
        return false;
    };
    has_typescript_extension(generated) && !has_typescript_extension(authored)
}

fn quoted_specifier_at(source: &str, range: ByteRange<usize>) -> Option<&str> {
    let start = range.start.min(source.len());
    let end = range.end.min(source.len()).max(start);
    let line_start = source
        .get(..start)?
        .rfind(['\n', '\r'])
        .map_or(0, |at| at + 1);
    let line_end = source
        .get(start..)?
        .find(['\n', '\r'])
        .map_or(source.len(), |at| start + at);
    let line = source.get(line_start..line_end)?;
    let local_start = start - line_start;
    let local_end = end - line_start;
    let mut quote: Option<(char, usize)> = None;
    let mut escaped = false;

    for (offset, character) in line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            continue;
        }
        match quote {
            Some((delimiter, open)) if character == delimiter => {
                let close = offset + character.len_utf8();
                if open <= local_end && local_start <= close {
                    return line.get(open + delimiter.len_utf8()..offset);
                }
                quote = None;
            }
            None if matches!(character, '\'' | '"' | '`') => quote = Some((character, offset)),
            _ => {}
        }
    }
    None
}

fn has_typescript_extension(specifier: &str) -> bool {
    let path = specifier
        .split_once(['?', '#'])
        .map_or(specifier, |(path, _)| path);
    [".ts", ".tsx", ".mts", ".cts"]
        .into_iter()
        .any(|extension| path.ends_with(extension))
}

fn diagnostic_code(diagnostic: &vize_canon::corsa_bridge::LspDiagnostic) -> Option<u32> {
    diagnostic.code.as_ref().and_then(|code| match code {
        serde_json::Value::Number(number) => {
            number.as_u64().and_then(|value| value.try_into().ok())
        }
        serde_json::Value::String(text) => text.trim_start_matches("TS").parse().ok(),
        _ => None,
    })
}

fn diagnostic_byte_range(
    generated: &str,
    diagnostic: &vize_canon::corsa_bridge::LspDiagnostic,
) -> Option<ByteRange<usize>> {
    let start = LineBreaks::Lsp.position_to_offset(
        generated,
        diagnostic.range.start.line,
        diagnostic.range.start.character,
    )?;
    let end = LineBreaks::Lsp
        .position_to_offset(
            generated,
            diagnostic.range.end.line,
            diagnostic.range.end.character,
        )
        .unwrap_or(start);
    Some(start..end)
}

#[cfg(test)]
mod tests {
    use super::{is_generated_project_import_extension, is_rewritten_project_import_extension};
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

    #[test]
    fn jsx_diagnostics_drop_typescript_extensions_added_to_authored_imports() {
        let generated = "import { account } from '/virtual/workspace/model.ts';\n";
        let authored = "import { account } from \"./model\";\n";
        let generated_diagnostic = diagnostic(generated, "'/virtual/workspace/model.ts'");
        let start = authored.find("\"./model\"").unwrap() as u32;
        let end = start + "\"./model\"".len() as u32;
        assert!(is_rewritten_project_import_extension(
            generated,
            authored,
            &generated_diagnostic,
            (0, 0, start, end),
        ));

        let explicit = "import { account } from \"./model.ts\";\n";
        let explicit_end = start + "\"./model.ts\"".len() as u32;
        assert!(!is_rewritten_project_import_extension(
            generated,
            explicit,
            &generated_diagnostic,
            (0, 0, start, explicit_end),
        ));
    }
}
