//! Plugin lint rules in the editor (#13074), on the shared per-document host.

use vize_patina::LintDiagnostic;
pub(crate) use vize_patina::plugin_batch::lint::host::{PluginLintHost, PluginLintOutcome};

impl super::ServerState {
    /// Plugin diagnostics of one Vue document that carry a fix, each with
    /// whether its rule is declared safe.
    pub(crate) fn plugin_lint_fixes(
        &self,
        uri: &tower_lsp::lsp_types::Url,
        content: &str,
    ) -> Vec<(LintDiagnostic, bool)> {
        let Some(host) = self.plugin_lint_for_uri(uri) else {
            return Vec::new();
        };
        let version = self.documents.version(uri);
        let cancelled = || self.documents.version(uri) != version;
        match host.lint(content, uri.path(), &cancelled) {
            PluginLintOutcome::Diagnostics(found) => found
                .into_iter()
                .filter(|diagnostic| diagnostic.fix.is_some())
                .map(|diagnostic| {
                    let safe = host.is_safe_fix(diagnostic.rule_name);
                    (diagnostic, safe)
                })
                .collect(),
            PluginLintOutcome::Cancelled | PluginLintOutcome::Failed(_) => Vec::new(),
        }
    }

    /// Add the plugin lint diagnostics of one Vue document. A failure is one
    /// error at the start of the document, never a silent skip.
    pub(crate) fn extend_plugin_lint_diagnostics(
        &self,
        uri: &tower_lsp::lsp_types::Url,
        content: &str,
        diagnostics: &mut Vec<LintDiagnostic>,
    ) {
        let Some(host) = self.plugin_lint_for_uri(uri) else {
            return;
        };
        let version = self.documents.version(uri);
        let cancelled = || self.documents.version(uri) != version;
        match host.lint(content, uri.path(), &cancelled) {
            PluginLintOutcome::Diagnostics(found) => diagnostics.extend(found),
            // The superseding version publishes its own pass.
            PluginLintOutcome::Cancelled => {}
            PluginLintOutcome::Failed(error) => {
                diagnostics.push(LintDiagnostic::error("plugin", error.as_str(), 0, 0));
            }
        }
    }
}
