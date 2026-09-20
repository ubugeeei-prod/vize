//! The diagnostic service: aggregation orchestration and shared types.

use crate::ide::ecosystem;
use crate::server::ServerState;
use crate::utils::{is_jsx_path, is_standalone_html_path};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Range, Url};

use super::{LineIndex, Severity};

#[cfg(feature = "native")]
pub(in crate::ide) struct VirtualTsResult {
    pub(in crate::ide) code: String,
    pub(in crate::ide) source_mappings: Vec<vize_canon::virtual_ts::VizeMapping>,
    pub(in crate::ide) semantic_links: Vec<vize_canon::virtual_ts::VizeSemanticLink>,
    /// Byte-offset mapping from post-rewrite to pre-rewrite virtual TS.
    /// Empty when no `.vue` import specifiers were rewritten.
    pub(in crate::ide) import_source_map: vize_canon::ImportSourceMap,
}

/// Diagnostic service for collecting and aggregating diagnostics.
pub struct DiagnosticService;
impl DiagnosticService {
    /// Collect all diagnostics for a document.
    pub fn collect(state: &ServerState, uri: &Url) -> Vec<Diagnostic> {
        let Some(content) = state.documents.text(uri) else {
            tracing::warn!("collect: document not found for {}", uri);
            return vec![];
        };

        let mut diagnostics = Vec::new();
        let features = state.lsp_features();

        if !features.has_diagnostics() {
            return diagnostics;
        }

        let line_index = LineIndex::new(&content);

        let path = uri.path();
        if path.ends_with(".art.vue") {
            if features.lint {
                diagnostics.extend(Self::collect_musea_diagnostics(
                    state,
                    uri,
                    &content,
                    &line_index,
                ));
            }
            return diagnostics;
        }

        if is_standalone_html_path(path) {
            if features.lint {
                let lint_diags = Self::collect_lint_diagnostics(
                    state,
                    uri,
                    &content,
                    features.ecosystem,
                    &line_index,
                );
                tracing::info!(
                    "collect: standalone HTML patina lint diagnostics: {}",
                    lint_diags.len()
                );
                diagnostics.extend(lint_diags);
            }
            return diagnostics;
        }

        if is_jsx_path(path) {
            let jsx_diags = Self::collect_jsx_diagnostics(uri, &content, &line_index);
            tracing::info!("collect: jsx compiler diagnostics: {}", jsx_diags.len());
            diagnostics.extend(jsx_diags);
            return diagnostics;
        }

        let descriptor = match Self::parse_sfc_for_collect(uri, &content) {
            Ok(descriptor) => descriptor,
            Err(parse_diagnostic) => {
                tracing::info!("collect: skipping dependent diagnostics after SFC parse error");
                diagnostics.push(parse_diagnostic);
                return diagnostics;
            }
        };

        // Collect parser diagnostics for script and template blocks before
        // dependent analyzers, so broken blocks do not fan out into noisy
        // lint/type/Corsa diagnostics.
        let script_diags =
            Self::collect_script_diagnostics(uri, &content, &descriptor, &line_index);
        let has_script_parse_error = has_error_severity_diagnostic(&script_diags);
        tracing::info!("collect: script parser diagnostics: {}", script_diags.len());
        diagnostics.extend(script_diags);

        let template_diags =
            Self::collect_template_diagnostics(uri, &content, &descriptor, &line_index);
        let has_template_parse_error = has_error_severity_diagnostic(&template_diags);
        tracing::info!(
            "collect: template parser diagnostics: {}",
            template_diags.len()
        );
        diagnostics.extend(template_diags);
        if has_script_parse_error || has_template_parse_error {
            tracing::info!("collect: skipping dependent diagnostics after block parse error");
            return diagnostics;
        }

        // Surface Vue-specific compile errors (e.g. DEFINE_PROPS_DESTRUCTURE_DEFAULT_TYPE)
        // that the TypeScript checker cannot derive on its own. Mirrors the
        // canon path used by `vize check` so editor and CLI stay aligned.
        let sfc_compile_diags =
            Self::collect_sfc_compile_diagnostics(uri, &content, &descriptor, &line_index);
        tracing::info!(
            "collect: sfc compile diagnostics: {}",
            sfc_compile_diags.len()
        );
        diagnostics.extend(sfc_compile_diags);
        Self::extend_vize_sfc_type_diagnostics(state, uri, &content, &line_index, &mut diagnostics);

        Self::extend_component_required_prop_diagnostics(
            state,
            uri,
            &content,
            &descriptor,
            &line_index,
            features.typecheck,
            &mut diagnostics,
        );
        if features.lint {
            // Collect linter diagnostics (vize_patina)
            let lint_diags = Self::collect_lint_diagnostics(
                state,
                uri,
                &content,
                features.ecosystem,
                &line_index,
            );
            tracing::info!("collect: patina lint diagnostics: {}", lint_diags.len());
            diagnostics.extend(lint_diags);
            if features.cross_file {
                // Cross-file analysis is opt-in (defaults to off) because it
                // touches every Vue file in the workspace. When enabled, the
                // same analyzer groups used by `vize lint --cross-file` join
                // the editor's diagnostic stream. The actual cross-file
                // analyzer wiring is being added incrementally — for now
                // the gate is observable through tracing so callers can
                // verify the config knob took effect.
                tracing::info!(
                    "collect: cross-file lint enabled (groups will surface as they are wired up)"
                );
            }
        } else {
            tracing::info!("collect: patina lint diagnostics skipped (disabled by config)");
        }

        if features.ecosystem {
            let ecosystem_diags = ecosystem::diagnostics(&content, uri);
            tracing::info!(
                "collect: ecosystem editor diagnostics: {}",
                ecosystem_diags.len()
            );
            diagnostics.extend(ecosystem_diags);
        }

        if state.is_lsp_typecheck_enabled() && !cfg!(feature = "native") {
            let type_diags = crate::ide::TypeService::collect_diagnostics(state, uri);
            tracing::info!("collect: type checker diagnostics: {}", type_diags.len());
            diagnostics.extend(type_diags);
        } else if !state.is_lsp_typecheck_enabled() {
            tracing::info!("collect: type checker diagnostics skipped (disabled by config)");
        }

        // Also lint inline <art> blocks in regular .vue files
        if features.lint {
            let inline_art_diags = Self::collect_inline_art_diagnostics(
                state,
                uri,
                &content,
                &descriptor,
                &line_index,
            );
            tracing::info!(
                "collect: inline art diagnostics: {}",
                inline_art_diags.len()
            );
            diagnostics.extend(inline_art_diags);
        }

        diagnostics
    }

    /// Collect only the lint-sourced diagnostics (`vize/lint`, `vize/musea`)
    /// for a document.
    ///
    /// This is a fast subset of [`Self::collect`] used by the hover handler,
    /// which only ever reads diagnostics whose source is `vize/lint` or
    /// `vize/musea`. It reproduces exactly the lint/musea diagnostics that the
    /// full pipeline would publish — including the parser-error short-circuit
    /// that gates them — while skipping the expensive SFC compile, ecosystem,
    /// and uncached SFC type-check passes that hover discards.
    pub fn collect_lint_only(state: &ServerState, uri: &Url) -> Vec<Diagnostic> {
        let Some(content) = state.documents.text(uri) else {
            return vec![];
        };

        let features = state.lsp_features();
        let mut diagnostics = Vec::new();

        if !features.has_diagnostics() || !features.lint {
            return diagnostics;
        }

        // Build the line index once for this document, shared by every
        // collector below (mirrors `collect`).
        let line_index = LineIndex::new(&content);

        // Art files (*.art.vue): Musea-specific lint only.
        let path = uri.path();
        if path.ends_with(".art.vue") {
            diagnostics.extend(Self::collect_musea_diagnostics(
                state,
                uri,
                &content,
                &line_index,
            ));
            return diagnostics;
        }

        // Standalone HTML: patina lint only.
        if is_standalone_html_path(path) {
            diagnostics.extend(Self::collect_lint_diagnostics(
                state,
                uri,
                &content,
                features.ecosystem,
                &line_index,
            ));
            return diagnostics;
        }

        // Standard SFC: parse once, then mirror `collect`'s parser-error
        // short-circuit so lint only surfaces when the full pipeline would
        // also surface it.
        let Ok(descriptor) = Self::parse_sfc_for_collect(uri, &content) else {
            return diagnostics;
        };
        let script_diags =
            Self::collect_script_diagnostics(uri, &content, &descriptor, &line_index);
        let template_diags =
            Self::collect_template_diagnostics(uri, &content, &descriptor, &line_index);
        let has_block_parse_error = has_error_severity_diagnostic(&script_diags)
            || has_error_severity_diagnostic(&template_diags);
        if has_block_parse_error {
            return diagnostics;
        }

        diagnostics.extend(Self::collect_lint_diagnostics(
            state,
            uri,
            &content,
            features.ecosystem,
            &line_index,
        ));
        diagnostics.extend(Self::collect_inline_art_diagnostics(
            state,
            uri,
            &content,
            &descriptor,
            &line_index,
        ));
        diagnostics
    }

    /// Create a diagnostic from a custom error.
    pub fn create_diagnostic(
        range: Range,
        severity: Severity,
        source: &str,
        code: Option<i32>,
        message: String,
    ) -> Diagnostic {
        Diagnostic {
            range,
            severity: Some(severity.into()),
            code: code.map(NumberOrString::Number),
            source: Some(source.to_string()),
            message,
            ..Default::default()
        }
    }
}

fn has_error_severity_diagnostic(diagnostics: &[Diagnostic]) -> bool {
    diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Some(DiagnosticSeverity::ERROR))
}

/// Build the hint diagnostic surfaced when LSP type checking is requested
/// but the Corsa bridge is not available. Single point of truth so the
/// wording stays consistent for tests and follow-up code-action work.
#[cfg(feature = "native")]
pub(super) fn typecheck_unavailable_hint() -> Diagnostic {
    Diagnostic {
        range: Range {
            start: tower_lsp::lsp_types::Position {
                line: 0,
                character: 0,
            },
            end: tower_lsp::lsp_types::Position {
                line: 0,
                character: 0,
            },
        },
        severity: Some(DiagnosticSeverity::HINT),
        code: Some(NumberOrString::String("typecheck-unavailable".to_string())),
        source: Some(super::sources::TYPE_CHECKER.to_string()),
        message: super::TYPECHECK_UNAVAILABLE_HINT_MESSAGE.to_string(),
        ..Default::default()
    }
}
