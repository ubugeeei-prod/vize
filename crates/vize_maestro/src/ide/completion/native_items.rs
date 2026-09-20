//! Native completion presentation shared by Vue and JSX authoring.

use crate::ide::markup;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, Documentation, InsertTextFormat, MarkupContent, MarkupKind,
};
use vize_canon::{LspCompletionItem, LspDocumentation};

impl super::CompletionService {
    /// Convert a Corsa completion item to tower-lsp CompletionItem.
    pub(in crate::ide) fn convert_lsp_completion(item: LspCompletionItem) -> CompletionItem {
        let documentation = item.documentation.map(|doc| match doc {
            LspDocumentation::String(text) => Documentation::String(text),
            LspDocumentation::Markup(markup) => Documentation::MarkupContent(MarkupContent {
                kind: if markup.kind == "markdown" {
                    MarkupKind::Markdown
                } else {
                    MarkupKind::PlainText
                },
                value: markup.value,
            }),
        });
        CompletionItem {
            documentation: signature_documentation(item.detail.as_deref(), documentation),
            label: item.label,
            kind: item.kind.map(Self::convert_completion_kind),
            detail: item.detail,
            insert_text: item.insert_text,
            insert_text_format: item.insert_text_format.map(|f| {
                if f == 2 {
                    InsertTextFormat::SNIPPET
                } else {
                    InsertTextFormat::PLAIN_TEXT
                }
            }),
            filter_text: item.filter_text,
            sort_text: item.sort_text,
            ..Default::default()
        }
    }

    /// Convert LSP completion item kind number to CompletionItemKind.
    fn convert_completion_kind(kind: u32) -> CompletionItemKind {
        match kind {
            1 => CompletionItemKind::TEXT,
            2 => CompletionItemKind::METHOD,
            3 => CompletionItemKind::FUNCTION,
            4 => CompletionItemKind::CONSTRUCTOR,
            5 => CompletionItemKind::FIELD,
            6 => CompletionItemKind::VARIABLE,
            7 => CompletionItemKind::CLASS,
            8 => CompletionItemKind::INTERFACE,
            9 => CompletionItemKind::MODULE,
            10 => CompletionItemKind::PROPERTY,
            11 => CompletionItemKind::UNIT,
            12 => CompletionItemKind::VALUE,
            13 => CompletionItemKind::ENUM,
            14 => CompletionItemKind::KEYWORD,
            15 => CompletionItemKind::SNIPPET,
            16 => CompletionItemKind::COLOR,
            17 => CompletionItemKind::FILE,
            18 => CompletionItemKind::REFERENCE,
            19 => CompletionItemKind::FOLDER,
            20 => CompletionItemKind::ENUM_MEMBER,
            21 => CompletionItemKind::CONSTANT,
            22 => CompletionItemKind::STRUCT,
            23 => CompletionItemKind::EVENT,
            24 => CompletionItemKind::OPERATOR,
            25 => CompletionItemKind::TYPE_PARAMETER,
            _ => CompletionItemKind::TEXT,
        }
    }
}

fn signature_documentation(
    detail: Option<&str>,
    documentation: Option<Documentation>,
) -> Option<Documentation> {
    let Some(detail) = detail.filter(|detail| !detail.trim().is_empty()) else {
        return documentation;
    };
    let mut markdown = markup::Markdown::new().code("typescript", detail);
    match documentation {
        Some(Documentation::MarkupContent(content)) if content.kind == MarkupKind::Markdown => {
            if !content.value.trim().is_empty() {
                markdown = markdown.paragraph(content.value.trim());
            }
        }
        Some(Documentation::MarkupContent(content)) => {
            if !content.value.trim().is_empty() {
                markdown = markdown.code("text", &content.value);
            }
        }
        Some(Documentation::String(text)) if !text.trim().is_empty() => {
            markdown = markdown.code("text", &text);
        }
        _ => {}
    }
    Some(markdown.into_documentation())
}
