//! Source map for mapping materialized project files back to original sources.

use super::SfcBlockType;
use super::import_rewriter::ImportSourceMap;

mod sfc;
pub use sfc::{SfcBlockRange, SfcSourceMap};

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
    /// 2. SFC mapping (virtual TS -> SFC source); unmapped SFC positions are rejected
    pub fn get_original_position(
        &self,
        virtual_offset: u32,
    ) -> Option<(u32, u32, Option<SfcBlockType>)> {
        let after_import = self.import_map.get_original_offset(virtual_offset);
        if let Some(ref sfc_map) = self.sfc_map {
            return sfc_map
                .get_original_position(after_import)
                .map(|(offset, column, block)| (offset, column, Some(block)));
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
        CompositeSourceMap, ImportSourceMap, SfcBlockRange, SfcBlockType, SfcSourceMap,
        line_col_to_offset, offset_to_line_col,
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
                sub_spans: Vec::new(),
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
    fn sub_spans_map_synthetic_identifiers_to_authored_expressions() {
        use crate::virtual_ts::VizeSubSpan;
        let map = SfcSourceMap::new(
            vec![VizeMapping {
                // const __vize_prop_check_0_msg: __T = 42;
                gen_range: Range {
                    start: 100,
                    end: 145,
                },
                // :msg="42"
                src_range: Range { start: 16, end: 25 },
                sub_spans: vec![VizeSubSpan {
                    // the synthetic check identifier
                    gen_range: Range {
                        start: 106,
                        end: 129,
                    },
                    // the authored bound expression
                    src_range: Range { start: 22, end: 24 },
                }],
            }],
            vec![SfcBlockRange {
                start: 0,
                end: 200,
                block_type: SfcBlockType::Template,
            }],
        );

        // A diagnostic on the identifier resolves to the authored expression.
        assert_eq!(
            map.get_original_position(106),
            Some((22, 0, SfcBlockType::Template))
        );
        // Offsets outside the sub-span clamp to the authored prop range
        // instead of drifting to a content-independent generated column.
        assert_eq!(
            map.get_original_position(144),
            Some((25, 0, SfcBlockType::Template))
        );
    }

    #[test]
    fn the_narrowest_mapping_wins_for_nested_generated_ranges() {
        let map = SfcSourceMap::new(
            vec![
                VizeMapping {
                    gen_range: Range { start: 0, end: 300 },
                    src_range: Range { start: 0, end: 100 },
                    sub_spans: Vec::new(),
                },
                VizeMapping {
                    gen_range: Range {
                        start: 120,
                        end: 140,
                    },
                    src_range: Range { start: 40, end: 60 },
                    sub_spans: Vec::new(),
                },
            ],
            vec![SfcBlockRange {
                start: 0,
                end: 100,
                block_type: SfcBlockType::Template,
            }],
        );

        assert_eq!(
            map.get_original_position(125),
            Some((45, 0, SfcBlockType::Template))
        );
    }

    #[test]
    fn composite_source_map_only_uses_identity_fallback_for_plain_scripts() {
        let import_map = ImportSourceMap::empty();
        assert_eq!(
            CompositeSourceMap::new_script(import_map).get_original_position(50),
            Some((50, 0, None))
        );
        assert_eq!(
            CompositeSourceMap::new_vue(SfcSourceMap::empty(), ImportSourceMap::empty())
                .get_original_position(50),
            None
        );
    }
}
