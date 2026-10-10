//! Helper methods for the Maestro LSP server.
//!
//! Provides block snippet completions, lint hover info, and
//! diagnostic publishing utilities.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "lsp_types fields store std String"
)]

use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, DiagnosticSeverity, Hover, HoverContents, InsertTextFormat,
    MarkupContent, MarkupKind, NumberOrString, Position, Url,
};

use super::MaestroServer;
use vize_l0::append;

#[cfg(test)]
mod supersession_tests;

impl MaestroServer {
    /// Refresh open typed documents that import `uri`, abandoning the fan-out
    /// as soon as a newer version of `uri` lands. Returns the importers actually
    /// refreshed, in order, so the supersession rule is directly assertable.
    ///
    /// Supersession matters because this loop is the only unbounded work a
    /// single keystroke schedules: each importer costs a full Corsa diagnostics
    /// pass, and a `.d.ts` edit fans out to *every* open typed document
    /// ([`super::importers::open_typecheck_dependents`]). Without the check, typing at
    /// editor speed queues one such fan-out per keystroke, each computed from
    /// text the user has already replaced and each republished over the last —
    /// the queue-depth growth behind #3315's silent stalls.
    ///
    /// The final publish is never lost: the newest version's pass is by
    /// definition not superseded, so it always runs the fan-out to completion.
    /// Abandoning an older pass also cannot leave stale diagnostics on screen,
    /// because it stops *before* publishing rather than after computing.
    pub(super) async fn publish_importer_diagnostics(
        &self,
        uri: &Url,
        version: Option<i32>,
    ) -> Vec<Url> {
        let mut refreshed = Vec::new();
        tracing::info!(
            "finding open typecheck dependents for {} version {:?}",
            uri,
            version
        );
        #[cfg(feature = "native")]
        let dependents = self.state.project_open_typecheck_dependents(uri);
        #[cfg(not(feature = "native"))]
        let dependents = super::importers::open_typecheck_dependents(&self.state, uri);
        tracing::info!(
            "refreshing {} open typecheck dependents for {}",
            dependents.len(),
            uri
        );
        for importer in dependents {
            if self.state.documents.version(uri) != version {
                tracing::debug!(
                    "abandoning superseded importer refresh for {}: pass version {:?}, current {:?}",
                    uri,
                    version,
                    self.state.documents.version(uri)
                );
                break;
            }
            tracing::info!("refreshing importer {} for {}", importer, uri);
            self.for_document(&importer)
                .publish_diagnostics(&importer)
                .await;
            tracing::info!("refreshed importer {} for {}", importer, uri);
            refreshed.push(importer);
        }
        refreshed
    }

    /// Publish diagnostics for a document.
    pub(crate) async fn publish_diagnostics(&self, uri: &Url) {
        self.publish_diagnostics_with_cause(uri, false).await;
    }

    /// Each explicit save retains its legacy complete notification even when
    /// neither the root version nor the source/environment changed.
    pub(crate) async fn publish_saved_diagnostics(&self, uri: &Url) {
        self.publish_diagnostics_with_cause(uri, true).await;
    }

    async fn publish_diagnostics_with_cause(&self, uri: &Url, explicit_save: bool) {
        #[cfg(feature = "native")]
        let retained = super::initial_diagnostics::RetainedDiagnostics::new(self, uri);
        // tower-lsp polls notifications concurrently. A watched declaration
        // change can therefore overlap consecutive didChange passes for the
        // same Vue file; serialize those passes so their shared Corsa virtual
        // document and overlay state cannot overtake one another.
        #[cfg(feature = "native")]
        let diagnostic_lock = self.state.diagnostic_lock(uri);
        #[cfg(feature = "native")]
        let diagnostic_guard = diagnostic_lock.lock().await;

        let diagnostics = self.collect_diagnostics_unlocked(uri, None).await;

        #[cfg(feature = "native")]
        drop(diagnostic_guard);

        if let Some(diagnostics) = diagnostics {
            self.publish_collected_diagnostics_with_cause(uri, diagnostics, explicit_save)
                .await;
        }
        #[cfg(feature = "native")]
        retained.finish();
    }

    /// Publish only if the document still has the version that scheduled the
    /// refresh. Watcher revalidation yields to a newer didChange publish.
    pub(crate) async fn publish_diagnostics_if_version(&self, uri: &Url, expected: i32) {
        #[cfg(feature = "native")]
        let retained = super::initial_diagnostics::RetainedDiagnostics::new(self, uri);
        #[cfg(feature = "native")]
        let diagnostic_lock = self.state.diagnostic_lock(uri);
        #[cfg(feature = "native")]
        let diagnostic_guard = diagnostic_lock.lock().await;

        let diagnostics = self.collect_diagnostics_unlocked(uri, Some(expected)).await;

        #[cfg(feature = "native")]
        drop(diagnostic_guard);

        if let Some(diagnostics) = diagnostics {
            self.publish_collected_diagnostics(uri, diagnostics).await;
        }
        #[cfg(feature = "native")]
        retained.finish();
    }

    /// Get block snippet completions (when outside all blocks).
    pub(crate) fn get_block_snippets(&self) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "template".to_string(),
                kind: Some(CompletionItemKind::SNIPPET),
                detail: Some("Add template block".to_string()),
                insert_text: Some("<template>\n\t$1\n</template>".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "script setup".to_string(),
                kind: Some(CompletionItemKind::SNIPPET),
                detail: Some("Add script setup block".to_string()),
                insert_text: Some("<script setup lang=\"ts\">\n$1\n</script>".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "script".to_string(),
                kind: Some(CompletionItemKind::SNIPPET),
                detail: Some("Add script block".to_string()),
                insert_text: Some(
                    "<script lang=\"ts\">\nexport default {\n\t$1\n}\n</script>".to_string(),
                ),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "style scoped".to_string(),
                kind: Some(CompletionItemKind::SNIPPET),
                detail: Some("Add scoped style block".to_string()),
                insert_text: Some("<style scoped>\n$1\n</style>".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "style".to_string(),
                kind: Some(CompletionItemKind::SNIPPET),
                detail: Some("Add style block".to_string()),
                insert_text: Some("<style>\n$1\n</style>".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
        ]
    }

    /// Get lint rule documentation for diagnostics at the given position.
    pub(crate) fn get_lint_hover_at_position(
        &self,
        uri: &Url,
        position: Position,
    ) -> Option<String> {
        if !self.state.is_lsp_lint_enabled() {
            return None;
        }

        // Reuse the lint/musea diagnostics collected for this document version.
        let diagnostics = self.state.lint_hover_diagnostics(uri);

        let lint_diags: Vec<_> = diagnostics
            .iter()
            .filter(|d| {
                let in_range = position.line >= d.range.start.line
                    && position.line <= d.range.end.line
                    && (position.line != d.range.start.line
                        || position.character >= d.range.start.character)
                    && (position.line != d.range.end.line
                        || position.character <= d.range.end.character);

                let is_lint = d
                    .source
                    .as_ref()
                    .is_some_and(|s| s == "vize/lint" || s == "vize/musea");

                in_range && is_lint
            })
            .collect();

        if lint_diags.is_empty() {
            return None;
        }

        let mut markdown = String::new();

        for diag in lint_diags {
            let severity = match diag.severity {
                Some(DiagnosticSeverity::ERROR) => "Error",
                Some(DiagnosticSeverity::WARNING) => "Warning",
                Some(DiagnosticSeverity::INFORMATION) => "Information",
                Some(DiagnosticSeverity::HINT) => "Hint",
                _ => "Diagnostic",
            };

            if let Some(NumberOrString::String(ref rule)) = diag.code {
                append!(markdown, "### {severity}: {rule}\n\n");
            }

            let mut parts = diag.message.split("\n\nHelp: ");
            markdown.push_str(parts.next().unwrap_or_default());
            markdown.push_str("\n\n");

            if let Some(help) = parts.next() {
                append!(markdown, "**Help:** {help}\n\n");
            }

            if let Some(ref code_desc) = diag.code_description {
                append!(
                    markdown,
                    "[View rule documentation]({})\n\n",
                    code_desc.href
                );
            }

            markdown.push_str("---\n\n");
        }

        if markdown.ends_with("---\n\n") {
            markdown.truncate(markdown.len() - 5);
        }

        Some(markdown)
    }

    /// Merge hover content with lint information.
    pub(crate) fn merge_hover_with_lint(hover: Option<Hover>, lint_info: String) -> Hover {
        match hover {
            Some(mut h) => {
                if let HoverContents::Markup(ref mut markup) = h.contents {
                    markup.value.push_str("\n\n---\n\n");
                    markup.value.push_str(&lint_info);
                }
                h
            }
            None => Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: lint_info,
                }),
                range: None,
            },
        }
    }
}
