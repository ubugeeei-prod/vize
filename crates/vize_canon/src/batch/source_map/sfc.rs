//! SFC-specific source maps for virtual TypeScript.

use crate::batch::SfcBlockType;
use crate::virtual_ts::{VizeMapping, VizeSemanticLink};

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

/// Precise source map for SFC virtual TypeScript.
#[derive(Debug, Default)]
pub struct SfcSourceMap {
    /// Fine-grained virtual TS mappings emitted by `vize_canon::virtual_ts`.
    mappings: Vec<VizeMapping>,
    /// Semantic links emitted by `vize_canon::virtual_ts`.
    semantic_links: Vec<VizeSemanticLink>,
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
            mappings,
            semantic_links,
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
        let mapping = crate::virtual_ts::mapping::mapping_for_generated_offset(
            &self.mappings,
            virtual_offset,
        )?;
        let src_offset =
            crate::virtual_ts::mapping::map_generated_offset_to_source(mapping, virtual_offset);
        let src_offset = u32::try_from(src_offset).ok()?;
        let block = self
            .blocks
            .iter()
            .find(|block| block.contains(src_offset))
            .map(|block| block.block_type)
            .unwrap_or(SfcBlockType::Script);
        Some((src_offset, 0, block))
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
            .mappings
            .iter()
            .find(|mapping| mapping.src_range.contains(&sfc_offset))?;
        let delta = sfc_offset.saturating_sub(mapping.src_range.start);
        let virtual_offset = mapping.gen_range.start.saturating_add(delta);
        u32::try_from(virtual_offset).ok()
    }

    /// Access the raw virtual TS mappings.
    pub fn mappings(&self) -> &[VizeMapping] {
        &self.mappings
    }

    /// Access the raw virtual TS semantic links.
    pub fn semantic_links(&self) -> &[VizeSemanticLink] {
        &self.semantic_links
    }
}
