//! Authored positions for a template expression whose comments were stripped.
//!
//! The emitted expression drops each comment, so a single whole-expression
//! mapping shifts every later byte into the comment; a multi-byte comment
//! moves it by a different number of UTF-16 columns again (#6430). The
//! identity runs between comments map back byte for byte instead.

use crate::virtual_ts::types::{VizeMapping, VizeSubSpan};
use vize_carton::append;
use vize_croquis::croquis::TemplateExpression;
use vize_croquis::drawer::strip_js_comments_with_offsets;

/// Sub-spans for `emitted`, the stripped text of authored `source` written at
/// `generated_start`. Empty when nothing was stripped or `emitted` is not a
/// verbatim slice of the stripped text.
pub(super) fn stripped_comment_sub_spans(
    source: &str,
    source_start: usize,
    emitted: &str,
    generated_start: usize,
) -> Vec<VizeSubSpan> {
    let (stripped, Some(offsets)) = strip_js_comments_with_offsets(source) else {
        return Vec::new();
    };
    let Some(offsets) = (!emitted.is_empty())
        .then(|| stripped.find(emitted))
        .flatten()
        .and_then(|at| offsets.get(at..at + emitted.len()))
    else {
        return Vec::new();
    };
    let mut spans = Vec::new();
    // (emitted index, first authored offset, last authored offset)
    let mut run: Option<(usize, u32, u32)> = None;
    let mut flush = |run: Option<(usize, u32, u32)>| {
        if let Some((index, first, last)) = run {
            let (first, last) = (first as usize, last as usize);
            spans.push(VizeSubSpan {
                gen_range: generated_start + index..generated_start + index + (last - first + 1),
                src_range: source_start + first..source_start + last + 1,
            });
        }
    };
    for (index, &offset) in offsets.iter().enumerate() {
        match run.as_mut() {
            Some((_, _, last)) if offset != u32::MAX && offset == *last + 1 => *last = offset,
            _ => {
                flush(run.take());
                run = (offset != u32::MAX).then_some((index, offset, offset));
            }
        }
    }
    flush(run);
    // A comment after the last code (`value // @ts-ignore`) moves nothing:
    // the whole-expression mapping is already exact.
    let shifted = spans
        .iter()
        .any(|span| span.gen_range.start - generated_start != span.src_range.start - source_start);
    if shifted { spans } else { Vec::new() }
}

/// The byte offset of a block comment that never closes, if any.
pub(super) fn unterminated_block_comment_start(source: &str) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        match byte {
            b'\'' | b'"' | b'`' => {
                index += 1;
                while let Some(&current) = bytes.get(index) {
                    index += 1;
                    if current == b'\\' {
                        index += 1;
                    } else if current == byte {
                        break;
                    }
                }
            }
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                index = source
                    .get(index..)
                    .and_then(|rest| rest.find('\n'))
                    .map_or(bytes.len(), |end| index + end);
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                let body = source.get(index + 2..).unwrap_or_default();
                let Some(end) = body.find("*/") else {
                    return Some(index);
                };
                index += 2 + end + 2;
            }
            _ => index += 1,
        }
    }
    None
}

/// Emit an expression that is only an unterminated block comment.
///
/// Stripping it leaves nothing, which silently accepted a template Vue cannot
/// compile. The comment is closed so the module still parses, and the syntax
/// error TypeScript reports at the empty `void ()` belongs to the comment's
/// opening `/*`.
pub(super) fn emit_unterminated_comment(
    ts: &mut crate::virtual_ts::TextBuf,
    mappings: &mut Vec<VizeMapping>,
    expr: &TemplateExpression,
    source_start: usize,
    indent: &str,
) {
    let source = expr.content.as_str();
    let Some(open) = unterminated_block_comment_start(source) else {
        return;
    };
    let comment = source.get(open..).unwrap_or_default().trim_end();
    append!(*ts, "{indent}void (");
    let generated_start = ts.len();
    ts.push_str(comment);
    let close = ts.len();
    ts.push_str("*/)");
    let generated_end = ts.len();
    append!(*ts, "; // {}\n", expr.kind.as_str());
    let authored_open = source_start + open;
    mappings.push(VizeMapping {
        gen_range: generated_start..generated_end,
        src_range: authored_open..authored_open + comment.len(),
        sub_spans: vec![VizeSubSpan {
            gen_range: close..generated_end,
            src_range: authored_open..authored_open,
        }],
    });
}
