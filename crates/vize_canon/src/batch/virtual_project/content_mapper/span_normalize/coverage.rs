//! Accepted ranges of one coordinate space as bitmaps over its offsets.
//!
//! Distinct non-empty accepted ranges are disjoint (see `DisjointIndex`), so a
//! bit per offset says whether some range covers it and another whether one
//! starts there. Probing and inserting touch only the probe's own words, where
//! an ordered map paid a node search and an allocation per range.

use std::ops::Range;

use super::RangeIndex;

const WORD: usize = u64::BITS as usize;

#[derive(Default)]
pub(super) struct CoverageIndex {
    /// Offsets inside a non-empty accepted range.
    covered: Vec<u64>,
    /// Offsets where a non-empty accepted range starts.
    starts: Vec<u64>,
    /// Offsets of accepted empty ranges.
    empties: Vec<u64>,
    /// Each start's end, kept only where ranges are collected.
    ends: Vec<u32>,
}

impl CoverageIndex {
    fn new(bound: usize, keep_ends: bool) -> Self {
        let words = bound.div_ceil(WORD) + 1;
        Self {
            covered: vec![0; words],
            starts: vec![0; words],
            empties: vec![0; words],
            ends: if keep_ends {
                vec![0; bound + 1]
            } else {
                Vec::new()
            },
        }
    }

    fn range_containing(&self, at: usize) -> Option<Range<usize>> {
        if !bit(&self.covered, at) {
            return None;
        }
        let start = last_set_at_or_before(&self.starts, at)?;
        let end = self.ends.get(start).copied()? as usize;
        Some(start..end)
    }
}

impl RangeIndex for CoverageIndex {
    fn for_generated(bound: usize) -> Self {
        Self::new(bound, false)
    }

    fn for_original(bound: usize) -> Self {
        Self::new(bound, true)
    }

    fn insert(&mut self, range: &Range<usize>) {
        if range.is_empty() {
            set(&mut self.empties, range.start);
            return;
        }
        set(&mut self.starts, range.start);
        set_range(&mut self.covered, range);
        if let Some(end) = self.ends.get_mut(range.start) {
            *end = u32::try_from(range.end).unwrap_or(u32::MAX);
        }
    }

    fn overlaps(&self, probe: &Range<usize>) -> bool {
        let strictly_inside = probe.start.saturating_add(1)..probe.end;
        let covered = if probe.is_empty() {
            bit(&self.covered, probe.start) && !bit(&self.starts, probe.start)
        } else {
            any_set(&self.covered, probe)
        };
        covered || any_set(&self.empties, &strictly_inside)
    }

    fn collect_overlapping(&self, probe: &Range<usize>, out: &mut Vec<Range<usize>>) {
        // The range around the probe's start, then every range starting inside.
        if let Some(range) = self.range_containing(probe.start)
            && (range.start < probe.start || !probe.is_empty())
        {
            out.push(range);
        }
        let inside = probe.start.saturating_add(1)..probe.end;
        for_each_set(&self.starts, &inside, |start| {
            if let Some(&end) = self.ends.get(start) {
                out.push(start..end as usize);
            }
        });
        for_each_set(&self.empties, &inside, |at| out.push(at..at));
    }
}

fn bit(words: &[u64], at: usize) -> bool {
    words
        .get(at / WORD)
        .is_some_and(|word| (word >> (at % WORD)) & 1 == 1)
}

fn set(words: &mut [u64], at: usize) {
    if let Some(word) = words.get_mut(at / WORD) {
        *word |= 1 << (at % WORD);
    }
}

/// The mask of bits `from..to` of one word, `to` at most [`WORD`].
fn mask(from: usize, to: usize) -> u64 {
    let high = if to >= WORD { u64::MAX } else { (1 << to) - 1 };
    high & !((1_u64 << from) - 1)
}

/// The words a range touches, with each word's mask of the range's bits.
fn masked_words(range: &Range<usize>) -> impl Iterator<Item = (usize, u64)> {
    let first = range.start / WORD;
    let last = range.end.saturating_sub(1) / WORD;
    let (start, end) = (range.start, range.end);
    (first..=last)
        .filter(move |_| start < end)
        .map(move |index| {
            let from = start.saturating_sub(index * WORD).min(WORD);
            let to = (end - index * WORD).min(WORD);
            (index, mask(from, to))
        })
}

fn set_range(words: &mut [u64], range: &Range<usize>) {
    for (index, bits) in masked_words(range) {
        if let Some(word) = words.get_mut(index) {
            *word |= bits;
        }
    }
}

fn any_set(words: &[u64], range: &Range<usize>) -> bool {
    masked_words(range).any(|(index, bits)| words.get(index).is_some_and(|word| word & bits != 0))
}

fn for_each_set(words: &[u64], range: &Range<usize>, mut each: impl FnMut(usize)) {
    for (index, bits) in masked_words(range) {
        let mut found = words.get(index).map_or(0, |word| word & bits);
        while found != 0 {
            each(index * WORD + found.trailing_zeros() as usize);
            found &= found - 1;
        }
    }
}

fn last_set_at_or_before(words: &[u64], at: usize) -> Option<usize> {
    let mut index = at / WORD;
    let mut found = words.get(index)? & mask(0, at % WORD + 1);
    loop {
        if found != 0 {
            return Some(index * WORD + (WORD - 1 - found.leading_zeros() as usize));
        }
        index = index.checked_sub(1)?;
        found = *words.get(index)?;
    }
}

#[cfg(test)]
mod tests {
    use super::{CoverageIndex, RangeIndex};
    use crate::batch::virtual_project::content_mapper::span_normalize::ranges_overlap;

    /// Disjoint (or repeated) ranges and empties across word boundaries,
    /// probed against a linear scan of the same ranges.
    #[test]
    fn coverage_answers_like_a_scan_of_disjoint_ranges() {
        let mut state = 0x2545_f491_4f6c_dd1d_u64;
        let mut next = |bound: u64| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state % bound) as usize
        };
        for round in 0..400 {
            let bound = [70, 130, 700, 4100][round % 4];
            let mut ranges: Vec<std::ops::Range<usize>> = Vec::new();
            for _ in 0..next(60) {
                let start = next(bound as u64);
                let length = [0, 1 + next(5), 1 + next(200)][next(3)];
                let range = start..(start + length).min(bound);
                let keeps_disjoint = ranges.iter().all(|other| {
                    *other == range
                        || range.is_empty()
                        || other.is_empty()
                        || !ranges_overlap(other, &range)
                });
                if keeps_disjoint {
                    ranges.push(range);
                }
            }
            let mut generated = CoverageIndex::for_generated(bound);
            let mut original = CoverageIndex::for_original(bound);
            for range in &ranges {
                generated.insert(range);
                original.insert(range);
            }
            for _ in 0..40 {
                let start = next(bound as u64 + 1);
                let probe = start..(start + next(90)).min(bound);
                let mut expected = ranges
                    .iter()
                    .filter(|range| ranges_overlap(range, &probe))
                    .cloned()
                    .collect::<Vec<_>>();
                expected.sort_by_key(|range| (range.start, range.end));
                expected.dedup();
                assert_eq!(
                    generated.overlaps(&probe),
                    !expected.is_empty(),
                    "round {round}, probe {probe:?}, ranges {ranges:?}"
                );
                let mut found = Vec::new();
                original.collect_overlapping(&probe, &mut found);
                found.sort_by_key(|range| (range.start, range.end));
                assert_eq!(found, expected, "round {round}, probe {probe:?}");
            }
        }
    }
}
