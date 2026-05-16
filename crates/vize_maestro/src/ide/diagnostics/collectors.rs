//! Diagnostic collectors for SFC parser, template parser, linter, and Musea.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use tower_lsp::lsp_types::{
    CodeDescription, Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range, Url,
};

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

    /// Collect linter diagnostics from vize_patina.
    pub(super) fn collect_lint_diagnostics(uri: &Url, content: &str) -> Vec<Diagnostic> {
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

        // Create linter and lint the template content
        let linter = vize_patina::Linter::new();
        let result = linter.lint_template(&template.content, uri.path());

        // Convert lint diagnostics to LSP diagnostics
        result
            .diagnostics
            .into_iter()
            .map(|lint_diag| {
                // Convert byte offset to line/column within template
                let (start_line, start_col) =
                    offset_to_line_col(&template.content, lint_diag.start as usize);
                let (end_line, end_col) =
                    offset_to_line_col(&template.content, lint_diag.end as usize);

                // Adjust line numbers based on template block position in SFC
                let sfc_start_line = template.loc.start_line as u32 + start_line;
                let sfc_end_line = template.loc.start_line as u32 + end_line;

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
                            line: sfc_start_line.saturating_sub(1),
                            character: start_col,
                        },
                        end: Position {
                            line: sfc_end_line.saturating_sub(1),
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
