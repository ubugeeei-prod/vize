//! Diagnostics aggregation from multiple sources.
//!
//! Aggregates diagnostics from:
//! - SFC parser errors
//! - Template parser errors
//! - vize_patina (linter)
//! - Future: vize_canon (type checker)
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

mod collectors;
#[cfg(feature = "native")]
mod corsa;

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Range, Url};

use crate::ide::ecosystem;
use crate::server::ServerState;
use crate::utils::is_standalone_html_path;

/// Diagnostic source identifiers.
pub mod sources {
    pub const SFC_PARSER: &str = "vize/sfc";
    pub const SFC_COMPILER: &str = "vize/sfc-compile";
    pub const TEMPLATE_PARSER: &str = "vize/template";
    pub const SCRIPT_PARSER: &str = "vize/script";
    pub const LINTER: &str = "vize/lint";
    pub const TYPE_CHECKER: &str = "vize/types";
    pub const MUSEA: &str = "vize/musea";
}

/// Diagnostic severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Information,
    Hint,
}

impl From<Severity> for DiagnosticSeverity {
    fn from(s: Severity) -> Self {
        match s {
            Severity::Error => DiagnosticSeverity::ERROR,
            Severity::Warning => DiagnosticSeverity::WARNING,
            Severity::Information => DiagnosticSeverity::INFORMATION,
            Severity::Hint => DiagnosticSeverity::HINT,
        }
    }
}

/// Source position mapping from @vize-map comments.
#[cfg(feature = "native")]
#[derive(Debug, Clone)]
pub(super) struct SourceMapping {
    /// Byte offset start in SFC
    pub(super) start: u32,
    /// Byte offset end in SFC
    pub(super) end: u32,
}

/// Virtual TypeScript generation result with position mapping info.
#[cfg(feature = "native")]
pub(super) struct VirtualTsResult {
    /// Generated TypeScript code
    pub(super) code: String,
    /// Byte-range source mappings from generated TS back to the source SFC.
    pub(super) source_mappings: Vec<vize_canon::virtual_ts::VizeMapping>,
    /// Line number where user code starts in virtual TS (0-indexed)
    pub(super) user_code_start_line: u32,
    /// Line number where script starts in original SFC (1-indexed)
    pub(super) sfc_script_start_line: u32,
    /// Line number where template scope starts in virtual TS (0-indexed)
    pub(super) template_scope_start_line: u32,
    /// Line-to-source mappings from @vize-map comments
    /// Index is virtual TS line number (0-indexed), value is source position in SFC
    pub(super) line_mappings: Vec<Option<SourceMapping>>,
    /// Number of import lines skipped from user code (to adjust line mapping)
    pub(super) skipped_import_lines: u32,
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

        // Check if this is an Art file (*.art.vue)
        let path = uri.path();
        if path.ends_with(".art.vue") {
            // Musea-specific diagnostics for Art files
            if features.lint {
                diagnostics.extend(Self::collect_musea_diagnostics(uri, &content));
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
                );
                tracing::info!(
                    "collect: standalone HTML patina lint diagnostics: {}",
                    lint_diags.len()
                );
                diagnostics.extend(lint_diags);
            }
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
        let script_diags = Self::collect_script_diagnostics(uri, &content, &descriptor);
        let has_script_parse_error = !script_diags.is_empty();
        tracing::info!("collect: script parser diagnostics: {}", script_diags.len());
        diagnostics.extend(script_diags);

        let template_diags = Self::collect_template_diagnostics(uri, &content, &descriptor);
        let has_template_parse_error = !template_diags.is_empty();
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
        let sfc_compile_diags = Self::collect_sfc_compile_diagnostics(uri, &content, &descriptor);
        tracing::info!(
            "collect: sfc compile diagnostics: {}",
            sfc_compile_diags.len()
        );
        diagnostics.extend(sfc_compile_diags);

        if features.lint {
            // Collect linter diagnostics (vize_patina)
            let linter_config = state.get_linter_config();
            let lint_diags =
                Self::collect_lint_diagnostics(uri, &content, features.ecosystem, &linter_config);
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
            let type_diags = super::TypeService::collect_diagnostics(state, uri);
            tracing::info!("collect: type checker diagnostics: {}", type_diags.len());
            diagnostics.extend(type_diags);
        } else {
            tracing::info!("collect: type checker diagnostics skipped (disabled by config)");
        }

        // Also lint inline <art> blocks in regular .vue files
        if features.lint {
            let inline_art_diags = Self::collect_inline_art_diagnostics(uri, &content, &descriptor);
            tracing::info!(
                "collect: inline art diagnostics: {}",
                inline_art_diags.len()
            );
            diagnostics.extend(inline_art_diags);
        }

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

/// Builder for creating diagnostics.
pub struct DiagnosticBuilder {
    range: Range,
    severity: Severity,
    source: String,
    code: Option<i32>,
    message: String,
    related_information: Vec<tower_lsp::lsp_types::DiagnosticRelatedInformation>,
}

impl DiagnosticBuilder {
    /// Create a new diagnostic builder.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            range: Range::default(),
            severity: Severity::Error,
            source: "vize".to_string(),
            code: None,
            message: message.into(),
            related_information: Vec::new(),
        }
    }

    /// Set the range.
    pub fn range(mut self, range: Range) -> Self {
        self.range = range;
        self
    }

    /// Set the severity.
    pub fn severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    /// Set the source.
    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }

    /// Set the error code.
    pub fn code(mut self, code: i32) -> Self {
        self.code = Some(code);
        self
    }

    /// Add related information.
    pub fn related(
        mut self,
        location: tower_lsp::lsp_types::Location,
        message: impl Into<String>,
    ) -> Self {
        self.related_information
            .push(tower_lsp::lsp_types::DiagnosticRelatedInformation {
                location,
                message: message.into(),
            });
        self
    }

    /// Build the diagnostic.
    pub fn build(self) -> Diagnostic {
        Diagnostic {
            range: self.range,
            severity: Some(self.severity.into()),
            code: self.code.map(NumberOrString::Number),
            source: Some(self.source),
            message: self.message,
            related_information: if self.related_information.is_empty() {
                None
            } else {
                Some(self.related_information)
            },
            ..Default::default()
        }
    }
}

/// Convert byte offset to (line, column) - both 0-indexed for LSP.
pub(super) fn offset_to_line_col(source: &str, offset: usize) -> (u32, u32) {
    let mut line = 0u32;
    let mut col = 0u32;
    let mut current_offset = 0;

    for ch in source.chars() {
        if current_offset >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += ch.len_utf16() as u32;
        }
        current_offset += ch.len_utf8();
    }

    (line, col)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{DiagnosticBuilder, DiagnosticService, Severity, offset_to_line_col, sources};
    use crate::server::ServerState;
    use tower_lsp::lsp_types::{DiagnosticSeverity, NumberOrString, Url};

    fn state_with_lsp_diagnostics(lint: bool, typecheck: bool) -> ServerState {
        let state = ServerState::new();
        state.apply_lsp_initialization_options(Some(&serde_json::json!({
            "lint": lint,
            "typecheck": typecheck
        })));
        state
    }

    fn state_with_ecosystem_diagnostics() -> ServerState {
        let state = ServerState::new();
        state.apply_lsp_initialization_options(Some(&serde_json::json!({
            "ecosystem": true
        })));
        state
    }

    #[test]
    fn test_diagnostic_builder() {
        let diagnostic = DiagnosticBuilder::new("Test error")
            .severity(Severity::Warning)
            .source("test")
            .code(42)
            .build();

        assert_eq!(diagnostic.message, "Test error");
        assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::WARNING));
        assert_eq!(diagnostic.source, Some("test".to_string()));
        assert_eq!(diagnostic.code, Some(NumberOrString::Number(42)));
    }

    #[test]
    fn test_severity_conversion() {
        assert_eq!(
            DiagnosticSeverity::from(Severity::Error),
            DiagnosticSeverity::ERROR
        );
        assert_eq!(
            DiagnosticSeverity::from(Severity::Warning),
            DiagnosticSeverity::WARNING
        );
        assert_eq!(
            DiagnosticSeverity::from(Severity::Information),
            DiagnosticSeverity::INFORMATION
        );
        assert_eq!(
            DiagnosticSeverity::from(Severity::Hint),
            DiagnosticSeverity::HINT
        );
    }

    #[test]
    fn offset_to_line_col_counts_utf16_code_units() {
        let source = "const icon = \"😀\"; missing";
        let offset = source.find("missing").unwrap();

        assert_eq!(offset_to_line_col(source, offset), (0, 19));
    }

    #[test]
    fn collect_short_circuits_dependent_diagnostics_after_sfc_parse_error() {
        let state = state_with_lsp_diagnostics(true, true);
        let uri = Url::parse("file:///Broken.vue").unwrap();
        state.documents.open(
            uri.clone(),
            "<template><div></div>".to_string(),
            1,
            "vue".to_string(),
        );

        let diagnostics = DiagnosticService::collect(&state, &uri);
        let diagnostic_sources: Vec<_> = diagnostics
            .iter()
            .filter_map(|diagnostic| diagnostic.source.as_deref())
            .collect();

        assert_eq!(diagnostic_sources, vec![sources::SFC_PARSER]);
    }

    #[test]
    fn collect_keeps_type_diagnostics_for_parseable_sfc() {
        let state = state_with_lsp_diagnostics(false, true);
        let uri = Url::parse("file:///Component.vue").unwrap();
        state.documents.open(
            uri.clone(),
            "<script setup>const props = defineProps(['count'])</script><template>{{ props.count }}</template>".to_string(),
            1,
            "vue".to_string(),
        );

        let diagnostics = DiagnosticService::collect(&state, &uri);

        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.source.as_deref() == Some(sources::TYPE_CHECKER))
        );
        assert!(
            !diagnostics
                .iter()
                .any(|diagnostic| diagnostic.source.as_deref() == Some(sources::SFC_PARSER))
        );
    }

    #[test]
    fn collect_reports_props_destructure_default_type_mismatch_for_lsp() {
        // Editor should surface the same DEFINE_PROPS_DESTRUCTURE_DEFAULT_TYPE
        // error that `vize check` shows, since TypeScript's checker cannot
        // detect a destructure default that conflicts with a Vue prop type.
        let state = state_with_lsp_diagnostics(false, false);
        let uri = Url::parse("file:///Bad.vue").unwrap();
        state.documents.open(
            uri.clone(),
            r#"<script setup lang="ts">
const { msg = 0 } = defineProps<{ msg?: string }>();
</script>

<template>
  <div>{{ msg }}</div>
</template>
"#
            .to_string(),
            1,
            "vue".to_string(),
        );

        let diagnostics = DiagnosticService::collect(&state, &uri);
        let diagnostic = diagnostics
            .iter()
            .find(|diagnostic| diagnostic.source.as_deref() == Some(sources::SFC_COMPILER))
            .expect("expected an SFC compile diagnostic");
        assert!(
            diagnostic
                .message
                .contains("DEFINE_PROPS_DESTRUCTURE_DEFAULT_TYPE"),
            "expected DEFINE_PROPS_DESTRUCTURE_DEFAULT_TYPE in message, got: {}",
            diagnostic.message
        );
        assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
    }

    #[test]
    fn collect_does_not_report_sfc_compile_diagnostic_for_valid_default() {
        let state = state_with_lsp_diagnostics(false, false);
        let uri = Url::parse("file:///Good.vue").unwrap();
        state.documents.open(
            uri.clone(),
            r#"<script setup lang="ts">
const { msg = "ok" } = defineProps<{ msg?: string }>();
</script>

<template>
  <div>{{ msg }}</div>
</template>
"#
            .to_string(),
            1,
            "vue".to_string(),
        );

        let diagnostics = DiagnosticService::collect(&state, &uri);
        assert!(
            diagnostics
                .iter()
                .all(|diagnostic| diagnostic.source.as_deref() != Some(sources::SFC_COMPILER)),
            "expected no SFC compile diagnostics, got: {:?}",
            diagnostics
                .iter()
                .map(|diagnostic| (diagnostic.source.as_deref(), diagnostic.message.as_str()))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn collect_surfaces_sfc_level_lint_diagnostics() {
        let state = state_with_lsp_diagnostics(true, false);
        let uri = Url::parse("file:///OutOfOrder.vue").unwrap();
        state.documents.open(
            uri.clone(),
            "<template><div /></template>\n<script setup>const count = 1</script>".to_string(),
            1,
            "vue".to_string(),
        );

        let diagnostics = DiagnosticService::collect(&state, &uri);
        let diagnostic = diagnostics
            .iter()
            .find(|diagnostic| {
                diagnostic.source.as_deref() == Some(sources::LINTER)
                    && diagnostic.code
                        == Some(NumberOrString::String("vue/sfc-element-order".to_string()))
            })
            .expect("SFC-level lint diagnostic");

        assert_eq!(diagnostic.range.start.line, 1);
        assert!(diagnostic.message.contains("<script> should come before"));
    }

    #[test]
    fn collect_lints_standalone_html_with_configured_rule() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("vize.config.json"),
            r#"{
                "lsp": { "lint": true },
                "linter": {
                    "rules": {
                        "script/no-options-api": "error"
                    }
                }
            }"#,
        )
        .unwrap();

        let state = ServerState::new();
        state.load_lsp_config(dir.path());

        let source_path = dir.path().join("index.html");
        let uri = Url::from_file_path(&source_path).unwrap();
        let source = r##"<!doctype html>
<html>
<head>
  <script src="https://unpkg.com/vue@3/dist/vue.global.js"></script>
</head>
<body>
  <div id="app">{{ count }}</div>
  <script>
Vue.createApp({
  data() {
    return { count: 0 }
  }
}).mount("#app")
  </script>
</body>
</html>
"##;
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "html".to_string());
        state.update_virtual_docs(&uri, source);

        let diagnostics = DiagnosticService::collect(&state, &uri);

        assert!(state.get_virtual_docs(&uri).is_some());
        assert!(
            diagnostics
                .iter()
                .all(|diagnostic| diagnostic.source.as_deref() != Some(sources::SFC_PARSER))
        );
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.source.as_deref() == Some(sources::LINTER)
                && diagnostic.code
                    == Some(NumberOrString::String("script/no-options-api".to_string()))
        }));
    }

    #[test]
    fn collect_reports_unknown_file_route_params() {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("src/pages/users/[id].vue");
        fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        let source = r#"<script setup lang="ts">
import { useRoute } from "vue-router"
const route = useRoute()
route.params.slug
</script>"#;
        fs::write(&source_path, source).unwrap();

        let state = state_with_ecosystem_diagnostics();
        let uri = Url::from_file_path(&source_path).unwrap();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());

        let diagnostics = DiagnosticService::collect(&state, &uri);

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.source.as_deref() == Some("vize/ecosystem")
                && diagnostic.code
                    == Some(NumberOrString::String(
                        "ecosystem/vue-router-route-param".to_string(),
                    ))
        }));
    }

    #[test]
    fn collect_reports_missing_define_art_source_file() {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("MissingSource.art.vue");
        let uri = Url::from_file_path(&source_path).unwrap();
        let source = r#"<script setup lang="ts">
defineArt("./Missing.vue", {
  title: "Missing",
});
</script>

<art>
  <variant name="Default">
    <Missing />
  </variant>
</art>
"#;

        let state = state_with_lsp_diagnostics(true, false);
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "art-vue".to_string());

        let diagnostics = DiagnosticService::collect(&state, &uri);

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.source.as_deref() == Some(sources::MUSEA)
                && diagnostic.code
                    == Some(NumberOrString::String(
                        "musea/define-art-source-not-found".to_string(),
                    ))
        }));
    }

    #[test]
    fn collect_reports_unknown_route_path_params() {
        let source = r#"<script setup lang="ts">
import { useRoute } from "vue-router"
const route = useRoute()
route.params.slug
</script>
<route lang="json">
{ "path": "/users/:id" }
</route>"#;

        let state = state_with_ecosystem_diagnostics();
        let uri = Url::parse("file:///RoutePathParams.vue").unwrap();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());

        let diagnostics = DiagnosticService::collect(&state, &uri);
        let diagnostic = diagnostics
            .iter()
            .find(|diagnostic| {
                diagnostic.source.as_deref() == Some("vize/ecosystem")
                    && diagnostic.code
                        == Some(NumberOrString::String(
                            "ecosystem/vue-router-route-param".to_string(),
                        ))
            })
            .expect("unknown route path param diagnostic");

        assert!(diagnostic.message.contains("Available params: id"));
    }

    #[test]
    fn collect_reports_workspace_i18n_missing_key() {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("src/components/LoginButton.vue");
        let locale_path = dir.path().join("src/locales/en.json");
        fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        fs::create_dir_all(locale_path.parent().unwrap()).unwrap();
        fs::write(&locale_path, r#"{ "auth": { "login": "Log in" } }"#).unwrap();

        let source = r#"<script setup lang="ts">
const title = t("auth.missing")
</script>"#;
        fs::write(&source_path, source).unwrap();

        let state = state_with_ecosystem_diagnostics();
        let uri = Url::from_file_path(&source_path).unwrap();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());

        let diagnostics = DiagnosticService::collect(&state, &uri);

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.source.as_deref() == Some("vize/ecosystem")
                && diagnostic.code
                    == Some(NumberOrString::String(
                        "ecosystem/vue-i18n-no-missing-key".to_string(),
                    ))
        }));
    }

    #[test]
    fn collect_short_circuits_dependent_diagnostics_after_script_parse_error() {
        let state = state_with_lsp_diagnostics(true, true);
        let uri = Url::parse("file:///BrokenScript.vue").unwrap();
        state.documents.open(
            uri.clone(),
            "<script setup lang=\"ts\">\nconst count =</script>\n<template>{{ count }}</template>"
                .to_string(),
            1,
            "vue".to_string(),
        );

        let diagnostics = DiagnosticService::collect(&state, &uri);
        let diagnostic_sources: Vec<_> = diagnostics
            .iter()
            .filter_map(|diagnostic| diagnostic.source.as_deref())
            .collect();

        assert_eq!(diagnostic_sources, vec![sources::SCRIPT_PARSER]);
        assert_eq!(
            diagnostics[0].code,
            Some(NumberOrString::String("script-parse-error".to_string()))
        );
        assert_eq!(diagnostics[0].range.start.line, 1);
    }

    #[test]
    fn collect_accepts_jsx_script_without_false_parse_diagnostic() {
        let state = state_with_lsp_diagnostics(false, false);
        let uri = Url::parse("file:///JsxScript.vue").unwrap();
        state.documents.open(
            uri.clone(),
            "<script setup lang=\"jsx\">\nconst count = 1\nconst vnode = <button>{count}</button>\n</script>"
                .to_string(),
            1,
            "vue".to_string(),
        );

        let diagnostics = DiagnosticService::collect(&state, &uri);

        assert!(
            !diagnostics
                .iter()
                .any(|diagnostic| diagnostic.source.as_deref() == Some(sources::SCRIPT_PARSER))
        );
    }

    #[test]
    fn collect_accepts_tsx_script_without_false_parse_diagnostic() {
        let state = state_with_lsp_diagnostics(false, false);
        let uri = Url::parse("file:///TsxScript.vue").unwrap();
        state.documents.open(
            uri.clone(),
            "<script setup lang=\"tsx\">\nconst count = 1\nconst vnode = <button>{count}</button>\n</script>"
                .to_string(),
            1,
            "vue".to_string(),
        );

        let diagnostics = DiagnosticService::collect(&state, &uri);

        assert!(
            !diagnostics
                .iter()
                .any(|diagnostic| diagnostic.source.as_deref() == Some(sources::SCRIPT_PARSER))
        );
    }

    #[test]
    fn collect_skips_unsupported_script_language() {
        let state = state_with_lsp_diagnostics(false, false);
        let uri = Url::parse("file:///CoffeeScript.vue").unwrap();
        state.documents.open(
            uri.clone(),
            "<script setup lang=\"coffee\">\ncount = ->\n</script>".to_string(),
            1,
            "vue".to_string(),
        );

        let diagnostics = DiagnosticService::collect(&state, &uri);

        assert!(
            !diagnostics
                .iter()
                .any(|diagnostic| diagnostic.source.as_deref() == Some(sources::SCRIPT_PARSER))
        );
    }

    #[test]
    fn collect_short_circuits_dependent_diagnostics_after_template_parse_error() {
        let state = state_with_lsp_diagnostics(true, true);
        let uri = Url::parse("file:///BrokenTemplate.vue").unwrap();
        state.documents.open(
            uri.clone(),
            "<script setup lang=\"ts\">const count = 1</script>\n<template><div>{{ count }}</template>"
                .to_string(),
            1,
            "vue".to_string(),
        );

        let diagnostics = DiagnosticService::collect(&state, &uri);
        let diagnostic_sources: Vec<_> = diagnostics
            .iter()
            .filter_map(|diagnostic| diagnostic.source.as_deref())
            .collect();

        assert_eq!(diagnostic_sources, vec![sources::TEMPLATE_PARSER]);
    }
}
