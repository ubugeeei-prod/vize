//! Art template/script findings alongside the existing Musea collector.

use super::{DiagnosticService, LineIndex};
use crate::server::ServerState;
use tower_lsp::lsp_types::{Diagnostic, NumberOrString, Url};

impl DiagnosticService {
    pub(super) fn collect_art_lint_diagnostics(
        state: &ServerState,
        uri: &Url,
        content: &str,
        ecosystem_enabled: bool,
        line_index: &LineIndex<'_>,
    ) -> Vec<Diagnostic> {
        let mut diagnostics =
            Self::collect_lint_diagnostics(state, uri, content, ecosystem_enabled, line_index);
        // The existing dedicated collector owns Musea metadata/style findings
        // and preserves their source, severity and documentation links.
        diagnostics.retain(|diagnostic| {
            !matches!(&diagnostic.code, Some(NumberOrString::String(rule)) if rule.starts_with("musea/"))
        });
        diagnostics
    }
}
