//! `volar/client/autoInsert` custom JSON-RPC request.
#![allow(clippy::disallowed_types)]

use serde::Deserialize;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{Position, TextDocumentIdentifier};

use super::MaestroServer;
use crate::ide::{AutoInsertService, IdeContext, position_to_offset};

pub(super) const AUTO_INSERT_METHOD: &str = "volar/client/autoInsert";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AutoInsertParams {
    text_document: TextDocumentIdentifier,
    selection: Position,
    change: AutoInsertChange,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AutoInsertChange {
    range_offset: usize,
    range_length: usize,
    text: String,
}

impl MaestroServer {
    pub(super) async fn auto_insert(&self, params: AutoInsertParams) -> Result<Option<String>> {
        if !self.state.lsp_features().auto_insert || params.change.range_length != 0 {
            return Ok(None);
        }

        let uri = &params.text_document.uri;
        let Some(content) = self.state.documents.text(uri) else {
            return Ok(None);
        };
        let Some(selection_offset) =
            position_to_offset(&content, params.selection.line, params.selection.character)
        else {
            return Ok(None);
        };
        let Some(range_offset) = utf16_offset_to_byte(&content, params.change.range_offset) else {
            return Ok(None);
        };
        let change_matches_document = if params.change.text == "{}" {
            let (Some(caret), Some(change_end)) =
                (range_offset.checked_add(1), range_offset.checked_add(2))
            else {
                return Ok(None);
            };
            selection_offset == caret && content.get(range_offset..change_end) == Some("{}")
        } else {
            let Some(change_end) = range_offset.checked_add(params.change.text.len()) else {
                return Ok(None);
            };
            selection_offset == change_end
                && content
                    .get(range_offset..selection_offset)
                    .is_some_and(|inserted| inserted == params.change.text)
        };
        if !change_matches_document {
            return Ok(None);
        }

        let ctx = IdeContext::with_content(&self.state, uri, selection_offset, content);
        Ok(
            AutoInsertService::snippet(&ctx, selection_offset, range_offset, &params.change.text)
                .await,
        )
    }
}

/// Volar's `rangeOffset` is a JavaScript string offset (UTF-16 code units),
/// while Rust strings and Maestro internals use UTF-8 byte offsets.
fn utf16_offset_to_byte(content: &str, target: usize) -> Option<usize> {
    let mut utf16 = 0usize;
    for (byte, ch) in content.char_indices() {
        if utf16 == target {
            return Some(byte);
        }
        utf16 = utf16.checked_add(ch.len_utf16())?;
        if utf16 > target {
            return None;
        }
    }
    (utf16 == target).then_some(content.len())
}

#[cfg(test)]
mod tests {
    use super::utf16_offset_to_byte;

    #[test]
    fn utf16_offsets_reject_surrogate_splits_and_map_boundaries() {
        let source = "a😀éz";
        assert_eq!(utf16_offset_to_byte(source, 0), Some(0));
        assert_eq!(utf16_offset_to_byte(source, 1), Some(1));
        assert_eq!(utf16_offset_to_byte(source, 2), None);
        assert_eq!(utf16_offset_to_byte(source, 3), Some(5));
        assert_eq!(utf16_offset_to_byte(source, 4), Some(7));
        assert_eq!(utf16_offset_to_byte(source, 5), Some(8));
        assert_eq!(utf16_offset_to_byte(source, usize::MAX), None);
    }
}
