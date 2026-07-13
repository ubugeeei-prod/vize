//! Musea diagnostics for Art documents and inline Art blocks.

use tower_lsp::lsp_types::{
    CodeDescription, Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range, Url,
};
use vize_atelier_sfc::SfcDescriptor;
use vize_carton::append;
use vize_patina::{HelpRenderTarget, render_help};

use crate::server::ServerState;

use super::super::{DiagnosticService, LineIndex, sources};

impl DiagnosticService {
    pub(in crate::ide::diagnostics) fn collect_musea(
        state: &ServerState,
        uri: &Url,
        content: &str,
        line_index: &LineIndex<'_>,
    ) -> Vec<Diagnostic> {
        state.ensure_artifact_source(uri, content);
        let croquis = state.sfc_croquis(uri);
        let script = croquis
            .as_deref()
            .map(musea_script_metadata)
            .unwrap_or_default();
        let result = vize_patina::rules::musea::MuseaLinter::new()
            .lint_with_script_metadata(content, script);
        let mut diagnostics = result
            .diagnostics
            .into_iter()
            .map(|diagnostic| musea_diagnostic(diagnostic, line_index, 0, content.len()))
            .collect::<Vec<_>>();
        diagnostics.extend(collect_define_art_source_diagnostics(
            uri,
            content,
            croquis.as_deref(),
        ));
        diagnostics
    }

    pub(in crate::ide::diagnostics) fn collect_inline_art_diagnostics(
        _uri: &Url,
        content: &str,
        descriptor: &SfcDescriptor<'_>,
        line_index: &LineIndex<'_>,
    ) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for custom in &descriptor.custom_blocks {
            if custom.block_type != "art" {
                continue;
            }
            let art_content = format!(
                "<art{}>\n{}\n</art>",
                custom
                    .attrs
                    .iter()
                    .fold(String::new(), |mut attrs, (key, value)| {
                        append!(attrs, " {key}=\"{value}\"");
                        attrs
                    }),
                custom.content
            );
            let prefix = art_content.find('\n').unwrap_or(0) + 1;
            for diagnostic in vize_patina::rules::musea::MuseaLinter::new()
                .lint(&art_content)
                .diagnostics
            {
                if (diagnostic.start as usize) < prefix {
                    let (start_line, start_col) = line_index.line_col(custom.loc.tag_start);
                    let (end_line, end_col) =
                        line_index.line_col(custom.loc.tag_end.min(content.len()));
                    diagnostics.push(musea_with_range(
                        diagnostic,
                        Range {
                            start: Position {
                                line: start_line,
                                character: start_col,
                            },
                            end: Position {
                                line: end_line,
                                character: end_col,
                            },
                        },
                    ));
                } else {
                    diagnostics.push(musea_diagnostic(
                        diagnostic,
                        line_index,
                        custom.loc.start.saturating_sub(prefix),
                        content.len(),
                    ));
                }
            }
        }
        diagnostics
    }
}

fn musea_diagnostic(
    diagnostic: vize_patina::LintDiagnostic,
    line_index: &LineIndex<'_>,
    offset: usize,
    source_len: usize,
) -> Diagnostic {
    let start = offset
        .saturating_add(diagnostic.start as usize)
        .min(source_len);
    let end = offset
        .saturating_add(diagnostic.end as usize)
        .min(source_len);
    let (start_line, start_col) = line_index.line_col(start);
    let (end_line, end_col) = line_index.line_col(end);
    musea_with_range(
        diagnostic,
        Range {
            start: Position {
                line: start_line,
                character: start_col,
            },
            end: Position {
                line: end_line,
                character: end_col,
            },
        },
    )
}

fn musea_with_range(diagnostic: vize_patina::LintDiagnostic, range: Range) -> Diagnostic {
    let message = diagnostic.help.as_ref().map_or_else(
        || diagnostic.message.to_string(),
        |help| {
            format!(
                "{}\n\nHelp: {}",
                diagnostic.message,
                render_help(help, HelpRenderTarget::PlainText)
            )
        },
    );
    Diagnostic {
        range,
        severity: Some(match diagnostic.severity {
            vize_patina::Severity::Error => DiagnosticSeverity::ERROR,
            vize_patina::Severity::Warning => DiagnosticSeverity::WARNING,
        }),
        code: Some(NumberOrString::String(diagnostic.rule_name.to_string())),
        code_description: musea_rule_description(),
        source: Some(sources::MUSEA.to_string()),
        message,
        ..Default::default()
    }
}

fn musea_rule_description() -> Option<CodeDescription> {
    Url::parse("https://github.com/ubugeeei-prod/vize/wiki/musea-rules")
        .or_else(|_| Url::parse("https://github.com/ubugeeei-prod/vize"))
        .ok()
        .map(|href| CodeDescription { href })
}

fn collect_define_art_source_diagnostics(
    uri: &Url,
    content: &str,
    croquis: Option<&vize_croquis::CroquisDocument>,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for source in croquis
        .map(crate::ide::musea::define_art_sources_from_croquis)
        .unwrap_or_default()
    {
        let (code, message) = if source.source.is_empty() {
            (
                "musea/define-art-empty-source",
                "defineArt component source must not be empty".to_string(),
            )
        } else if crate::ide::musea::should_check_define_art_source(&source.source)
            && crate::ide::musea::resolve_define_art_source(uri, &source.source).is_none()
        {
            (
                "musea/define-art-source-not-found",
                format!(
                    "Cannot resolve defineArt component source \"{}\" for <{}>",
                    source.source, source.component_name
                ),
            )
        } else {
            continue;
        };
        diagnostics.push(Diagnostic {
            range: crate::ide::musea::range_for_offsets(
                content,
                source.value_start,
                source.value_end,
            ),
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::String(code.to_string())),
            source: Some(sources::MUSEA.to_string()),
            message,
            ..Default::default()
        });
    }
    diagnostics
}

fn musea_script_metadata(
    document: &vize_croquis::CroquisDocument,
) -> vize_patina::rules::musea::MuseaScriptMetadata {
    let Some(art) = document.analysis().macros.define_art() else {
        return Default::default();
    };
    vize_patina::rules::musea::MuseaScriptMetadata {
        has_title: art.title.is_some() || !art.component_name.is_empty(),
        has_component: art.component_source.is_some(),
    }
}
