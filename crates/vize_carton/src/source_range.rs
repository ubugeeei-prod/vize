//! Source range utilities for position tracking.
//!
//! Provides types for representing byte ranges in source code,
//! used for source mapping between Vue SFC and generated code.

use serde::{Deserialize, Serialize};

use crate::CompactString as String;

/// A range of byte offsets in a source file.
///
/// Used for tracking positions in source code for:
/// - Error reporting
/// - Source mapping between original and generated code
/// - IDE features (hover, goto definition, etc.)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceRange {
    /// Start byte offset (inclusive)
    pub start: u32,
    /// End byte offset (exclusive)
    pub end: u32,
}

impl SourceRange {
    /// Create a new source range.
    #[inline]
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    /// Create a range for a single position.
    #[inline]
    pub const fn point(offset: u32) -> Self {
        Self {
            start: offset,
            end: offset,
        }
    }

    /// Check if this range contains the given offset.
    #[inline]
    pub const fn contains(&self, offset: u32) -> bool {
        offset >= self.start && offset < self.end
    }

    /// Get the length of this range.
    #[inline]
    pub const fn len(&self) -> u32 {
        self.end.saturating_sub(self.start)
    }

    /// Check if this range is empty.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Check if this range intersects with another range.
    #[inline]
    pub const fn intersects(&self, other: &Self) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// Get the union of two ranges (smallest range containing both).
    #[inline]
    pub fn union(&self, other: &Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    /// Get the intersection of two ranges (largest range contained in both).
    #[inline]
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        let start = self.start.max(other.start);
        let end = self.end.min(other.end);
        if start < end {
            Some(Self { start, end })
        } else {
            None
        }
    }

    /// Offset this range by a given amount.
    #[inline]
    pub const fn offset(&self, amount: i32) -> Self {
        Self {
            start: offset_u32(self.start, amount),
            end: offset_u32(self.end, amount),
        }
    }

    /// Extend this range by a given amount on both ends.
    #[inline]
    pub const fn extend(&self, amount: u32) -> Self {
        Self {
            start: self.start.saturating_sub(amount),
            end: self.end.saturating_add(amount),
        }
    }
}

#[inline]
const fn offset_u32(value: u32, amount: i32) -> u32 {
    if amount >= 0 {
        value.saturating_add(amount as u32)
    } else {
        value.saturating_sub(amount.unsigned_abs())
    }
}

impl std::fmt::Display for SourceRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

/// A single source mapping entry.
///
/// Maps a range in the original source to a range in the generated code.
#[derive(Debug, Clone, Default)]
pub struct SourceMapping {
    /// Range in the original source
    pub source: SourceRange,
    /// Range in the generated code
    pub generated: SourceRange,
    /// Optional metadata about this mapping
    pub data: Option<MappingData>,
}

impl SourceMapping {
    /// Create a new mapping.
    #[inline]
    pub fn new(source: SourceRange, generated: SourceRange) -> Self {
        Self {
            source,
            generated,
            data: None,
        }
    }

    /// Create a mapping with metadata.
    #[inline]
    pub fn with_data(source: SourceRange, generated: SourceRange, data: MappingData) -> Self {
        Self {
            source,
            generated,
            data: Some(data),
        }
    }

    /// Check if this mapping contains the source offset.
    #[inline]
    pub fn contains_source(&self, offset: u32) -> bool {
        self.source.contains(offset)
    }

    /// Check if this mapping contains the generated offset.
    #[inline]
    pub fn contains_generated(&self, offset: u32) -> bool {
        self.generated.contains(offset)
    }

    /// Map a source offset to generated offset.
    pub fn source_to_generated(&self, source_offset: u32) -> Option<u32> {
        if self.source.contains(source_offset) {
            let relative = source_offset - self.source.start;
            let gen_offset =
                self.generated.start + relative.min(self.generated.len().saturating_sub(1));
            Some(gen_offset)
        } else {
            None
        }
    }

    /// Map a generated offset to source offset.
    pub fn generated_to_source(&self, gen_offset: u32) -> Option<u32> {
        if self.generated.contains(gen_offset) {
            let relative = gen_offset - self.generated.start;
            let src_offset = self.source.start + relative.min(self.source.len().saturating_sub(1));
            Some(src_offset)
        } else {
            None
        }
    }
}

/// Metadata associated with a mapping.
#[derive(Debug, Clone)]
pub enum MappingData {
    /// Expression (e.g., {{ expr }})
    Expression { text: String },
    /// Directive expression (e.g., v-if="expr")
    Directive { name: String, expr: String },
    /// Event handler (e.g., @click="handler")
    Event { name: String, handler: String },
    /// Binding (e.g., :prop="value")
    Binding { prop: String, value: String },
    /// Component reference
    Component { name: String },
    /// Slot binding
    Slot { name: String },
    /// Import statement
    Import { source: String, specifier: String },
}

/// Bidirectional source map.
///
/// Maintains mappings between original source and generated code,
/// supporting efficient lookup in both directions.
#[derive(Debug, Clone, Default)]
pub struct SourceMap {
    /// Mappings sorted by source offset
    mappings: Vec<SourceMapping>,
    /// Block offset in the original file (for nested blocks)
    pub block_offset: u32,
}

impl SourceMap {
    /// Create an empty source map.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create from a list of mappings.
    pub fn from_mappings(mut mappings: Vec<SourceMapping>) -> Self {
        mappings.sort_by_key(|m| m.source.start);
        Self {
            mappings,
            block_offset: 0,
        }
    }

    /// Set the block offset.
    #[inline]
    pub fn set_block_offset(&mut self, offset: u32) {
        self.block_offset = offset;
    }

    /// Add a mapping.
    pub fn add(&mut self, mapping: SourceMapping) {
        self.mappings.push(mapping);
        self.mappings.sort_by_key(|m| m.source.start);
    }

    /// Add a simple mapping without metadata.
    pub fn add_simple(&mut self, source_start: u32, source_end: u32, gen_start: u32, gen_end: u32) {
        self.add(SourceMapping::new(
            SourceRange::new(source_start, source_end),
            SourceRange::new(gen_start, gen_end),
        ));
    }

    /// Get all mappings.
    #[inline]
    pub fn mappings(&self) -> &[SourceMapping] {
        &self.mappings
    }

    /// Map source offset to generated offset.
    pub fn to_generated(&self, source_offset: u32) -> Option<u32> {
        let idx = self
            .mappings
            .binary_search_by(|m| {
                if m.source.end <= source_offset {
                    std::cmp::Ordering::Less
                } else if m.source.start > source_offset {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .ok()?;

        self.mappings.get(idx)?.source_to_generated(source_offset)
    }

    /// Map generated offset to source offset.
    pub fn to_source(&self, gen_offset: u32) -> Option<u32> {
        for mapping in &self.mappings {
            if let Some(src) = mapping.generated_to_source(gen_offset) {
                return Some(src + self.block_offset);
            }
        }
        None
    }

    /// Find mappings containing the source offset.
    pub fn find_by_source(&self, offset: u32) -> Vec<&SourceMapping> {
        self.mappings
            .iter()
            .filter(|m| m.contains_source(offset))
            .collect()
    }

    /// Find mappings containing the generated offset.
    pub fn find_by_generated(&self, offset: u32) -> Vec<&SourceMapping> {
        self.mappings
            .iter()
            .filter(|m| m.contains_generated(offset))
            .collect()
    }

    /// Check if the map is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
    }

    /// Get the number of mappings.
    #[inline]
    pub fn len(&self) -> usize {
        self.mappings.len()
    }
}

#[cfg(test)]
mod tests {
    use super::{SourceMap, SourceMapping, SourceRange};

    #[test]
    fn test_source_range_contains() {
        let range = SourceRange::new(10, 20);
        assert!(!range.contains(9));
        assert!(range.contains(10));
        assert!(range.contains(15));
        assert!(range.contains(19));
        assert!(!range.contains(20));
    }

    #[test]
    fn test_source_range_intersects() {
        let a = SourceRange::new(10, 20);
        let b = SourceRange::new(15, 25);
        let c = SourceRange::new(25, 30);

        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
        assert!(!a.intersects(&c));
        assert!(!c.intersects(&a));
    }

    #[test]
    fn test_source_range_offset_saturates() {
        assert_eq!(SourceRange::new(10, 20).offset(5), SourceRange::new(15, 25));
        assert_eq!(SourceRange::new(10, 20).offset(-5), SourceRange::new(5, 15));
        assert_eq!(SourceRange::new(10, 20).offset(-50), SourceRange::new(0, 0));
        assert_eq!(
            SourceRange::new(u32::MAX - 1, u32::MAX).offset(10),
            SourceRange::new(u32::MAX, u32::MAX),
        );
    }

    #[test]
    fn test_mapping_source_to_generated() {
        let mapping = SourceMapping::new(SourceRange::new(10, 20), SourceRange::new(100, 110));

        assert_eq!(mapping.source_to_generated(10), Some(100));
        assert_eq!(mapping.source_to_generated(15), Some(105));
        assert_eq!(mapping.source_to_generated(19), Some(109));
        assert_eq!(mapping.source_to_generated(9), None);
        assert_eq!(mapping.source_to_generated(20), None);
    }

    #[test]
    fn test_source_map() {
        let mut map = SourceMap::new();
        map.add_simple(10, 20, 100, 110);
        map.add_simple(30, 40, 200, 210);

        assert_eq!(map.to_generated(15), Some(105));
        assert_eq!(map.to_generated(35), Some(205));
        assert_eq!(map.to_generated(25), None);

        assert_eq!(map.to_source(105), Some(15));
        assert_eq!(map.to_source(205), Some(35));
    }
}
