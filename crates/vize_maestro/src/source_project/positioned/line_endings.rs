//! Preserve selected bytes when an editor cannot address a CRLF interior.
use super::SnapshotKey;
use tower_lsp::lsp_types::{AnnotatedTextEdit, OneOf, Position, Range, TextEdit};
use vize_l0::line_index::{LineBreaks, utf16_len};
use vize_l1::edit::SpanEdit;

type Edit<'a, 'r> = SpanEdit<'a, 'r, SnapshotKey<'a>>;

#[inline]
fn inside_crlf(bytes: &[u8], at: usize) -> bool {
    bytes.get(at) == Some(&b'\n') && bytes.get(..at).and_then(<[u8]>::last) == Some(&b'\r')
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
    let (prefix, _) = bytes.split_at(end);
    let mut returns = memchr::memchr_iter(b'\r', prefix);
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

// Spans were checked on this exact buffer. Widening can only add ASCII CR or LF.
fn between(source: &str, start: usize, end: usize) -> &str {
    let (prefix, _) = source.split_at(end);
    prefix.split_at(start).1
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
        column += utf16_len(between(source, previous, at)) as u32;
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
        let (removed, inserted) = group.iter().fold((0, 0), |(removed, inserted), edit| {
            let span = edit.span();
            (
                removed + (span.end - span.start) as usize,
                inserted + edit.replacement().len(),
            )
        });
        let mut wire = TextEdit::new(
            Range::new(position(start), position(end)),
            Default::default(),
        );
        wire.new_text
            .reserve_exact(end - start - removed + inserted);
        for edit in group {
            let span = edit.span();
            wire.new_text
                .push_str(between(source, written, span.start as usize));
            wire.new_text.push_str(edit.replacement());
            written = span.end as usize;
        }
        wire.new_text.push_str(between(source, written, end));
        output.push(OneOf::Left(wire));
    }
    output
}
