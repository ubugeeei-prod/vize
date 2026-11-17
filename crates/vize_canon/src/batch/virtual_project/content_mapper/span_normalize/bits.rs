//! Accepted ranges as bits, for ranges that are pairwise disjoint or equal.

use std::ops::Range;

/// One bit per offset for the bytes accepted ranges cover, the offsets they
/// start at, and the offsets of empty ranges. Ranges are pairwise disjoint or
/// equal, so a covered bit inside a probe is an overlap, and an empty probe
/// overlaps exactly when its offset is covered but starts no range: a scan of
/// the probe's own bits, with no tree walk and no per-range allocation.
pub(super) struct Bits {
    covered: Vec<u64>,
    starts: Vec<u64>,
    points: Vec<u64>,
}

impl Bits {
    /// Room for offsets `0..=len`.
    pub(super) fn new(len: usize) -> Self {
        let words = len / 64 + 1;
        Self {
            covered: vec![0; words],
            starts: vec![0; words],
            points: vec![0; words],
        }
    }

    pub(super) fn insert(&mut self, range: &Range<usize>) {
        if range.is_empty() {
            set(&mut self.points, range.start);
        } else {
            set(&mut self.starts, range.start);
            let mut at = range.start;
            while at < range.end {
                let (word, bit) = (at / 64, at % 64);
                let span = (range.end - at).min(64 - bit);
                if let Some(slot) = self.covered.get_mut(word) {
                    *slot |= mask(bit, span);
                }
                at += span;
            }
        }
    }

    /// Whether `probe` overlaps an inserted range, as `ranges_overlap` decides:
    /// an empty probe overlaps a range strictly containing its offset, and an
    /// empty range one strictly inside the probe.
    pub(super) fn overlaps(&self, probe: &Range<usize>) -> bool {
        if probe.is_empty() {
            get(&self.covered, probe.start) && !get(&self.starts, probe.start)
        } else {
            any(&self.covered, probe.start, probe.end)
                || any(&self.points, probe.start + 1, probe.end)
        }
    }
}

fn mask(bit: usize, span: usize) -> u64 {
    let ones = if span >= 64 { u64::MAX } else { (1 << span) - 1 };
    ones << bit
}

fn set(words: &mut [u64], at: usize) {
    if let Some(word) = words.get_mut(at / 64) {
        *word |= 1 << (at % 64);
    }
}

fn get(words: &[u64], at: usize) -> bool {
    words
        .get(at / 64)
        .is_some_and(|word| word & (1 << (at % 64)) != 0)
}

/// Any bit set in `start..end`.
fn any(words: &[u64], start: usize, end: usize) -> bool {
    let mut at = start;
    while at < end {
        let (word, bit) = (at / 64, at % 64);
        let span = (end - at).min(64 - bit);
        if words.get(word).is_some_and(|value| value & mask(bit, span) != 0) {
            return true;
        }
        at += span;
    }
    false
}
