//! `textDocument/documentSymbol`: the SFC block outline.
//!
//! Moved out of `handlers.rs` (over the per-file length budget) into the
//! document-structure group: the outline is the same block layout that folding
//! and selection ranges read.
// `DocumentSymbol::deprecated` is deprecated in the LSP crate but still part of
// the struct literal.
#![allow(deprecated, clippy::disallowed_methods)]

use tower_lsp::lsp_types::{
    DocumentSymbol, DocumentSymbolParams, DocumentSymbolResponse, Position, Range, SymbolKind,
};
use vize_atelier_sfc::BlockLocation;
use vize_s0::line_index::LineIndex;

use crate::server::ServerState;

fn block_ranges(index: &LineIndex<'_>, loc: &BlockLocation, tag_name: &str) -> (Range, Range) {
    let position = |offset| {
        let (line, character) = index.line_col(offset);
        Position { line, character }
    };
    (
        Range {
            start: position(loc.tag_start),
            end: position(loc.tag_end),
        },
        Range {
            start: position(loc.tag_start + 1),
            end: position(loc.tag_start + 1 + tag_name.len()),
        },
    )
}

/// Build the block outline for `params.text_document`.
pub(crate) fn document_symbols(
    state: &ServerState,
    params: &DocumentSymbolParams,
) -> Option<DocumentSymbolResponse> {
    let uri = &params.text_document.uri;

    let content = state.documents.text(uri)?;

    // `.jsx`/`.tsx` documents have no SFC blocks; list their component
    // functions instead. Structural (parse-based), so it is not gated on
    // `typeChecker.jsxTypecheck`.
    if crate::utils::is_jsx_path(uri.path()) {
        return crate::ide::JsxDocumentSymbolsService::symbols(&content, uri)
            .map(DocumentSymbolResponse::Nested);
    }

    let options = vize_atelier_sfc::SfcParseOptions {
        filename: uri.path().to_string().into(),
        ..Default::default()
    };

    let Ok(descriptor) = vize_atelier_sfc::parse_sfc(&content, options) else {
        return None;
    };
    let line_index = LineIndex::new(&content);

    let mut symbols = Vec::new();

    if let Some(ref template) = descriptor.template {
        let (range, selection_range) = block_ranges(&line_index, &template.loc, "template");
        symbols.push(DocumentSymbol {
            name: "template".to_string(),
            kind: SymbolKind::MODULE,
            tags: None,
            deprecated: None,
            range,
            selection_range,
            detail: template.lang.as_ref().map(|l| l.to_string()),
            children: None,
        });
    }

    if let Some(ref script) = descriptor.script {
        let (range, selection_range) = block_ranges(&line_index, &script.loc, "script");
        symbols.push(DocumentSymbol {
            name: "script".to_string(),
            kind: SymbolKind::MODULE,
            tags: None,
            deprecated: None,
            range,
            selection_range,
            detail: script.lang.as_ref().map(|l| l.to_string()),
            children: None,
        });
    }

    if let Some(ref script_setup) = descriptor.script_setup {
        let (range, selection_range) = block_ranges(&line_index, &script_setup.loc, "script");
        symbols.push(DocumentSymbol {
            name: "script setup".to_string(),
            kind: SymbolKind::MODULE,
            tags: None,
            deprecated: None,
            range,
            selection_range,
            detail: script_setup.lang.as_ref().map(|l| l.to_string()),
            children: None,
        });
    }

    for (i, style) in descriptor.styles.iter().enumerate() {
        let (range, selection_range) = block_ranges(&line_index, &style.loc, "style");
        #[allow(clippy::disallowed_macros)]
        let name = if let Some(ref module) = style.module {
            format!("style module={}", module)
        } else if style.scoped {
            "style scoped".to_string()
        } else {
            format!("style[{}]", i)
        };

        symbols.push(DocumentSymbol {
            name,
            kind: SymbolKind::MODULE,
            tags: None,
            deprecated: None,
            range,
            selection_range,
            detail: style.lang.as_ref().map(|l| l.to_string()),
            children: None,
        });
    }

    Some(DocumentSymbolResponse::Nested(symbols))
}
