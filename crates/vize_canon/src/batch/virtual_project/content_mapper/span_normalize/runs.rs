//! Join verbatim lines across the line breaks that the generated and authored
//! text share.
//!
//! Copied setup lines map one span per line, and every newline between them
//! becomes an unmapped-gap ignore. When both sides hold the same line break
//! and indentation, one span maps the whole run at the same offsets, so no
//! position moves. Only a gap that holds a `\n` joins: pieces within one line
//! (a name and its delimiters) keep their exact spans. A span whose original
//! another span also maps, or a gap that another span maps into, stays split:
//! originals must remain disjoint or equal.

use super::super::{ContentMapperSpan, ContentMapperSpanKind};

#[derive(Clone, Copy)]
struct JoinMetadata {
    input_index: usize,
    original_at: usize,
    original_len: usize,
    group: usize,
    shared_original: bool,
}

pub(in super::super) fn join_verbatim_runs(
    source: &str,
    generated: &str,
    mut spans: Vec<ContentMapperSpan>,
) -> Vec<ContentMapperSpan> {
    if spans.len() < 2 {
        return spans;
    }

    // Keep temporary ownership data outside the public mapping protocol. In
    // particular, feature masks may use more than half of a wasm32 `usize`.
    let mut metadata: Vec<_> = spans
        .iter()
        .enumerate()
        .map(|(input_index, span)| JoinMetadata {
            input_index,
            original_at: span.0[2],
            original_len: span.0[3],
            group: 0,
            shared_original: false,
        })
        .collect();
    metadata
        .sort_unstable_by_key(|entry| (entry.original_at, entry.original_len, entry.input_index));

    for (group, original_group) in metadata
        .chunk_by_mut(|left, right| {
            (left.original_at, left.original_len) == (right.original_at, right.original_len)
        })
        .enumerate()
    {
        let shared_original = original_group.len() > 1;
        for entry in original_group {
            entry.group = group;
            entry.shared_original = shared_original;
        }
    }

    metadata.sort_unstable_by_key(|entry| entry.input_index);
    let mut metadata = metadata.into_iter();
    let Some(mut previous_metadata) = metadata.next() else {
        return spans;
    };
    spans.dedup_by(|next, previous| {
        let Some(next_metadata) = metadata.next() else {
            return false;
        };
        let joined = joinable(
            source,
            generated,
            previous,
            next,
            (previous_metadata.group, previous_metadata.shared_original),
            (next_metadata.group, next_metadata.shared_original),
        );
        previous_metadata = next_metadata;
        if joined {
            let end = next.0[0] + next.0[1];
            previous.0[1] = end - previous.0[0];
            previous.0[3] = previous.0[1];
        }
        joined
    });
    spans
}

fn joinable(
    source: &str,
    generated: &str,
    previous: &ContentMapperSpan,
    next: &ContentMapperSpan,
    previous_metadata: (usize, bool),
    next_metadata: (usize, bool),
) -> bool {
    let [
        left_at,
        left_len,
        left_original,
        left_original_len,
        left_kind,
        left_features,
    ] = previous.0;
    let [at, len, original, original_len, kind, features] = next.0;
    let (left_group, left_shared_original) = previous_metadata;
    let (group, shared_original) = next_metadata;
    let verbatim = ContentMapperSpanKind::Verbatim as usize;
    let (gap_start, original_gap_start) = (left_at + left_len, left_original + left_original_len);
    left_kind == verbatim
        && kind == verbatim
        && left_features == features
        && left_len == left_original_len
        && len == original_len
        && at >= gap_start
        && original >= original_gap_start
        && at - gap_start == original - original_gap_start
        && !left_shared_original
        && !shared_original
        && group <= left_group + 1
        && generated
            .get(gap_start..at)
            .zip(source.get(original_gap_start..original))
            .is_some_and(|(gap, original_gap)| {
                gap == original_gap
                    && gap.contains('\n')
                    && gap.bytes().all(|byte| matches!(byte, b' ' | b'\t' | b'\n'))
            })
}

#[cfg(test)]
#[expect(clippy::string_slice, reason = "tests assert by panicking")]
#[expect(clippy::panic, reason = "fixture assertions fail by panicking")]
mod tests;
