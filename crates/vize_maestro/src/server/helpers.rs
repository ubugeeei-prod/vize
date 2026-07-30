//! Helper methods for the Maestro LSP server.
//!
//! Provides block snippet completions, lint hover info, and
//! diagnostic publishing utilities.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, DiagnosticSeverity, Hover, HoverContents, InsertTextFormat,
    MarkupContent, MarkupKind, MessageType, NumberOrString, Position, Url,
};

use crate::ide::DiagnosticService;

use super::MaestroServer;
use vize_carton::append;

#[cfg(test)]
mod supersession_tests;

impl MaestroServer {
    /// Publish the changed document first, then refresh open Vue files that
    /// directly import it. Corsa has already received the changed virtual
    /// document by this point, so importer diagnostics observe the new shape.
    pub(crate) async fn publish_changed_diagnostics(&self, uri: &Url, content: &str) {
        self.state.update_virtual_docs(uri, content);
        let version = self.state.documents.version(uri);
        self.publish_diagnostics(uri).await;
        if !self.state.is_lsp_typecheck_enabled() {
            return;
        }

        let Some(version) = version else {
            return;
        };
        self.publish_importer_diagnostics(uri, version).await;
    }

    /// Refresh the open Vue documents that import `uri`, abandoning the fan-out
    /// as soon as a newer version of `uri` lands. Returns the importers actually
    /// refreshed, in order, so the supersession rule is directly assertable.
    ///
    /// Supersession matters because this loop is the only unbounded work a
    /// single keystroke schedules: each importer costs a full Corsa diagnostics
    /// pass, and a `.d.ts` edit fans out to *every* open Vue document
    /// ([`super::importers::open_vue_dependents`]). Without the check, typing at
    /// editor speed queues one such fan-out per keystroke, each computed from
    /// text the user has already replaced and each republished over the last —
    /// the queue-depth growth behind #3315's silent stalls.
    ///
    /// The final publish is never lost: the newest version's pass is by
    /// definition not superseded, so it always runs the fan-out to completion.
    /// Abandoning an older pass also cannot leave stale diagnostics on screen,
    /// because it stops *before* publishing rather than after computing.
    async fn publish_importer_diagnostics(&self, uri: &Url, version: i32) -> Vec<Url> {
        let mut refreshed = Vec::new();
        for importer in super::importers::open_vue_dependents(&self.state, uri) {
            if self.state.documents.version(uri) != Some(version) {
                tracing::debug!(
                    "abandoning superseded importer refresh for {}: pass version {}, current {:?}",
                    uri,
                    version,
                    self.state.documents.version(uri)
                );
                break;
            }
            self.publish_diagnostics(&importer).await;
            refreshed.push(importer);
        }
        refreshed
    }

    /// Publish diagnostics for a document.
    pub(crate) async fn publish_diagnostics(&self, uri: &Url) {
        let version = self
            .state
            .documents
            .get(uri)
            .map(|document| document.version);

        if !self.state.lsp_features().has_diagnostics() {
            self.client
                .publish_diagnostics(uri.clone(), Vec::new(), version)
                .await;
            return;
        }

        let Some(version) = version else {
            tracing::debug!("skipping diagnostics for unopened document: {}", uri);
            return;
        };

        // Use async version when native feature is enabled (includes Corsa diagnostics)
        #[cfg(feature = "native")]
        let diagnostics = DiagnosticService::collect_async(&self.state, uri).await;

        #[cfg(not(feature = "native"))]
        let diagnostics = DiagnosticService::collect(&self.state, uri);

        let current_version = self
            .state
            .documents
            .get(uri)
            .map(|document| document.version);
        if current_version != Some(version) {
            tracing::debug!(
                "skipping stale diagnostics for {}: collected version {}, current {:?}",
                uri,
                version,
                current_version
            );
            return;
        }

        self.client
            .publish_diagnostics(uri.clone(), diagnostics, Some(version))
            .await;

        // Surface a one-shot UI notification when type checking is requested
        // but Corsa never came up. The hint diagnostic emitted by
        // collect_async (see #708) shows up in the Problems panel; this
        // adds a window/showMessage so users with the Problems panel
        // collapsed also notice. See #681.
        #[cfg(feature = "native")]
        if self.state.is_lsp_typecheck_enabled()
            && !self.state.has_corsa_bridge()
            && self.state.claim_typecheck_unavailable_notice()
        {
            self.client
                .show_message(
                    MessageType::WARNING,
                    "Vize: type checking is unavailable in this workspace. \
                     Make sure tsconfig.json exists and the Corsa runtime is reachable.",
                )
                .await;
        }
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
        _content: &str,
        position: Position,
    ) -> Option<String> {
        if !self.state.is_lsp_lint_enabled() {
            return None;
        }

        // Hover only surfaces `vize/lint` / `vize/musea` diagnostics at the
        // cursor, so collect just that subset instead of re-running the entire
        // pipeline (full SFC type-check + ecosystem passes) on every hover.
        let diagnostics = DiagnosticService::collect_lint_only(&self.state, uri);

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
            let severity_icon = match diag.severity {
                Some(DiagnosticSeverity::ERROR) => "🔴",
                Some(DiagnosticSeverity::WARNING) => "🟡",
                Some(DiagnosticSeverity::INFORMATION) => "🔵",
                Some(DiagnosticSeverity::HINT) => "💡",
                _ => "⚪",
            };

            if let Some(NumberOrString::String(ref rule)) = diag.code {
                append!(markdown, "### {severity_icon} {rule}\n\n");
            }

            let parts: Vec<&str> = diag.message.split("\n\nHelp: ").collect();
            markdown.push_str(parts[0]);
            markdown.push_str("\n\n");

            if parts.len() > 1 {
                append!(markdown, "**Help:** {}\n\n", parts[1]);
            }

            if let Some(ref code_desc) = diag.code_description {
                append!(
                    markdown,
                    "[📖 View rule documentation]({})\n\n",
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
