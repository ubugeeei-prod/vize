//! The diagnostic service: aggregation orchestration and shared types.

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Range, Url};

use crate::ide::ecosystem;
use crate::server::ServerState;
use crate::utils::{is_jsx_path, is_standalone_html_path};

use super::{LineIndex, Severity, sources};

/// Source position mapping from @vize-map comments.
#[cfg(feature = "native")]
#[derive(Debug, Clone)]
pub(in crate::ide) struct SourceMapping {
    /// Byte offset start in SFC
    pub(in crate::ide) start: u32,
    /// Byte offset end in SFC
    pub(in crate::ide) end: u32,
}

/// Virtual TypeScript generation result with position mapping info.
#[cfg(feature = "native")]
pub(in crate::ide) struct VirtualTsResult {
    /// Generated TypeScript code (post `.vue` → `.vue.ts` import rewrite).
    pub(in crate::ide) code: String,
    /// Byte-range source mappings from generated TS back to the source SFC.
    /// Offsets are in pre-rewrite generated TS coordinates; callers must
    /// translate post-rewrite byte offsets via `import_source_map` first.
    pub(in crate::ide) source_mappings: Vec<vize_canon::virtual_ts::VizeMapping>,
    /// Byte-offset mapping from post-rewrite to pre-rewrite virtual TS.
    /// Empty when no `.vue` import specifiers were rewritten.
    pub(in crate::ide) import_source_map: vize_canon::ImportSourceMap,
    /// Relative `.vue` import specifiers found in the SFC's script. The
    /// editor session overlays each sibling's virtual TS so relative
    /// imports resolve under the temp-dir Corsa session (issue #752).
    pub(in crate::ide) relative_vue_imports: Vec<std::string::String>,
    /// Line number where user code starts in virtual TS (0-indexed)
    pub(in crate::ide) user_code_start_line: u32,
    /// Line number where script starts in original SFC (1-indexed)
    pub(in crate::ide) sfc_script_start_line: u32,
    /// Line number where template scope starts in virtual TS (0-indexed)
    pub(in crate::ide) template_scope_start_line: u32,
    /// Line-to-source mappings from @vize-map comments
    /// Index is virtual TS line number (0-indexed), value is source position in SFC
    pub(in crate::ide) line_mappings: Vec<Option<SourceMapping>>,
    /// Number of import lines skipped from user code (to adjust line mapping)
    pub(in crate::ide) skipped_import_lines: u32,
}

/// Diagnostic service for collecting and aggregating diagnostics.
pub struct DiagnosticService;

impl DiagnosticService {
    /// Collect all diagnostics for a document.
    pub fn collect(state: &ServerState, uri: &Url) -> Vec<Diagnostic> {
        let Some(doc) = state.documents.get(uri) else {
            tracing::warn!("collect: document not found for {}", uri);
            return vec![];
        };

        let content = doc.text();
        let mut diagnostics = Vec::new();
        let features = state.lsp_features();

        if !features.has_diagnostics() {
            return diagnostics;
        }

        // Build the line index once for this document. Every collector below
        // maps byte offsets in `content` to (line, utf16_col) against it, so
        // sharing one index avoids re-scanning the whole document per offset.
        let line_index = LineIndex::new(&content);

        // Check if this is an Art file (*.art.vue)
        let path = uri.path();
        if path.ends_with(".art.vue") {
            // Musea-specific diagnostics for Art files
            if features.lint {
                diagnostics.extend(Self::collect_musea_diagnostics(uri, &content, &line_index));
            }
            // Don't return early here; async collection still adds Corsa diagnostics.
            return diagnostics;
        }

        if is_standalone_html_path(path) {
            if features.lint {
                let linter_config = state.get_linter_config();
                let lint_diags = Self::collect_lint_diagnostics(
                    uri,
                    &content,
                    features.ecosystem,
                    &linter_config,
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

        // JSX/TSX files (*.jsx, *.tsx): surface JSX compiler/lowering
        // diagnostics (parse errors, lowering warnings) as LSP squiggles. This
        // is diagnostics-only — no virtual TypeScript document is generated for
        // JSX/TSX (type-aware features are deferred to #1497).
        if is_jsx_path(path) {
            let jsx_diags = Self::collect_jsx_diagnostics(uri, &content, &line_index);
            tracing::info!("collect: jsx compiler diagnostics: {}", jsx_diags.len());
            diagnostics.extend(jsx_diags);
            return diagnostics;
        }

        // Standard SFC processing — parse once and share the descriptor with
        // every block-level collector below.
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

        if features.lint {
            // Collect linter diagnostics (vize_patina)
            let linter_config = state.get_linter_config();
            let lint_diags = Self::collect_lint_diagnostics(
                uri,
                &content,
                features.ecosystem,
                &linter_config,
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

        if state.is_lsp_typecheck_enabled() {
            // Collect type checker diagnostics (vize_canon)
            let type_diags = crate::ide::TypeService::collect_diagnostics(state, uri);
            tracing::info!("collect: type checker diagnostics: {}", type_diags.len());
            diagnostics.extend(type_diags);
        } else {
            tracing::info!("collect: type checker diagnostics skipped (disabled by config)");
        }

        // Also lint inline <art> blocks in regular .vue files
        if features.lint {
            let inline_art_diags =
                Self::collect_inline_art_diagnostics(uri, &content, &descriptor, &line_index);
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
    /// and (per-call, uncached) SFC type-check passes that hover discards. The
    /// returned set is byte-for-byte the lint/musea subset of `collect`'s
    /// output for every document shape (Art file, standalone HTML, SFC).
    pub fn collect_lint_only(state: &ServerState, uri: &Url) -> Vec<Diagnostic> {
        let Some(doc) = state.documents.get(uri) else {
            return vec![];
        };

        let content = doc.text();
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
            diagnostics.extend(Self::collect_musea_diagnostics(uri, &content, &line_index));
            return diagnostics;
        }

        // Standalone HTML: patina lint only.
        if is_standalone_html_path(path) {
            let linter_config = state.get_linter_config();
            diagnostics.extend(Self::collect_lint_diagnostics(
                uri,
                &content,
                features.ecosystem,
                &linter_config,
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

        let linter_config = state.get_linter_config();
        diagnostics.extend(Self::collect_lint_diagnostics(
            uri,
            &content,
            features.ecosystem,
            &linter_config,
            &line_index,
        ));
        diagnostics.extend(Self::collect_inline_art_diagnostics(
            uri,
            &content,
            &descriptor,
            &line_index,
        ));
        diagnostics
    }

    /// Collect diagnostics asynchronously (includes Corsa diagnostics when available).
    #[cfg(feature = "native")]
    pub async fn collect_async(state: &ServerState, uri: &Url) -> Vec<Diagnostic> {
        tracing::info!("collect_async: {}", uri);

        // Start with sync diagnostics (patina, etc.)
        let mut diagnostics = Self::collect(state, uri);
        tracing::info!("sync diagnostics count: {}", diagnostics.len());
        if has_blocking_parser_error(&diagnostics) {
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
                Ok(corsa_diags) => {
                    tracing::info!("corsa diagnostics count: {}", corsa_diags.len());
                    diagnostics.extend(corsa_diags);
                }
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
                    .any(|d| d.source.as_deref() == Some(sources::TYPE_CHECKER))
            {
                diagnostics.push(typecheck_unavailable_hint());
            }
        } else {
            tracing::info!("collect_async: Corsa diagnostics skipped (disabled by config)");
        }

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
fn typecheck_unavailable_hint() -> Diagnostic {
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
        source: Some(sources::TYPE_CHECKER.to_string()),
        message: "Type checking is unavailable in this workspace. \
            Make sure `tsconfig.json` exists and the Corsa runtime is reachable; \
            see https://vizejs.dev/guide/static-analysis."
            .to_string(),
        ..Default::default()
    }
}

#[cfg(feature = "native")]
fn has_blocking_parser_error(diagnostics: &[Diagnostic]) -> bool {
    diagnostics.iter().any(|diagnostic| {
        matches!(
            diagnostic.source.as_deref(),
            // Parser-level errors plus SFC compile-time validation errors
            // both leave the script body in a state where Corsa would just
            // cascade — the user already has the actionable diagnostic.
            Some(
                sources::SFC_PARSER
                    | sources::SCRIPT_PARSER
                    | sources::TEMPLATE_PARSER
                    | sources::SFC_COMPILER
            )
        ) && diagnostic.severity == Some(DiagnosticSeverity::ERROR)
    })
}
