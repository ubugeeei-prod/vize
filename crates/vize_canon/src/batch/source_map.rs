//! Source map for mapping materialized project files back to original sources.

use super::import_rewriter::ImportSourceMap;
use super::SfcBlockType;
use crate::virtual_ts::VizeMapping;

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
    /// Coarse block ranges used to recover the SFC block type.
    blocks: Vec<SfcBlockRange>,
}

impl SfcSourceMap {
    /// Create a new SFC source map.
    pub fn new(mappings: Vec<VizeMapping>, mut blocks: Vec<SfcBlockRange>) -> Self {
        blocks.sort_by_key(|block| block.start);
        Self { mappings, blocks }
    }

    /// Create an empty SFC source map.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Get the original SFC position from a virtual TS offset.
    pub fn get_original_position(&self, virtual_offset: u32) -> Option<(u32, u32, SfcBlockType)> {
        let virtual_offset = virtual_offset as usize;
        let mapping = self
            .mappings
            .iter()
            .find(|mapping| mapping.gen_range.contains(&virtual_offset))?;
        let delta = virtual_offset.saturating_sub(mapping.gen_range.start);
        let src_offset = mapping.src_range.start.saturating_add(delta);
        let src_offset = src_offset.min(mapping.src_range.end.saturating_sub(1));
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
}

/// Composite source map combining import rewrites and optional SFC mapping.
#[derive(Debug, Default)]
pub struct CompositeSourceMap {
    /// Source map for SFC blocks (only for `.vue` files).
    pub sfc_map: Option<SfcSourceMap>,
    /// Source map for import rewrites.
    pub import_map: ImportSourceMap,
}

impl CompositeSourceMap {
    /// Create a new composite source map for a Vue SFC.
    pub fn new_vue(sfc_map: SfcSourceMap, import_map: ImportSourceMap) -> Self {
        Self {
            sfc_map: Some(sfc_map),
            import_map,
        }
    }

    /// Create a new composite source map for a plain source file.
    pub fn new_script(import_map: ImportSourceMap) -> Self {
        Self {
            sfc_map: None,
            import_map,
        }
    }

    /// Create an empty composite source map.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Get the original position from a virtual position.
    ///
    /// The mapping order is:
    /// 1. Import rewrite mapping (materialized TS -> original TS with `.vue` specifiers)
    /// 2. SFC mapping (virtual TS -> SFC source)
    pub fn get_original_position(
        &self,
        virtual_offset: u32,
    ) -> Option<(u32, u32, Option<SfcBlockType>)> {
        let after_import = self.import_map.get_original_offset(virtual_offset);
        if let Some(ref sfc_map) = self.sfc_map {
            if let Some((offset, column, block)) = sfc_map.get_original_position(after_import) {
                return Some((offset, column, Some(block)));
            }
        }
        Some((after_import, 0, None))
    }
}

/// Convert byte offset to line and column (0-based).
pub fn offset_to_line_col(content: &str, offset: u32) -> Option<(u32, u32)> {
    let offset = offset as usize;
    if offset > content.len() {
        return None;
    }

    let mut line = 0u32;
    let mut col = 0u32;
    let mut current = 0;

    for ch in content.chars() {
        if current >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
        current += ch.len_utf8();
    }

    Some((line, col))
}

/// Convert line and column to byte offset (0-based).
pub fn line_col_to_offset(content: &str, line: u32, col: u32) -> Option<u32> {
    let mut current_line = 0u32;
    let mut current_col = 0u32;
    let mut offset = 0u32;

    for ch in content.chars() {
        if current_line == line && current_col == col {
            return Some(offset);
        }
        if ch == '\n' {
            if current_line == line {
                // Column out of bounds on this line
                return None;
            }
            current_line += 1;
            current_col = 0;
        } else {
            current_col += 1;
        }
        offset += ch.len_utf8() as u32;
    }

    // Handle end of file
    if current_line == line && current_col == col {
        return Some(offset);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{
        line_col_to_offset, offset_to_line_col, CompositeSourceMap, ImportSourceMap, SfcBlockRange,
        SfcBlockType, SfcSourceMap,
    };
    use crate::virtual_ts::VizeMapping;
    use std::ops::Range;

    #[test]
    fn test_offset_to_line_col() {
        let content = "abc\ndef\nghi";
        assert_eq!(offset_to_line_col(content, 0), Some((0, 0)));
        assert_eq!(offset_to_line_col(content, 3), Some((0, 3)));
        assert_eq!(offset_to_line_col(content, 4), Some((1, 0)));
        assert_eq!(offset_to_line_col(content, 8), Some((2, 0)));
    }

    #[test]
    fn test_line_col_to_offset() {
        let content = "abc\ndef\nghi";
        assert_eq!(line_col_to_offset(content, 0, 0), Some(0));
        assert_eq!(line_col_to_offset(content, 0, 3), Some(3));
        assert_eq!(line_col_to_offset(content, 1, 0), Some(4));
        assert_eq!(line_col_to_offset(content, 2, 0), Some(8));
    }

    #[test]
    fn test_sfc_source_map() {
        let map = SfcSourceMap::new(
            vec![VizeMapping {
                gen_range: Range {
                    start: 100,
                    end: 200,
                },
                src_range: Range {
                    start: 50,
                    end: 150,
                },
            }],
            vec![SfcBlockRange {
                start: 50,
                end: 150,
                block_type: SfcBlockType::ScriptSetup,
            }],
        );

        // Virtual offset 150 should map to SFC offset 100
        let result = map.get_original_position(150);
        assert!(result.is_some());
        let (offset, _, block) = result.unwrap();
        assert_eq!(offset, 100);
        assert_eq!(block, SfcBlockType::ScriptSetup);
    }

    #[test]
    fn test_composite_source_map() {
        let sfc_map = SfcSourceMap::empty();
        let import_map = ImportSourceMap::empty();
        let composite = CompositeSourceMap::new_vue(sfc_map, import_map);

        // Should return position even without mappings
        let result = composite.get_original_position(50);
        assert!(result.is_some());
    }
}
