//! Shared project-configured Patina constructor for diagnostics and actions.

use super::{DiagnosticService, LineIndex, linter_options, sources};
use tower_lsp::lsp_types::{
    CodeDescription, Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range, Url,
};
use vize_patina::{HelpRenderTarget, LintPreset, render_help};

pub(in crate::ide) fn linter_for_uri(
    state: &crate::server::ServerState,
    uri: &Url,
    ecosystem_enabled: bool,
) -> Option<vize_patina::Linter> {
    let (linter_config, rule_options, features) = state.linter_settings_for_uri(uri)?;
    if !linter_config.enabled {
        return None;
    }
    let preset = linter_config.preset.as_deref();
    let preset = preset.and_then(LintPreset::parse).unwrap_or_default();
    let lint_options = linter_options::resolve_patina_options(uri, &linter_config, &rule_options);
    let mut linter = if ecosystem_enabled && linter_config.preset.is_none() {
        vize_patina::Linter::with_ecosystem()
    } else {
        vize_patina::Linter::with_preset(preset)
    }
    .with_report_unused_disable_directives(linter_config.report_unused_disable_directives())
    .with_additional_rules(lint_options.additional_rules)
    .with_disabled_rules(lint_options.disabled_rules)
    .with_category_severity_overrides(lint_options.category_severity_overrides)
    .with_rule_severity_overrides(lint_options.rule_severity_overrides)
    .with_restricted_globals(lint_options.restricted_globals)
    .with_restricted_members(lint_options.restricted_members)
    .with_musea_design_tokens(lint_options.musea_design_tokens);
    linter = linter.with_vue_version(features.vue_version);
    linter = linter_options::apply_rule_options(
        linter,
        &rule_options,
        linter_options::package_is_installed(uri, "vue-router"),
        linter_options::package_is_installed(uri, "nuxt"),
    );

    #[cfg(not(target_arch = "wasm32"))]
    let linter = if linter_config.strict_reactivity_enabled() {
        linter.with_rule(Box::new(
            vize_patina::rules::type_aware::NoReactivityLoss::new(),
        ))
    } else {
        linter
    };
    Some(linter)
}

impl DiagnosticService {
    /// Collect linter diagnostics from vize_patina.
    #[expect(
        clippy::disallowed_methods,
        clippy::disallowed_macros,
        reason = "tower-lsp Diagnostic fields (message, code, source) are std String"
    )]
    pub(super) fn collect_lint_diagnostics(
        state: &crate::server::ServerState,
        uri: &Url,
        content: &str,
        ecosystem_enabled: bool,
        line_index: &LineIndex<'_>,
    ) -> Vec<Diagnostic> {
        let is_standalone_html = crate::utils::is_standalone_html_path(uri.path());
        let Some(linter) = linter_for_uri(state, uri, ecosystem_enabled) else {
            return vec![];
        };
        let mut result = if is_standalone_html {
            linter.lint_standalone_html(content, uri.path())
        } else {
            linter.lint_sfc(content, uri.path())
        };
        // Preserve configured rule precedence when visitors share a source range.
        let rule_names = linter.registry().rule_names();
        result.diagnostics.sort_by_cached_key(|diagnostic| {
            let rule_index = rule_names
                .iter()
                .position(|name| *name == diagnostic.rule_name)
                .unwrap_or(usize::MAX);
            (diagnostic.start, diagnostic.end, rule_index)
        });

        result
            .diagnostics
            .into_iter()
            .map(|lint_diag| {
                // Convert byte offsets directly in the SFC. vize_patina::lint_sfc
                // already maps template diagnostics back to source coordinates.
                let (start_line, start_col) = line_index.line_col(lint_diag.start as usize);
                let (end_line, end_col) = line_index.line_col(lint_diag.end as usize);

                // Build the diagnostic message with help text (render as plain text for LSP)
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
                    code_description: linter_options::rule_docs_url(lint_diag.rule_name)
                        .map(|href| CodeDescription { href }),
                    source: Some(sources::LINTER.to_string()),
                    message,
                    ..Default::default()
                }
            })
            .collect()
    }
}
