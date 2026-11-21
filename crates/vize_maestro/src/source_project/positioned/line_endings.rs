//! Preserve selected bytes when an editor cannot address a CRLF interior.
use super::SnapshotKey;
use tower_lsp::lsp_types::{AnnotatedTextEdit, OneOf, Position, Range, TextEdit};
use vize_l0::line_index::{LineBreaks, utf16_len};
use vize_l1::edit::SpanEdit;

type Edit<'a, 'r> = SpanEdit<'a, 'r, SnapshotKey<'a>>;

#[inline]
fn inside_crlf(bytes: &[u8], at: usize) -> bool {
    bytes.get(at) == Some(&b'\n') && at != 0 && bytes[at - 1] == b'\r'
}

/// The existing LF positions agree with LSP through complete CRLF pairs.
/// Only selected endpoints and the prefix that can affect them are relevant.
#[inline]
pub(super) fn needs_projection(source: &str, edits: &[Edit<'_, '_>]) -> bool {
    let Some(last) = edits.last() else {
        return false;
    };
    let bytes = source.as_bytes();
    let end = last.span().end as usize;
    let mut returns = memchr::memchr_iter(b'\r', &bytes[..end]);
    let Some(first) = returns.next() else {
        return false;
    };
    let standalone = |at: usize| bytes.get(at + 1) != Some(&b'\n');
    standalone(first)
        || returns.any(standalone)
        || edits.iter().any(|edit| {
            let span = edit.span();
            inside_crlf(bytes, span.start as usize) || inside_crlf(bytes, span.end as usize)
        })
}

fn extent(source: &[u8], edit: Edit<'_, '_>) -> (usize, usize) {
    let span = edit.span();
    let start = span.start as usize;
    let end = span.end as usize;
    (
        start - usize::from(inside_crlf(source, start)),
        end + usize::from(inside_crlf(source, end)),
    )
}

/// Widen to a whole line ending. Adjacent edits whose widened ranges intersect
/// must be applied to one replacement in the original selection order.
pub(super) fn project(
    source: &str,
    mut edits: &[Edit<'_, '_>],
) -> Vec<OneOf<TextEdit, AnnotatedTextEdit>> {
    let mut output = Vec::with_capacity(edits.len());
    let mut starts = LineBreaks::Lsp.line_starts(source).skip(1).peekable();
    let (mut line, mut line_start, mut previous, mut column) = (0, 0, 0, 0);
    let mut position = |at| {
        while let Some(next) = starts.next_if(|next| *next <= at) {
            line += 1;
            line_start = next;
        }
        if line_start > previous {
            previous = line_start;
            column = 0;
        }
        column += utf16_len(&source[previous..at]) as u32;
        previous = at;
        Position::new(line, column)
    };
    while let Some(&first) = edits.first() {
        let (start, mut end) = extent(source.as_bytes(), first);
        let mut count = 1;
        while let Some(&next) = edits.get(count) {
            let (next_start, next_end) = extent(source.as_bytes(), next);
            if next_start >= end {
                break;
            }
            end = end.max(next_end);
            count += 1;
        }
        let (group, remaining) = edits.split_at(count);
        edits = remaining;
        let mut written = start;
        let length = group.iter().fold(end - start, |size, edit| {
            let span = edit.span();
            size - (span.end - span.start) as usize + edit.replacement().len()
        });
        let mut new_text = String::with_capacity(length);
        for edit in group {
            let span = edit.span();
            new_text.push_str(&source[written..span.start as usize]);
            new_text.push_str(edit.replacement());
            written = span.end as usize;
        }
        new_text.push_str(&source[written..end]);
        output.push(OneOf::Left(TextEdit {
            range: Range::new(position(start), position(end)),
            new_text,
        }));
    }
    output
}
