//! Select one expectation interval from emitter-owned diagnostic checks.

use super::ContentMapperSpan;
use std::ops::Range;

pub(super) fn expect_interval(
    generated: &str,
    spans: &[ContentMapperSpan],
    targets: &[Range<usize>],
    ranges: &[Range<usize>],
    sibling_starts: &[usize],
    primaries: &[Range<usize>],
) -> (usize, usize) {
    let Some((start, end)) = range_bounds(ranges) else {
        return (0, 0);
    };
    let mut cuts = foreign_diagnostic_cuts(generated, spans, targets, start, end);
    cuts.extend(
        sibling_starts
            .iter()
            .copied()
            .filter(|cut| start < *cut && *cut < end),
    );
    cuts.sort_unstable();
    cuts.dedup();
    if cuts.is_empty() {
        return (start, end);
    }
    let kept = ranges
        .iter()
        .filter(|range| !range_contains_cut(range, &cuts))
        .cloned()
        .collect::<Vec<_>>();
    let clusters = clusters_split_by_cuts(&kept, &cuts);
    clusters
        .into_iter()
        .max_by_key(|cluster| {
            let text = generated.get(cluster.start..cluster.end).unwrap_or("");
            let rank = if text.contains("__vize_prop_check_") {
                3_u8
            } else if text.contains("__vize_event_keys_") || text.contains("__vize_required_slots_")
            {
                2
            } else {
                1
            };
            // Prefer the actual emitted check over a differently wrapped
            // navigation copy. Generated punctuation has no owner identity.
            let owns_primary = primaries
                .iter()
                .any(|primary| cluster.start <= primary.start && primary.end <= cluster.end);
            (owns_primary, rank, cluster.end - cluster.start)
        })
        .map_or((0, 0), |cluster| (cluster.start, cluster.end))
}

fn range_bounds(ranges: &[Range<usize>]) -> Option<(usize, usize)> {
    Some((ranges.first()?.start, ranges.last()?.end))
}

fn range_contains_cut(range: &Range<usize>, cuts: &[usize]) -> bool {
    cuts.iter()
        .any(|cut| range.start < *cut && *cut < range.end)
}

fn clusters_split_by_cuts(ranges: &[Range<usize>], cuts: &[usize]) -> Vec<Range<usize>> {
    let mut clusters = Vec::new();
    for range in ranges {
        let separated = clusters.last().is_some_and(|last: &Range<usize>| {
            cuts.iter()
                .any(|cut| last.end <= *cut && *cut < range.start)
        });
        if separated || clusters.is_empty() {
            clusters.push(range.clone());
            continue;
        }
        if let Some(last) = clusters.last_mut() {
            last.end = last.end.max(range.end);
        }
    }
    clusters
}

fn foreign_diagnostic_cuts(
    generated: &str,
    spans: &[ContentMapperSpan],
    targets: &[Range<usize>],
    start: usize,
    end: usize,
) -> Vec<usize> {
    let mut cuts = spans
        .iter()
        .filter_map(|span| {
            let ContentMapperSpan([generated_at, length, original, original_length, _, _]) = *span;
            if length == 0 || generated_at < start || end <= generated_at {
                return None;
            }
            // A span mapped outside the owner belongs to another node. Unless
            // it only navigates, it is that node's check: never cover it.
            let overlaps_target = targets
                .iter()
                .any(|target| original < target.end && original + original_length > target.start);
            (!overlaps_target && !navigation_read(generated, generated_at)).then_some(generated_at)
        })
        .collect::<Vec<_>>();
    cuts.sort_unstable();
    cuts.dedup();
    cuts
}

/// Navigation reads resolve a tag (`void Tag;`) or a key on an open
/// `__vize_*nav*` record and report nothing.
fn navigation_read(generated: &str, start: usize) -> bool {
    let line_start = generated
        .get(..start)
        .and_then(|before| before.rfind('\n'))
        .map_or(0, |index| index + 1);
    let Some(rest) = generated.get(line_start..) else {
        return false;
    };
    let line = rest.split_once('\n').map_or(rest, |(line, _)| line).trim();
    let tag = |tag: &str| {
        !tag.is_empty()
            && tag
                .bytes()
                .all(|byte| matches!(byte, b'_' | b'$' | b'.') || byte.is_ascii_alphanumeric())
    };
    line.contains("_nav")
        || line
            .strip_prefix("void ")
            .and_then(|rest| rest.strip_suffix(';'))
            .is_some_and(tag)
}
