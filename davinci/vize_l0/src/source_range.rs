//! Source range utilities for position tracking.
//!
//! Provides types for representing byte ranges in source code,
//! used for source mapping between Vue SFC and generated code.
//!
//! Creusot proves the range and mapping arithmetic for every `u32` and `i32`
//! value (`tests/formal/creusot/coordinates`, root `../creusot.rs`). The
//! contracts restate the documentation: a range is the half-open set of
//! offsets `[start, end)`. Under `cfg(creusot)` the derives other than
//! `Clone` and `Copy`, the `Display` impl and the `SourceMap` methods are
//! compiled out: no range or mapping law reads them, and the inventory
//! (`docs/davinci/decisions/2026-11-17-creusot-inventory.md`) names the
//! check that covers each. The product build keeps every item.

#[cfg(creusot)]
use creusot_std::prelude::{Int, ensures, logic};
#[cfg(not(creusot))]
use serde::{Deserialize, Serialize};

use crate::CompactString as String;

/// A range of byte offsets in a source file.
///
/// Used for tracking positions in source code for:
/// - Error reporting
/// - Source mapping between original and generated code
/// - IDE features (hover, goto definition, etc.)
#[cfg_attr(
    not(creusot),
    derive(
        Debug,
        Clone,
        Copy,
        Default,
        PartialEq,
        Eq,
        Hash,
        Serialize,
        Deserialize
    )
)]
#[cfg_attr(creusot, derive(Copy))]
pub struct SourceRange {
    /// Start byte offset (inclusive)
    pub start: u32,
    /// End byte offset (exclusive)
    pub end: u32,
}

impl SourceRange {
    /// Create a new source range.
    #[inline]
    #[cfg_attr(creusot, ensures(result.start == start && result.end == end))]
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    /// Create a range for a single position.
    #[inline]
    #[cfg_attr(creusot, ensures(result.start == offset && result.end == offset))]
    pub const fn point(offset: u32) -> Self {
        Self {
            start: offset,
            end: offset,
        }
    }

    /// Check if this range contains the given offset.
    #[inline]
    #[cfg_attr(creusot, ensures(result == self.covers(offset@)))]
    pub const fn contains(&self, offset: u32) -> bool {
        offset >= self.start && offset < self.end
    }

    /// Get the length of this range.
    #[inline]
    #[cfg_attr(creusot, ensures(result@ == if self.start@ <= self.end@ {
        self.end@ - self.start@
    } else {
        0
    }))]
    pub const fn len(&self) -> u32 {
        self.end.saturating_sub(self.start)
    }

    /// Check if this range is empty.
    #[inline]
    #[cfg_attr(creusot, ensures(result == forall<offset: Int> !self.covers(offset)))]
    pub const fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Check if this range intersects with another range.
    #[inline]
    #[cfg_attr(creusot, ensures(result == exists<offset: Int>
        self.covers(offset) && other.covers(offset)))]
    pub const fn intersects(&self, other: &Self) -> bool {
        !self.is_empty() && !other.is_empty() && self.start < other.end && other.start < self.end
    }

    /// Get the union of two ranges (smallest range containing both).
    #[inline]
    #[cfg_attr(creusot, ensures(result.encloses(*self) && result.encloses(*other)))]
    #[cfg_attr(creusot, ensures(forall<outer: SourceRange>
        outer.encloses(*self) && outer.encloses(*other) ==> outer.encloses(result)))]
    pub fn union(&self, other: &Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    /// Get the intersection of two ranges (largest range contained in both).
    #[inline]
    #[cfg_attr(creusot, ensures(match result {
        Some(both) => both.start@ < both.end@ && forall<offset: Int>
            both.covers(offset) == (self.covers(offset) && other.covers(offset)),
        None => forall<offset: Int> !(self.covers(offset) && other.covers(offset)),
    }))]
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
    #[cfg_attr(creusot, ensures(result.start@ == shifted(self.start@, amount@)))]
    #[cfg_attr(creusot, ensures(result.end@ == shifted(self.end@, amount@)))]
    pub const fn offset(&self, amount: i32) -> Self {
        Self {
            start: offset_u32(self.start, amount),
            end: offset_u32(self.end, amount),
        }
    }

    /// Extend this range by a given amount on both ends.
    #[inline]
    #[cfg_attr(creusot, ensures(result.start@ == shifted(self.start@, -amount@)))]
    #[cfg_attr(creusot, ensures(result.end@ == shifted(self.end@, amount@)))]
    pub const fn extend(&self, amount: u32) -> Self {
        Self {
            start: self.start.saturating_sub(amount),
            end: self.end.saturating_add(amount),
        }
    }
}

/// The proof crate's `Clone`: a copy. Creusot's `Clone` derive does not
/// resolve on a struct of machine integers.
#[cfg(creusot)]
impl Clone for SourceRange {
    #[ensures(result == *self)]
    fn clone(&self) -> Self {
        *self
    }
}

/// The laws' reading of a range: the offsets it covers, and enclosure.
#[cfg(creusot)]
impl SourceRange {
    /// `offset` lies in the half-open range `[start, end)`.
    #[logic(open)]
    pub fn covers(self, offset: Int) -> bool {
        pearlite! { self.start@ <= offset && offset < self.end@ }
    }

    /// The endpoints of `self` enclose those of `inner`.
    #[logic(open)]
    pub fn encloses(self, inner: SourceRange) -> bool {
        pearlite! { self.start@ <= inner.start@ && inner.end@ <= self.end@ }
    }
}

/// An offset moved by `amount` in the integers, clamped into `u32`.
#[cfg(creusot)]
#[logic(open)]
pub fn shifted(offset: Int, amount: Int) -> Int {
    pearlite! {
        if offset + amount < 0 {
            0
        } else if offset + amount > u32::MAX@ {
            u32::MAX@
        } else {
            offset + amount
        }
    }
}

/// The offset at the distance of `offset` from `from.start`, counted from
/// `to.start` and clamped to the last offset of `to` (its start when `to`
/// is empty).
#[cfg(creusot)]
#[logic(open)]
pub fn carried(offset: Int, from: SourceRange, to: SourceRange) -> Int {
    pearlite! {
        let last = if to.start@ < to.end@ { to.end@ - 1 } else { to.start@ };
        let target = to.start@ + (offset - from.start@);
        if target <= last { target } else { last }
    }
}

/// `amount.abs_diff(0)` is `amount.unsigned_abs()`, spelled with the std
/// function Creusot specifies; both compile to the same negation.
#[inline]
#[cfg_attr(creusot, ensures(result@ == shifted(value@, amount@)))]
const fn offset_u32(value: u32, amount: i32) -> u32 {
    if amount >= 0 {
        value.saturating_add(amount as u32)
    } else {
        value.saturating_sub(amount.abs_diff(0))
    }
}

#[cfg(not(creusot))]
impl std::fmt::Display for SourceRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

/// A single source mapping entry.
///
/// Maps a range in the original source to a range in the generated code.
#[cfg_attr(not(creusot), derive(Debug, Clone, Default))]
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
    #[cfg_attr(creusot, ensures(result.source == source && result.generated == generated))]
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
    #[cfg_attr(creusot, ensures(result == self.source.covers(offset@)))]
    pub fn contains_source(&self, offset: u32) -> bool {
        self.source.contains(offset)
    }

    /// Check if this mapping contains the generated offset.
    #[inline]
    #[cfg_attr(creusot, ensures(result == self.generated.covers(offset@)))]
    pub fn contains_generated(&self, offset: u32) -> bool {
        self.generated.contains(offset)
    }

    /// Map a source offset to generated offset.
    #[cfg_attr(creusot, ensures(match result {
        Some(target) => self.source.covers(source_offset@)
            && target@ == carried(source_offset@, self.source, self.generated),
        None => !self.source.covers(source_offset@),
    }))]
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
    #[cfg_attr(creusot, ensures(match result {
        Some(origin) => self.generated.covers(gen_offset@)
            && origin@ == carried(gen_offset@, self.generated, self.source),
        None => !self.generated.covers(gen_offset@),
    }))]
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
#[cfg_attr(not(creusot), derive(Debug, Clone))]
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
#[cfg_attr(not(creusot), derive(Debug, Clone, Default))]
pub struct SourceMap {
    /// Mappings sorted by source offset
    mappings: Vec<SourceMapping>,
    /// Block offset in the original file (for nested blocks)
    pub block_offset: u32,
}

#[cfg(not(creusot))]
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

    /// Map source offset to generated offset, through the first mapping in
    /// source order whose source range contains it.
    ///
    /// Mappings may be empty, inverted or overlapping, so source order alone
    /// does not bound where the containing mapping sits; the lookup checks
    /// each mapping, as [`Self::to_source`] does.
    pub fn to_generated(&self, source_offset: u32) -> Option<u32> {
        self.mappings
            .iter()
            .find_map(|mapping| mapping.source_to_generated(source_offset))
    }

    /// Map generated offset to source offset, through the first mapping whose
    /// generated range contains it, and shift the result by `block_offset`
    /// into file coordinates.
    ///
    /// `None` when no mapping contains the offset, or when the shifted offset
    /// lies past `u32::MAX`: a file offset beyond 4 GiB has no `u32`
    /// coordinate, so the map does not invent one.
    pub fn to_source(&self, gen_offset: u32) -> Option<u32> {
        self.mappings
            .iter()
            .find_map(|mapping| mapping.generated_to_source(gen_offset))?
            .checked_add(self.block_offset)
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

#[cfg(kani)]
mod proofs;

#[cfg(test)]
mod tests;
