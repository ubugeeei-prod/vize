//! Pairing an authored SFC's blocks with the formatted document's blocks.
//!
//! Both `rangeFormatting` and `onTypeFormatting` answer by formatting the whole
//! document once and then projecting the result back onto the part of the file
//! the request is about. That projection needs the same blocks on both sides,
//! which is what this module provides: content spans in one fixed discovery
//! order, so `authored[i]` and `formatted[i]` are the same block.

/// Byte span of one block's content — from just after the open tag to just
/// before the close tag — in discovery order.
pub(super) type BlockSpan = (usize, usize);

pub(super) fn block_spans(source: &str, filename: &str) -> Option<Vec<BlockSpan>> {
    let options = vize_atelier_sfc::SfcParseOptions {
        filename: filename.into(),
        ..Default::default()
    };
    let descriptor = vize_atelier_sfc::parse_sfc(source, options).ok()?;

    Some(
        descriptor
            .template
            .as_ref()
            .map(|block| (block.loc.start, block.loc.end))
            .into_iter()
            .chain(
                descriptor
                    .script_setup
                    .as_ref()
                    .map(|block| (block.loc.start, block.loc.end)),
            )
            .chain(
                descriptor
                    .script
                    .as_ref()
                    .map(|block| (block.loc.start, block.loc.end)),
            )
            .chain(
                descriptor
                    .styles
                    .iter()
                    .map(|block| (block.loc.start, block.loc.end)),
            )
            .collect(),
    )
}
