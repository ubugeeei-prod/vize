//! Offset bookkeeping for rewritten module specifiers.

use vize_carton::String;

#[derive(Debug, Clone)]
pub struct OffsetAdjustment {
    pub original_offset: u32,
    pub adjustment: i32,
}

#[derive(Debug)]
pub struct RewriteResult {
    pub code: String,
    pub source_map: ImportSourceMap,
}

#[derive(Debug, Default, Clone)]
pub struct ImportSourceMap {
    adjustments: Vec<OffsetAdjustment>,
}

impl ImportSourceMap {
    pub fn new(adjustments: Vec<OffsetAdjustment>) -> Self {
        Self { adjustments }
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn get_original_offset(&self, virtual_offset: u32) -> u32 {
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
