//! Source map implementation for bidirectional position mapping.
//!
//! Maps positions between the original SFC and generated virtual documents.
#![allow(clippy::disallowed_types)]

use super::SourceRange;

/// Mapping features that can be enabled/disabled per mapping.
#[derive(Debug, Clone, Copy, Default)]
pub struct MappingFeatures {
    /// Enable hover information
    pub hover: bool,
    /// Enable completion
    pub completion: bool,
    /// Enable go-to-definition
    pub definition: bool,
    /// Enable find references
    pub references: bool,
    /// Enable rename
    pub rename: bool,
    /// Enable diagnostics
    pub diagnostics: bool,
    /// Enable semantic tokens
    pub semantic_tokens: bool,
}

impl MappingFeatures {
    /// All features enabled.
    pub fn all() -> Self {
        Self {
            hover: true,
            completion: true,
            definition: true,
            references: true,
            rename: true,
            diagnostics: true,
            semantic_tokens: true,
        }
    }

    /// Only hover and diagnostics.
    pub fn basic() -> Self {
        Self {
            hover: true,
            diagnostics: true,
            ..Default::default()
        }
    }

    /// Completion and definition.
    pub fn navigation() -> Self {
        Self {
            completion: true,
            definition: true,
            references: true,
            ..Default::default()
        }
    }
}

/// A single source mapping entry.
#[derive(Debug, Clone)]
pub struct SourceMapping {
    /// Range in the original SFC
    pub source: SourceRange,
    /// Range in the generated virtual document
    pub generated: SourceRange,
    /// Features enabled for this mapping
    pub features: MappingFeatures,
    /// Optional data associated with this mapping
    pub data: Option<MappingData>,
}

impl SourceMapping {
    /// Create a new mapping with all features enabled.
    pub fn new(source: SourceRange, generated: SourceRange) -> Self {
        Self {
            source,
            generated,
            features: MappingFeatures::all(),
            data: None,
        }
    }

    /// Create with specific features.
    pub fn with_features(
        source: SourceRange,
        generated: SourceRange,
        features: MappingFeatures,
    ) -> Self {
        Self {
            source,
            generated,
            features,
            data: None,
        }
    }

    /// Create with data.
    pub fn with_data(source: SourceRange, generated: SourceRange, data: MappingData) -> Self {
        Self {
            source,
            generated,
            features: MappingFeatures::all(),
            data: Some(data),
        }
    }

    /// Check if this mapping contains the source offset.
    pub fn contains_source(&self, offset: u32) -> bool {
        self.source.contains(offset)
    }

    /// Check if this mapping contains the generated offset.
    pub fn contains_generated(&self, offset: u32) -> bool {
        self.generated.contains(offset)
    }

    /// Map a source offset to generated offset.
    pub fn source_to_generated(&self, source_offset: u32) -> Option<u32> {
        if self.source.contains(source_offset) {
            let relative = source_offset - self.source.start;
            // Clamp to generated range
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
            // Clamp to source range
            let src_offset = self.source.start + relative.min(self.source.len().saturating_sub(1));
            Some(src_offset)
        } else {
            None
        }
    }
}

/// Additional data for a mapping.
#[derive(Debug, Clone)]
pub enum MappingData {
    /// Expression in template (e.g., {{ expr }})
    Expression {
        /// The expression text
        text: String,
    },
    /// Directive argument (e.g., v-bind:prop)
    DirectiveArg {
        /// Directive name
        name: String,
        /// Argument name
        arg: String,
    },
    /// Directive expression (e.g., v-if="expr")
    DirectiveExpr {
        /// Directive name
        name: String,
        /// Expression text
        expr: String,
    },
    /// Event handler (e.g., @click="handler")
    EventHandler {
        /// Event name
        event: String,
        /// Handler expression
        handler: String,
    },
    /// Component tag
    Component {
        /// Component name
        name: String,
    },
    /// Slot binding
    Slot {
        /// Slot name
        name: String,
    },
}

/// Bidirectional source map.
#[derive(Debug, Clone, Default)]
pub struct SourceMap {
    /// Mappings sorted by source offset
    mappings: Vec<SourceMapping>,
    /// Block offset in the original SFC (for nested blocks like template)
    pub block_offset: u32,
}

impl SourceMap {
    /// Create an empty source map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create from a list of mappings.
    pub fn from_mappings(mut mappings: Vec<SourceMapping>) -> Self {
        // Sort by source start offset for binary search
        mappings.sort_by_key(|m| m.source.start);
        Self {
            mappings,
            block_offset: 0,
        }
    }

    /// Set the block offset (for template block in SFC).
    pub fn set_block_offset(&mut self, offset: u32) {
        self.block_offset = offset;
    }

    /// Add a mapping.
    pub fn add(&mut self, mapping: SourceMapping) {
        self.mappings.push(mapping);
        // Keep sorted
        self.mappings.sort_by_key(|m| m.source.start);
    }

    /// Add a simple mapping.
    pub fn add_simple(&mut self, source_start: u32, source_end: u32, gen_start: u32, gen_end: u32) {
        self.add(SourceMapping::new(
            SourceRange::new(source_start, source_end),
            SourceRange::new(gen_start, gen_end),
        ));
    }

    /// Get all mappings.
    pub fn mappings(&self) -> &[SourceMapping] {
        &self.mappings
    }

    /// Find mappings that contain the source offset.
    pub fn find_by_source(&self, offset: u32) -> Vec<&SourceMapping> {
        self.mappings
            .iter()
            .filter(|m| m.contains_source(offset))
            .collect()
    }

    /// Find mappings that contain the generated offset.
    pub fn find_by_generated(&self, offset: u32) -> Vec<&SourceMapping> {
        self.mappings
            .iter()
            .filter(|m| m.contains_generated(offset))
            .collect()
    }

    /// Map source offset to generated offset.
    /// Returns the first matching mapping's result.
    pub fn to_generated(&self, source_offset: u32) -> Option<u32> {
        // Binary search for the first mapping that might contain this offset
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
        // Linear search (could optimize with a second sorted index)
        for mapping in &self.mappings {
            if let Some(src) = mapping.generated_to_source(gen_offset) {
                return Some(src + self.block_offset);
            }
        }
        None
    }

    /// Map source offset to generated offset with feature check.
    pub fn to_generated_for(
        &self,
        source_offset: u32,
        check: impl Fn(&MappingFeatures) -> bool,
    ) -> Option<u32> {
        for mapping in &self.mappings {
            if mapping.contains_source(source_offset) && check(&mapping.features) {
                return mapping.source_to_generated(source_offset);
            }
        }
        None
    }

    /// Map generated offset to source offset with feature check.
    pub fn to_source_for(
        &self,
        gen_offset: u32,
        check: impl Fn(&MappingFeatures) -> bool,
    ) -> Option<u32> {
        for mapping in &self.mappings {
            if mapping.contains_generated(gen_offset) && check(&mapping.features) {
                return mapping
                    .generated_to_source(gen_offset)
                    .map(|o| o + self.block_offset);
            }
        }
        None
    }

    /// Get the mapping containing the source offset.
    pub fn get_mapping_at_source(&self, offset: u32) -> Option<&SourceMapping> {
        self.mappings.iter().find(|m| m.contains_source(offset))
    }

    /// Get the mapping containing the generated offset.
    pub fn get_mapping_at_generated(&self, offset: u32) -> Option<&SourceMapping> {
        self.mappings.iter().find(|m| m.contains_generated(offset))
    }

    /// Map a source range to generated range.
    pub fn source_range_to_generated(&self, source: SourceRange) -> Option<SourceRange> {
        let start = self.to_generated(source.start)?;
        let end = self.to_generated(source.end.saturating_sub(1))? + 1;
        Some(SourceRange::new(start, end))
    }

    /// Map a generated range to source range.
    pub fn generated_range_to_source(&self, generated: SourceRange) -> Option<SourceRange> {
        let start = self.to_source(generated.start)?;
        let end = self.to_source(generated.end.saturating_sub(1))? + 1;
        Some(SourceRange::new(start, end))
    }

    /// Check if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
    }

    /// Get the number of mappings.
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
    fn test_mapping_source_to_generated() {
        let mapping = SourceMapping::new(SourceRange::new(10, 20), SourceRange::new(100, 110));

        assert_eq!(mapping.source_to_generated(10), Some(100));
        assert_eq!(mapping.source_to_generated(15), Some(105));
        assert_eq!(mapping.source_to_generated(19), Some(109));
        assert_eq!(mapping.source_to_generated(9), None);
        assert_eq!(mapping.source_to_generated(20), None);
    }

    #[test]
    fn test_mapping_generated_to_source() {
        let mapping = SourceMapping::new(SourceRange::new(10, 20), SourceRange::new(100, 110));

        assert_eq!(mapping.generated_to_source(100), Some(10));
        assert_eq!(mapping.generated_to_source(105), Some(15));
        assert_eq!(mapping.generated_to_source(109), Some(19));
        assert_eq!(mapping.generated_to_source(99), None);
        assert_eq!(mapping.generated_to_source(110), None);
    }

    #[test]
    fn test_source_map_to_generated() {
        let mut map = SourceMap::new();
        map.add_simple(10, 20, 100, 110);
        map.add_simple(30, 40, 200, 210);

        assert_eq!(map.to_generated(15), Some(105));
        assert_eq!(map.to_generated(35), Some(205));
        assert_eq!(map.to_generated(25), None);
    }

    #[test]
    fn test_source_map_to_source() {
        let mut map = SourceMap::new();
        map.add_simple(10, 20, 100, 110);
        map.add_simple(30, 40, 200, 210);

        assert_eq!(map.to_source(105), Some(15));
        assert_eq!(map.to_source(205), Some(35));
        assert_eq!(map.to_source(150), None);
    }

    #[test]
    fn test_source_map_with_block_offset() {
        let mut map = SourceMap::new();
        map.set_block_offset(50); // Template starts at offset 50 in SFC
        map.add_simple(10, 20, 100, 110);

        assert_eq!(map.to_source(105), Some(65)); // 15 + 50
    }
}
