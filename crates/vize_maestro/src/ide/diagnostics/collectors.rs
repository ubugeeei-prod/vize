//! Diagnostic collectors for SFC parser, template parser, linter, and Musea.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use tower_lsp::lsp_types::{
    CodeDescription, Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range, Url,
};

use oxc_allocator::Allocator as OxcAllocator;
use oxc_parser::Parser as OxcParser;
use oxc_span::SourceType;
use vize_patina::{render_help, HelpRenderTarget};

use super::{offset_to_line_col, sources, DiagnosticService};
use vize_carton::append;

impl DiagnosticService {
    /// Collect diagnostics for Art files (*.art.vue) using vize_patina's MuseaLinter.
    pub(super) fn collect_musea_diagnostics(_uri: &Url, content: &str) -> Vec<Diagnostic> {
        use vize_patina::rules::musea::MuseaLinter;

        let linter = MuseaLinter::new();
        let result = linter.lint(content);

        result
            .diagnostics
            .into_iter()
            .map(|lint_diag| {
                // Convert byte offset to line/column
                let (start_line, start_col) = offset_to_line_col(content, lint_diag.start as usize);
                let (end_line, end_col) = offset_to_line_col(content, lint_diag.end as usize);

                // Build the diagnostic message with help text (render as plain text for LSP)
                #[allow(clippy::disallowed_macros)]
                let message = if let Some(ref help) = lint_diag.help {
                    format!(
                        "{}\n\nHelp: {}",
                        lint_diag.message,
                        render_help(help, HelpRenderTarget::PlainText)
                    )
                } else {
                    lint_diag.message.to_string()
                };

                Diagnostic {
                    range: Range {
                        start: Position {
                            line: start_line,
                            character: start_col,
                        },
                        end: Position {
                            line: end_line,
                            character: end_col,
                        },
                    },
                    severity: Some(match lint_diag.severity {
                        vize_patina::Severity::Error => DiagnosticSeverity::ERROR,
                        vize_patina::Severity::Warning => DiagnosticSeverity::WARNING,
                    }),
                    code: Some(NumberOrString::String(lint_diag.rule_name.to_string())),
                    code_description: Some(CodeDescription {
                        href: Url::parse("https://github.com/ubugeeei/vize/wiki/musea-rules")
                            .unwrap_or_else(|_| {
                                Url::parse("https://github.com/ubugeeei/vize").unwrap()
                            }),
                    }),
                    source: Some(sources::MUSEA.to_string()),
                    message,
                    ..Default::default()
                }
            })
            .collect()
    }

    /// Collect diagnostics for inline <art> custom blocks in regular .vue files.
    pub(super) fn collect_inline_art_diagnostics(uri: &Url, content: &str) -> Vec<Diagnostic> {
        use vize_patina::rules::musea::MuseaLinter;

        let options = vize_atelier_sfc::SfcParseOptions {
            filename: uri.path().to_string().into(),
            ..Default::default()
        };

        let Ok(descriptor) = vize_atelier_sfc::parse_sfc(content, options) else {
            return vec![];
        };

        let mut diagnostics = Vec::new();

        for custom in &descriptor.custom_blocks {
            if custom.block_type != "art" {
                continue;
            }

            // Reconstruct the art block content including tags for the linter
            // The linter expects a full art file, so we wrap the content
            #[allow(clippy::disallowed_macros)]
            let art_content = format!(
                "<art{}>\n{}\n</art>",
                // Reconstruct attributes
                custom.attrs.iter().fold(String::new(), |mut acc, (k, v)| {
                    append!(acc, " {k}=\"{v}\"");
                    acc
                }),
                custom.content
            );

            let linter = MuseaLinter::new();
            let result = linter.lint(&art_content);

            // Map diagnostics back to the original file positions
            let block_content_start = custom.loc.start;

            for lint_diag in result.diagnostics {
                // The lint_diag offsets are relative to art_content
                // We need to adjust: skip the reconstructed <art ...>\n prefix
                let art_tag_prefix_len = art_content.find('\n').unwrap_or(0) + 1;

                // Only process diagnostics that fall within the content area
                if (lint_diag.start as usize) < art_tag_prefix_len {
                    // Diagnostic is on the <art> tag itself - map to the original tag
                    let (start_line, start_col) = offset_to_line_col(content, custom.loc.tag_start);
                    let (end_line, end_col) =
                        offset_to_line_col(content, custom.loc.tag_end.min(content.len()));

                    #[allow(clippy::disallowed_macros)]
                    let message = if let Some(ref help) = lint_diag.help {
                        format!(
                            "{}\n\nHelp: {}",
                            lint_diag.message,
                            render_help(help, HelpRenderTarget::PlainText)
                        )
                    } else {
                        lint_diag.message.to_string()
                    };

                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position {
                                line: start_line,
                                character: start_col,
                            },
                            end: Position {
                                line: end_line,
                                character: end_col,
                            },
                        },
                        severity: Some(match lint_diag.severity {
                            vize_patina::Severity::Error => DiagnosticSeverity::ERROR,
                            vize_patina::Severity::Warning => DiagnosticSeverity::WARNING,
                        }),
                        code: Some(NumberOrString::String(lint_diag.rule_name.to_string())),
                        source: Some(sources::MUSEA.to_string()),
                        message,
                        ..Default::default()
                    });
                } else {
                    // Diagnostic is in the content area - map offset to original file
                    let content_relative_start =
                        (lint_diag.start as usize).saturating_sub(art_tag_prefix_len);
                    let content_relative_end =
                        (lint_diag.end as usize).saturating_sub(art_tag_prefix_len);

                    let sfc_start = block_content_start + content_relative_start;
                    let sfc_end = block_content_start + content_relative_end;

                    let (start_line, start_col) =
                        offset_to_line_col(content, sfc_start.min(content.len()));
                    let (end_line, end_col) =
                        offset_to_line_col(content, sfc_end.min(content.len()));

                    #[allow(clippy::disallowed_macros)]
                    let message = if let Some(ref help) = lint_diag.help {
                        format!(
                            "{}\n\nHelp: {}",
                            lint_diag.message,
                            render_help(help, HelpRenderTarget::PlainText)
                        )
                    } else {
                        lint_diag.message.to_string()
                    };

                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position {
                                line: start_line,
                                character: start_col,
                            },
                            end: Position {
                                line: end_line,
                                character: end_col,
                            },
                        },
                        severity: Some(match lint_diag.severity {
                            vize_patina::Severity::Error => DiagnosticSeverity::ERROR,
                            vize_patina::Severity::Warning => DiagnosticSeverity::WARNING,
                        }),
                        code: Some(NumberOrString::String(lint_diag.rule_name.to_string())),
                        source: Some(sources::MUSEA.to_string()),
                        message,
                        ..Default::default()
                    });
                }
            }
        }

        diagnostics
    }

    /// Collect SFC parser diagnostics.
    pub(super) fn collect_sfc_diagnostics(uri: &Url, content: &str) -> Vec<Diagnostic> {
        let options = vize_atelier_sfc::SfcParseOptions {
            filename: uri.path().to_string().into(),
            ..Default::default()
        };

        match vize_atelier_sfc::parse_sfc(content, options) {
            Ok(_) => vec![],
            Err(err) => {
                let range = if let Some(ref loc) = err.loc {
                    Range {
                        start: Position {
                            line: loc.start_line.saturating_sub(1) as u32,
                            character: loc.start_column.saturating_sub(1) as u32,
                        },
                        end: Position {
                            line: loc.end_line.saturating_sub(1) as u32,
                            character: loc.end_column.saturating_sub(1) as u32,
                        },
                    }
                } else {
                    Range::default()
                };

                vec![Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    source: Some(sources::SFC_PARSER.to_string()),
                    #[allow(clippy::disallowed_methods)]
                    message: err.message.to_string(),
                    ..Default::default()
                }]
            }
        }
    }

    /// Collect template parser diagnostics.
    pub(super) fn collect_template_diagnostics(uri: &Url, content: &str) -> Vec<Diagnostic> {
        let options = vize_atelier_sfc::SfcParseOptions {
            filename: uri.path().to_string().into(),
            ..Default::default()
        };

        let Ok(descriptor) = vize_atelier_sfc::parse_sfc(content, options) else {
            return vec![];
        };

        let Some(ref template) = descriptor.template else {
            return vec![];
        };

        let allocator = vize_carton::Bump::new();
        let (_, errors) = vize_armature::parse(&allocator, &template.content);

        errors
            .iter()
            .filter_map(|error| {
                let loc = error.loc.as_ref()?;

                // Adjust line numbers based on template block position
                let start_line =
                    (template.loc.start_line as u32) + loc.start.line.saturating_sub(1);
                let end_line = (template.loc.start_line as u32) + loc.end.line.saturating_sub(1);

                Some(Diagnostic {
                    range: Range {
                        start: Position {
                            line: start_line.saturating_sub(1),
                            character: loc.start.column.saturating_sub(1),
                        },
                        end: Position {
                            line: end_line.saturating_sub(1),
                            character: loc.end.column.saturating_sub(1),
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::Number(error.code as i32)),
                    source: Some(sources::TEMPLATE_PARSER.to_string()),
                    #[allow(clippy::disallowed_methods)]
                    message: error.message.to_string(),
                    ..Default::default()
                })
            })
            .collect()
    }

    /// Collect script parser diagnostics.
    pub(super) fn collect_script_diagnostics(uri: &Url, content: &str) -> Vec<Diagnostic> {
        let options = vize_atelier_sfc::SfcParseOptions {
            filename: uri.path().to_string().into(),
            ..Default::default()
        };

        let Ok(descriptor) = vize_atelier_sfc::parse_sfc(content, options) else {
            return vec![];
        };

        let mut diagnostics = Vec::new();

        if let Some(ref script) = descriptor.script {
            diagnostics.extend(collect_script_block_diagnostics(
                content,
                &script.content,
                script.loc.start,
                script.lang.as_deref(),
            ));
        }

        if let Some(ref script_setup) = descriptor.script_setup {
            diagnostics.extend(collect_script_block_diagnostics(
                content,
                &script_setup.content,
                script_setup.loc.start,
                script_setup.lang.as_deref(),
            ));
        }

        diagnostics
    }

    /// Collect linter diagnostics from vize_patina.
    pub(super) fn collect_lint_diagnostics(uri: &Url, content: &str) -> Vec<Diagnostic> {
        let options = vize_atelier_sfc::SfcParseOptions {
            filename: uri.path().to_string().into(),
            ..Default::default()
        };

        if vize_atelier_sfc::parse_sfc(content, options).is_err() {
            return vec![];
        }

        // Create linter and lint the full SFC so editor diagnostics match the CLI.
        let linter = vize_patina::Linter::new();
        let result = linter.lint_sfc(content, uri.path());

        // Convert lint diagnostics to LSP diagnostics
        result
            .diagnostics
            .into_iter()
            .map(|lint_diag| {
                // Convert byte offsets directly in the SFC. vize_patina::lint_sfc
                // already maps template diagnostics back to source coordinates.
                let (start_line, start_col) = offset_to_line_col(content, lint_diag.start as usize);
                let (end_line, end_col) = offset_to_line_col(content, lint_diag.end as usize);

                // Build the diagnostic message with help text (render as plain text for LSP)
                #[allow(clippy::disallowed_macros)]
                let message = if let Some(ref help) = lint_diag.help {
                    format!(
                        "{}\n\nHelp: {}",
                        lint_diag.message,
                        render_help(help, HelpRenderTarget::PlainText)
                    )
                } else {
                    lint_diag.message.to_string()
                };

                #[allow(clippy::disallowed_macros)]
                Diagnostic {
                    range: Range {
                        start: Position {
                            line: start_line,
                            character: start_col,
                        },
                        end: Position {
                            line: end_line,
                            character: end_col,
                        },
                    },
                    severity: Some(match lint_diag.severity {
                        vize_patina::Severity::Error => DiagnosticSeverity::ERROR,
                        vize_patina::Severity::Warning => DiagnosticSeverity::WARNING,
                    }),
                    code: Some(NumberOrString::String(lint_diag.rule_name.to_string())),
                    code_description: Some(CodeDescription {
                        href: Url::parse(&format!(
                            "https://eslint.vuejs.org/rules/{}.html",
                            lint_diag
                                .rule_name
                                .strip_prefix("vue/")
                                .unwrap_or(lint_diag.rule_name)
                        ))
                        .unwrap_or_else(|_| Url::parse("https://eslint.vuejs.org/rules/").unwrap()),
                    }),
                    source: Some(sources::LINTER.to_string()),
                    message,
                    ..Default::default()
                }
            })
            .collect()
    }
}

fn collect_script_block_diagnostics(
    sfc_content: &str,
    script_content: &str,
    script_offset: usize,
    lang: Option<&str>,
) -> Vec<Diagnostic> {
    let Some(source_type) = script_source_type(lang) else {
        return vec![];
    };

    let allocator = OxcAllocator::default();
    let parsed = OxcParser::new(&allocator, script_content, source_type).parse();

    parsed
        .errors
        .iter()
        .map(|error| {
            let (local_start, local_end) = diagnostic_span(error, script_content.len());
            let start = script_offset + local_start;
            let end = script_offset + local_end;
            let (start_line, start_col) = offset_to_line_col(sfc_content, start);
            let (end_line, end_col) = offset_to_line_col(sfc_content, end);

            Diagnostic {
                range: Range {
                    start: Position {
                        line: start_line,
                        character: start_col,
                    },
                    end: Position {
                        line: end_line,
                        character: end_col,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("script-parse-error".to_string())),
                source: Some(sources::SCRIPT_PARSER.to_string()),
                message: format!("Script parse error: {}", error),
                ..Default::default()
            }
        })
        .collect()
}

fn script_source_type(lang: Option<&str>) -> Option<SourceType> {
    let extension = match lang.map(|value| value.trim().to_ascii_lowercase()) {
        None => "js".to_string(),
        Some(value) if value.is_empty() => "js".to_string(),
        Some(value) if matches!(value.as_str(), "js" | "javascript") => "js".to_string(),
        Some(value) if matches!(value.as_str(), "jsx") => "jsx".to_string(),
        Some(value) if matches!(value.as_str(), "ts" | "typescript") => "ts".to_string(),
        Some(value) if matches!(value.as_str(), "tsx") => "tsx".to_string(),
        Some(_) => return None,
    };

    SourceType::from_path(format!("script.{extension}")).ok()
}

fn diagnostic_span(error: &oxc_diagnostics::OxcDiagnostic, source_len: usize) -> (usize, usize) {
    let fallback_end = source_len.max(1);
    let Some(label) = error.labels.as_ref().and_then(|labels| {
        labels
            .iter()
            .find(|label| label.primary())
            .or_else(|| labels.first())
    }) else {
        return (0, fallback_end);
    };

    let start = label.offset().min(source_len);
    let end = start.saturating_add(label.len().max(1)).min(fallback_end);
    (start, end.max(start + 1))
}
