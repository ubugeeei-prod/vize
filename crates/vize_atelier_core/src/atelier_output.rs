//! The `AtelierOutput` finishing plate.
//!
//! `AtelierOutput` is the structured output a target Atelier produces *before*
//! its generated JavaScript becomes a flat string: imports, hoists, render
//! functions, exports, the section ranges that describe them, an optional
//! source-map fragment, and any fallback marks observed while emitting. It is
//! the owned counterpart to the borrowed [`RenduPlate`], and the shared shape
//! DOM/SSR/Vapor/SFC assembly can grow onto so structure is registered while
//! emitting instead of recovered by scanning generated code later.

use vize_atlas::{SourceAtlasFallback, SourceAtlasFallbackSet, SourceAtlasTarget};
use vize_carton::String;
use vize_rendu::{RenduModuleSections, RenduPlate};

/// Structured target output before it is flattened to a single module string.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct AtelierOutput {
    pub imports: String,
    pub hoists: String,
    pub functions: String,
    pub exports: String,
    source_map: Option<String>,
    fallbacks: SourceAtlasFallbackSet,
}

impl AtelierOutput {
    /// Build a finishing plate from its module chunks.
    pub fn new(imports: String, hoists: String, functions: String, exports: String) -> Self {
        Self {
            imports,
            hoists,
            functions,
            exports,
            source_map: None,
            fallbacks: SourceAtlasFallbackSet::empty(),
        }
    }

    /// Attach a source-map fragment for this output.
    pub fn with_source_map(mut self, source_map: String) -> Self {
        self.source_map = Some(source_map);
        self
    }

    /// Record a fallback observed while emitting this output.
    pub fn with_fallback(mut self, fallback: SourceAtlasFallback) -> Self {
        self.fallbacks = self.fallbacks.with(fallback);
        self
    }

    /// The source-map fragment, if one was attached.
    pub fn source_map(&self) -> Option<&str> {
        self.source_map.as_deref()
    }

    /// The fallback marks recorded for this output.
    pub fn fallbacks(&self) -> SourceAtlasFallbackSet {
        self.fallbacks
    }

    /// Section ranges describing the chunks once flattened.
    ///
    /// Matches [`flatten`](Self::flatten)'s layout, so a consumer can slice the
    /// flat string by section without rescanning it.
    pub fn sections(&self) -> RenduModuleSections {
        RenduModuleSections::from_chunk_lengths(
            self.imports.len(),
            self.hoists.len(),
            self.functions.len(),
            self.exports.len(),
        )
    }

    /// Flatten the chunks into the final module string.
    ///
    /// `imports` and `hoists` are adjacent; a single newline separates the
    /// hoists from the functions and the functions from the exports, matching
    /// the marks [`RenduModuleSections::from_chunk_lengths`] records.
    pub fn flatten(&self) -> String {
        let mut code = String::with_capacity(
            self.imports.len() + self.hoists.len() + self.functions.len() + self.exports.len() + 2,
        );
        code.push_str(&self.imports);
        code.push_str(&self.hoists);
        code.push('\n');
        code.push_str(&self.functions);
        code.push('\n');
        code.push_str(&self.exports);
        code
    }

    /// A borrowed [`RenduPlate`] view of this output for a target lane.
    pub fn borrowed_plate(&self, target: SourceAtlasTarget) -> RenduPlate<'_> {
        RenduPlate::new(
            target,
            &self.imports,
            &self.hoists,
            &self.functions,
            &self.exports,
        )
        .with_source_map(self.source_map())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> AtelierOutput {
        AtelierOutput::new(
            String::from("import { h } from \"vue\"\n"),
            String::from("const _hoisted_1 = null\n"),
            String::from("function render() {}"),
            String::from("export default _sfc_main\n"),
        )
    }

    #[test]
    fn sections_match_the_flattened_layout() {
        let output = sample();
        let sections = output.sections();
        // The exports section ends at the flattened length, so section ranges
        // slice the flat string without a rescan.
        assert_eq!(sections.exports.end, output.flatten().len());
        assert_eq!(sections.imports.start, 0);
    }

    #[test]
    fn flatten_keeps_imports_and_hoists_adjacent_with_newline_separators() {
        let output = sample();
        let flat = output.flatten();
        let expected = vize_carton::cstr!(
            "{}{}\n{}\n{}",
            "import { h } from \"vue\"\n",
            "const _hoisted_1 = null\n",
            "function render() {}",
            "export default _sfc_main\n"
        );
        assert_eq!(flat.as_str(), expected.as_str());
    }

    #[test]
    fn carries_source_map_and_fallback_marks() {
        let output = sample()
            .with_source_map(String::from("{\"version\":3}"))
            .with_fallback(SourceAtlasFallback::SourceMapCompositionSkipped);

        assert_eq!(output.source_map(), Some("{\"version\":3}"));
        let fallbacks = output.fallbacks();
        assert!(fallbacks.contains(SourceAtlasFallback::SourceMapCompositionSkipped));

        // The borrowed plate exposes the same chunks and map for a target lane.
        let plate = output.borrowed_plate(SourceAtlasTarget::Ssr);
        assert_eq!(plate.functions, "function render() {}");
        assert!(plate.has_source_map());
    }
}
