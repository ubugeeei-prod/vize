use std::fs;

use tower_lsp::lsp_types::{
    Hover, HoverContents, MarkedString, Position, Range, TextDocumentContentChangeEvent, Url,
};

use super::HoverService;
use crate::{
    ide::{CompletionService, IdeContext},
    server::ServerState,
};

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
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        return None;
    }

    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)?;
    for candidate in [
        workspace_root.parent()?.join("corsa-bind/.cache/tsgo"),
        workspace_root
            .parent()?
            .join("corsa-bind/ref/corsa-upstream/.cache/tsgo"),
        workspace_root.join("node_modules/.bin/tsgo"),
    ] {
        if candidate.exists() {
            return Some(candidate);
        }
    }

    vize_s0::corsa_resolver::discover_corsa_in_ancestors(workspace_root)
}
