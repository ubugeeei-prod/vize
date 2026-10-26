//! Offset bookkeeping for rewritten module specifiers.

use vize_carton::String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OffsetAdjustment {
    pub original_offset: u32,
    pub adjustment: i32,
}

#[cfg(test)]
mod storage_tests {
    use super::{ImportSourceMap, ReplacementRange};

    #[test]
    fn normal_rewrites_use_one_presized_metadata_buffer_without_reallocation() {
        for count in [0, 1, 4, 5, 64, 256] {
            let mut map = ImportSourceMap::for_rewrites(count);
            let pointer = map.replacement_ranges.as_ptr();
            for index in (0..count).rev() {
                let start = index as u32 * 8 + 10;
                map.record_replacement(start, start + 2, 4);
                assert_eq!(map.replacement_ranges.as_ptr(), pointer);
            }
            map.finish_descending_replacements();
            assert_eq!(map.adjustments.capacity(), 0);
            assert_eq!(map.replacement_ranges.len(), count);
            assert_eq!(map.replacement_ranges.as_ptr(), pointer);
            eprintln!(
                "literal_mapping_storage rows={count} owned_buffers={} capacity={} bytes={}",
                usize::from(map.replacement_ranges.capacity() != 0),
                map.replacement_ranges.capacity(),
                map.replacement_ranges.capacity() * std::mem::size_of::<ReplacementRange>()
            );
        }
    }
}

#[derive(Debug)]
pub struct RewriteResult {
    pub code: String,
    pub source_map: ImportSourceMap,
}

#[derive(Default, Clone, PartialEq, Eq)]
pub struct ImportSourceMap {
    adjustments: Vec<OffsetAdjustment>,
    replacement_ranges: Vec<ReplacementRange>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReplacementRange {
    original_start: u32,
    original_end: u32,
    virtual_start: u32,
    virtual_end: u32,
}

impl std::fmt::Debug for ImportSourceMap {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = formatter.debug_struct("ImportSourceMap");
        output.field("adjustments", &self.adjustments);
        if !self.replacement_ranges.is_empty() {
            output.field("replacement_ranges", &self.replacement_ranges);
        }
        output.finish()
    }
}

impl ImportSourceMap {
    pub fn new(adjustments: Vec<OffsetAdjustment>) -> Self {
        Self {
            adjustments,
            replacement_ranges: Vec::new(),
        }
    }

    /// Normal module rewrites own one range buffer instead of also duplicating
    /// their end adjustments. Legacy adjustment-only callers remain supported.
    pub(super) fn for_rewrites(count: usize) -> Self {
        Self {
            adjustments: Vec::new(),
            replacement_ranges: Vec::with_capacity(count),
        }
    }

    pub(super) fn record_replacement(&mut self, start: u32, end: u32, length: u32) {
        self.replacement_ranges.push(ReplacementRange {
            original_start: start,
            original_end: end,
            virtual_start: start,
            virtual_end: start.saturating_add(length),
        });
    }

    /// The writer replaces literals right to left. Publish their actual virtual
    /// coordinates in source order, applying each shift at its exclusive end.
    pub(super) fn finish_descending_replacements(&mut self) {
        self.replacement_ranges.reverse();
        let mut delta = 0_i64;
        for range in &mut self.replacement_ranges {
            let length = range.virtual_end - range.virtual_start;
            range.virtual_start = translated_offset(range.original_start, delta);
            range.virtual_end = range.virtual_start.saturating_add(length);
            delta = i64::from(range.virtual_end) - i64::from(range.original_end);
        }
    }

    fn preceding_delta(&self, index: usize) -> i64 {
        index
            .checked_sub(1)
            .and_then(|index| self.replacement_ranges.get(index))
            .map_or(0, |range| {
                i64::from(range.virtual_end) - i64::from(range.original_end)
            })
    }

    fn original_offset_from_ranges(&self, offset: u32) -> u32 {
        let index = self
            .replacement_ranges
            .partition_point(|range| range.virtual_end <= offset);
        if let Some(range) = self.replacement_ranges.get(index)
            && offset >= range.virtual_start
        {
            return range.original_start;
        }
        translated_offset(offset, -self.preceding_delta(index))
    }

    fn virtual_offset_from_ranges(&self, offset: u32) -> u32 {
        let index = self
            .replacement_ranges
            .partition_point(|range| range.original_end <= offset);
        if let Some(range) = self.replacement_ranges.get(index)
            && offset >= range.original_start
        {
            return range.virtual_start;
        }
        translated_offset(offset, self.preceding_delta(index))
    }

    pub(super) fn with_replacement_ranges(
        adjustments: Vec<OffsetAdjustment>,
        replacements: Vec<(u32, u32, u32)>,
    ) -> Self {
        let mut map = Self::new(adjustments);
        map.replacement_ranges = replacements
            .into_iter()
            .map(|(original_start, original_end, new_len)| {
                let virtual_start = map.get_virtual_offset(original_start);
                ReplacementRange {
                    original_start,
                    original_end,
                    virtual_start,
                    virtual_end: virtual_start + new_len,
                }
            })
            .collect();
        map.replacement_ranges
            .sort_by_key(|range| range.original_start);
        map
    }

    /// Record star bytes replaced by text of the given virtual length.
    pub(super) fn with_added_star_replacements(&self, replacements: &[(u32, u32)]) -> Self {
        if replacements.is_empty() {
            return self.clone();
        }
        if self.adjustments.is_empty() {
            let mut map = Self::for_rewrites(self.replacement_ranges.len() + replacements.len());
            for range in &self.replacement_ranges {
                map.record_replacement(
                    range.original_start,
                    range.original_end,
                    range.virtual_end - range.virtual_start,
                );
            }
            for (start, length) in replacements {
                map.record_replacement(*start, start + 1, *length);
            }
            map.replacement_ranges
                .sort_by_key(|range| std::cmp::Reverse(range.original_start));
            map.finish_descending_replacements();
            return map;
        }
        let mut adjustments = self.adjustments.clone();
        adjustments.extend(replacements.iter().map(|(start, length)| OffsetAdjustment {
            original_offset: start + 1,
            adjustment: *length as i32 - 1,
        }));
        adjustments.sort_by_key(|adjustment| adjustment.original_offset);
        let mut ranges: Vec<_> = self
            .replacement_ranges
            .iter()
            .map(|range| {
                (
                    range.original_start,
                    range.original_end,
                    range.virtual_end - range.virtual_start,
                )
            })
            .collect();
        ranges.extend(
            replacements
                .iter()
                .map(|(start, length)| (*start, start + 1, *length)),
        );
        Self::with_replacement_ranges(adjustments, ranges)
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn get_original_offset(&self, virtual_offset: u32) -> u32 {
        if self.adjustments.is_empty() {
            if self.replacement_ranges.is_empty() {
                return virtual_offset;
            }
            return self.original_offset_from_ranges(virtual_offset);
        }
        if let Some(range) = self.replacement_ranges.iter().find(|range| {
            range.virtual_start <= virtual_offset && virtual_offset < range.virtual_end
        }) {
            return range.original_start;
        }
        let mut cumulative: i32 = 0;
        for adj in &self.adjustments {
            let adjusted = (adj.original_offset as i32 + cumulative) as u32;
            if virtual_offset < adjusted {
                break;
            }
            cumulative += adj.adjustment;
        }
        (virtual_offset as i32 - cumulative) as u32
    }

    pub fn get_virtual_offset(&self, original_offset: u32) -> u32 {
        if self.adjustments.is_empty() {
            if self.replacement_ranges.is_empty() {
                return original_offset;
            }
            return self.virtual_offset_from_ranges(original_offset);
        }
        if let Some(range) = self.replacement_ranges.iter().find(|range| {
            range.original_start <= original_offset && original_offset < range.original_end
        }) {
            return range.virtual_start;
        }
        let mut cumulative: i32 = 0;
        for adj in &self.adjustments {
            if original_offset < adj.original_offset {
                break;
            }
            cumulative += adj.adjustment;
        }
        (original_offset as i32 + cumulative) as u32
    }
}

/// Real document coordinates fit OXC's u32 spans. An out-of-document caller
/// must not panic or wrap an overflowing query onto an unrelated source byte.
fn translated_offset(offset: u32, delta: i64) -> u32 {
    (i64::from(offset) + delta).clamp(0, i64::from(u32::MAX)) as u32
}
