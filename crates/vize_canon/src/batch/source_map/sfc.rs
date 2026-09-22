//! SFC block view over the projection mapping model.

use crate::batch::SfcBlockType;
use crate::virtual_ts::{ProjectionMapping, VizeMapping, VizeSemanticLink};

/// Original SFC block span in source coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SfcBlockRange {
    /// Inclusive start byte offset in the original SFC source.
    pub start: u32,
    /// Exclusive end byte offset in the original SFC source.
    pub end: u32,
    /// Block kind.
    pub block_type: SfcBlockType,
}

impl SfcBlockRange {
    #[inline]
    pub fn contains(self, offset: u32) -> bool {
        offset >= self.start && offset < self.end
    }
}

/// Precise source map for SFC virtual TypeScript: the generator's
/// [`ProjectionMapping`] plus the coarse SFC block ranges that recover a
/// position's block type.
#[derive(Debug, Default)]
pub struct SfcSourceMap {
    /// Span links and semantic links emitted by `vize_canon::virtual_ts`.
    projection: ProjectionMapping,
    /// Coarse block ranges used to recover the SFC block type.
    blocks: Vec<SfcBlockRange>,
}

impl SfcSourceMap {
    /// Create a new SFC source map.
    pub fn new(mappings: Vec<VizeMapping>, blocks: Vec<SfcBlockRange>) -> Self {
        Self::new_with_semantic_links(mappings, blocks, Vec::new())
    }

    /// Create a new SFC source map with stable semantic links.
    pub fn new_with_semantic_links(
        mappings: Vec<VizeMapping>,
        mut blocks: Vec<SfcBlockRange>,
        semantic_links: Vec<VizeSemanticLink>,
    ) -> Self {
        blocks.sort_by_key(|block| block.start);
        Self {
            projection: ProjectionMapping::from_parts(mappings, semantic_links),
            blocks,
        }
    }

    /// Create an empty SFC source map.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Get the original SFC position from a virtual TS offset.
    ///
    /// Shares the language server's arithmetic: the narrowest mapping wins,
    /// an exact-expression sub-span beats the enclosing statement mapping,
    /// and synthetic generated text can never push a position past the
    /// authored bytes. A diagnostic on a synthetic prop-check identifier
    /// therefore lands on the authored expression instead of a
    /// content-independent generated column.
    pub fn get_original_position(&self, virtual_offset: u32) -> Option<(u32, u32, SfcBlockType)> {
        let virtual_offset = virtual_offset as usize;
        let mapping = self.projection.span_at_generated(virtual_offset)?;
        let src_offset =
            crate::virtual_ts::mapping::map_generated_offset_to_source(mapping, virtual_offset);
        let src_offset = u32::try_from(src_offset).ok()?;
        Some((src_offset, 0, self.block_type_at(src_offset)))
    }

    /// Get the virtual TS offset from an SFC offset.
    pub fn get_virtual_offset(&self, sfc_offset: u32, block_type: SfcBlockType) -> Option<u32> {
        let sfc_offset = usize::try_from(sfc_offset).ok()?;
        let block = self
            .blocks
            .iter()
            .find(|block| block.block_type == block_type && block.contains(sfc_offset as u32))?;
        if !block.contains(sfc_offset as u32) {
            return None;
        }

        let mapping = self
            .projection
            .spans()
            .iter()
            .find(|mapping| mapping.src_range.contains(&sfc_offset))?;
        let delta = sfc_offset.saturating_sub(mapping.src_range.start);
        let virtual_offset = mapping.gen_range.start.saturating_add(delta);
        u32::try_from(virtual_offset).ok()
    }

    /// The projection mapping this view reads.
    pub fn projection(&self) -> &ProjectionMapping {
        &self.projection
    }

    /// The SFC block containing authored `offset` (script outside every block).
    pub fn block_type_at(&self, offset: u32) -> SfcBlockType {
        self.blocks
            .iter()
            .find(|block| block.contains(offset))
            .map(|block| block.block_type)
            .unwrap_or(SfcBlockType::Script)
    }

    /// The template block's authored range.
    pub fn template_block(&self) -> Option<std::ops::Range<usize>> {
        self.blocks
            .iter()
            .find(|block| block.block_type == SfcBlockType::Template)
            .map(|block| block.start as usize..block.end as usize)
    }

    /// Access the raw virtual TS mappings.
    pub fn mappings(&self) -> &[VizeMapping] {
        self.projection.spans()
    }

    /// Access the raw virtual TS semantic links.
    pub fn semantic_links(&self) -> &[VizeSemanticLink] {
        self.projection.semantic_links()
    }
}
