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

pub(in super::super) fn join_verbatim_runs(
    source: &str,
    generated: &str,
    mut spans: Vec<ContentMapperSpan>,
) -> Vec<ContentMapperSpan> {
    if spans.len() < 2 {
        return spans;
    }

    // Keep the hot path allocation-free. The upper half of `kind` temporarily
    // holds the original-range group and its duplicate bit; the upper half of
    // `features` holds the generated-order index. Both are cleared before the
    // protocol leaves this function. Unusually large inputs keep their valid,
    // unjoined spans instead of truncating temporary metadata.
    const META_SHIFT: u32 = usize::BITS / 2;
    const VALUE_MASK: usize = usize::MAX >> META_SHIFT;
    const SHARED_ORIGINAL: usize = 1 << (META_SHIFT - 1);
    if spans.len() >= SHARED_ORIGINAL
        || spans
            .iter()
            .any(|span| span.0[4] > VALUE_MASK || span.0[5] > VALUE_MASK)
    {
        return spans;
    }

    for (index, span) in spans.iter_mut().enumerate() {
        span.0[5] |= index << META_SHIFT;
    }
    spans.sort_unstable_by_key(|span| (span.0[2], span.0[3], span.0[5] >> META_SHIFT));

    let mut group = 0;
    let mut group_start = 0;
    for index in 1..=spans.len() {
        let group_ended = index == spans.len()
            || (spans[index].0[2], spans[index].0[3])
                != (spans[group_start].0[2], spans[group_start].0[3]);
        if !group_ended {
            continue;
        }
        let duplicate = (index - group_start > 1).then_some(SHARED_ORIGINAL);
        let metadata = group | duplicate.unwrap_or_default();
        for span in &mut spans[group_start..index] {
            span.0[4] |= metadata << META_SHIFT;
        }
        group += 1;
        group_start = index;
    }

    spans.sort_unstable_by_key(|span| span.0[5] >> META_SHIFT);
    let mut written = 0;
    for read in 0..spans.len() {
        let span = spans[read];
        if written > 0
            && joinable(
                source,
                generated,
                &spans[written - 1],
                &span,
                VALUE_MASK,
                SHARED_ORIGINAL,
            )
        {
            let previous = &mut spans[written - 1];
            let end = span.0[0] + span.0[1];
            previous.0[1] = end - previous.0[0];
            previous.0[3] = previous.0[1];
            previous.0[4] = (previous.0[4] & VALUE_MASK) | (span.0[4] & !VALUE_MASK);
            continue;
        }
        spans[written] = span;
        written += 1;
    }
    spans.truncate(written);
    for span in &mut spans {
        span.0[4] &= VALUE_MASK;
        span.0[5] &= VALUE_MASK;
    }
    spans
}

fn joinable(
    source: &str,
    generated: &str,
    previous: &ContentMapperSpan,
    next: &ContentMapperSpan,
    value_mask: usize,
    shared_original: usize,
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
    let left_metadata = left_kind >> (usize::BITS / 2);
    let metadata = kind >> (usize::BITS / 2);
    let left_group = left_metadata & (shared_original - 1);
    let group = metadata & (shared_original - 1);
    let verbatim = ContentMapperSpanKind::Verbatim as usize;
    let (gap_start, original_gap_start) = (left_at + left_len, left_original + left_original_len);
    (left_kind & value_mask) == verbatim
        && (kind & value_mask) == verbatim
        && (left_features & value_mask) == (features & value_mask)
        && left_len == left_original_len
        && len == original_len
        && at >= gap_start
        && original >= original_gap_start
        && at - gap_start == original - original_gap_start
        && left_metadata & shared_original == 0
        && metadata & shared_original == 0
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
