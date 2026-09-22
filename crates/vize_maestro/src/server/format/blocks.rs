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
    // The formatted buffer is not the open document, so this must not write
    // the resident document cache. `parse_descriptor` is that parse.
    let descriptor = vize_resident::parse_descriptor(filename, source)?;

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
