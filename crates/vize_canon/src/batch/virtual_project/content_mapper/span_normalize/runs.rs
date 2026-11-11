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

const KIND_BITS: u32 = 2;
const KIND_MASK: usize = (1 << KIND_BITS) - 1;
const SHARED_ORIGINAL: usize = 1;

pub(in super::super) fn join_verbatim_runs(
    source: &str,
    generated: &str,
    mut spans: Vec<ContentMapperSpan>,
) -> Vec<ContentMapperSpan> {
    if spans.len() < 2 {
        return spans;
    }

    // Generated starts are normally strictly increasing. That lets the same
    // allocation serve as both the original-range index and the output: sort
    // by authored range, annotate the finite kind field, then restore the
    // exact input order by generated start. Feature masks stay untouched on
    // every target, including wasm32. If an unusual producer does not provide
    // that order, retain its valid spans without joining them.
    let metadata_fits = spans
        .len()
        .checked_mul(2)
        .and_then(|value| value.checked_add(SHARED_ORIGINAL))
        .and_then(|value| value.checked_shl(KIND_BITS))
        .is_some();
    if !metadata_fits
        || spans
            .iter()
            .any(|span| span.0[4] > ContentMapperSpanKind::Alias as usize)
        || !spans.windows(2).all(|pair| {
            let [left, right] = pair else {
                return false;
            };
            left.0[0] < right.0[0]
        })
    {
        return spans;
    }

    spans.sort_unstable_by_key(|span| (span.0[2], span.0[3], span.0[0]));
    for (group, original_group) in spans
        .chunk_by_mut(|left, right| (left.0[2], left.0[3]) == (right.0[2], right.0[3]))
        .enumerate()
    {
        let shared_original = usize::from(original_group.len() > 1);
        let metadata = ((group * 2) | shared_original) << KIND_BITS;
        for span in original_group {
            span.0[4] |= metadata;
        }
    }

    spans.sort_unstable_by_key(|span| span.0[0]);
    spans.dedup_by(|next, previous| {
        let joined = joinable(source, generated, previous, next);
        if joined {
            let end = next.0[0] + next.0[1];
            previous.0[1] = end - previous.0[0];
            previous.0[3] = previous.0[1];
            previous.0[4] = (previous.0[4] & KIND_MASK) | (next.0[4] & !KIND_MASK);
        }
        joined
    });
    for span in &mut spans {
        span.0[4] &= KIND_MASK;
    }
    spans
}

fn joinable(
    source: &str,
    generated: &str,
    previous: &ContentMapperSpan,
    next: &ContentMapperSpan,
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
    let left_metadata = left_kind >> KIND_BITS;
    let metadata = kind >> KIND_BITS;
    let left_group = left_metadata >> 1;
    let group = metadata >> 1;
    let verbatim = ContentMapperSpanKind::Verbatim as usize;
    let (gap_start, original_gap_start) = (left_at + left_len, left_original + left_original_len);
    (left_kind & KIND_MASK) == verbatim
        && (kind & KIND_MASK) == verbatim
        && left_features == features
        && left_len == left_original_len
        && len == original_len
        && at >= gap_start
        && original >= original_gap_start
        && at - gap_start == original - original_gap_start
        && left_metadata & SHARED_ORIGINAL == 0
        && metadata & SHARED_ORIGINAL == 0
        && group <= left_group.saturating_add(1)
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
