//! Helper methods for the Maestro LSP server.
//!
//! Provides block snippet completions, lint hover info, and
//! diagnostic publishing utilities.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, DiagnosticSeverity, Hover, HoverContents, InsertTextFormat,
    MarkupContent, MarkupKind, NumberOrString, Position, Url,
};

use crate::ide::DiagnosticService;

use super::MaestroServer;
use vize_carton::append;

impl MaestroServer {
    /// Publish diagnostics for a document.
    pub(crate) async fn publish_diagnostics(&self, uri: &Url) {
        // Use async version when native feature is enabled (includes Corsa diagnostics)
        #[cfg(feature = "native")]
        let diagnostics = DiagnosticService::collect_async(&self.state, uri).await;

        #[cfg(not(feature = "native"))]
        let diagnostics = DiagnosticService::collect(&self.state, uri);

        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
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
        let diagnostics = DiagnosticService::collect(&self.state, uri);

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
