use std::fs;

use tower_lsp::lsp_types::{
    Hover, HoverContents, MarkedString, Position, Range, TextDocumentContentChangeEvent, Url,
};

use super::HoverService;
use crate::{
    ide::{CompletionService, IdeContext},
    server::ServerState,
};

#[expect(clippy::string_slice, reason = "tests assert by panicking")]
mod art_variants;
mod component_props;
mod template_bindings;

fn hover_markdown(hover: Hover) -> String {
    match hover.contents {
        HoverContents::Markup(content) => content.value,
        HoverContents::Scalar(value) => marked_string_value(value),
        HoverContents::Array(parts) => parts
            .into_iter()
            .map(marked_string_value)
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

fn marked_string_value(value: MarkedString) -> String {
    match value {
        MarkedString::String(value) => value,
        MarkedString::LanguageString(value) => value.value,
    }
}

fn resolve_tsgo_binary() -> Option<std::path::PathBuf> {
    crate::test_support::tsgo()
}
