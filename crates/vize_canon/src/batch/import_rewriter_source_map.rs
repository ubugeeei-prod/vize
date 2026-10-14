//! Offset bookkeeping for rewritten module specifiers.

use vize_carton::String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OffsetAdjustment {
    pub original_offset: u32,
    pub adjustment: i32,
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
