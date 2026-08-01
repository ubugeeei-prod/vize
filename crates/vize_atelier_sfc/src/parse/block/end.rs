//! Raw style and custom-block boundary scanning.

use std::borrow::Cow;

use memchr::memchr;

use super::{
    BlockEndSearch, BlockParseResult, advance_line, build_malformed_error, content_end_column,
    find_closing_tag_end,
};

/// Find the end of a raw block by jumping between `<` bytes.
pub(super) fn find_block_end<'a>(search: BlockEndSearch<'a>) -> BlockParseResult<'a> {
    let BlockEndSearch {
        bytes,
        source,
        tag_name,
        mut pos,
        content_start,
        start_line,
        start_column,
        initial_last_newline,
        attrs,
    } = search;
    let len = bytes.len();
    let mut line = start_line;
    let mut last_newline = initial_last_newline;

    while pos < len {
        if let Some(lt_offset) = memchr(b'<', &bytes[pos..]) {
            advance_line(
                &bytes[pos..pos + lt_offset],
                pos,
                &mut line,
                &mut last_newline,
            );
            pos += lt_offset;

            if bytes[pos] == b'<'
                && let Some(end_tag_pos) = find_closing_tag_end(bytes, pos, len, tag_name)
            {
                let content_end = pos;
                let column = content_end_column(
                    content_start,
                    start_line,
                    start_column,
                    content_end,
                    line,
                    last_newline,
                );
                let content = Cow::Borrowed(&source[content_start..content_end]);
                return Ok(Some((
                    tag_name,
                    attrs,
                    content,
                    content_start,
                    content_end,
                    end_tag_pos,
                    line,
                    column,
                )));
            }
            pos += 1;
        } else {
            break;
        }
    }

    Err(build_malformed_error(
        tag_name,
        "the closing tag is missing",
    ))
}
