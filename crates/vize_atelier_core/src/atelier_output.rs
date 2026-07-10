//! The `AtelierOutput` finishing plate.
//!
//! `AtelierOutput` is the structured output a target Atelier produces *before*
//! its generated JavaScript becomes a flat string: imports, hoists, render
//! functions, exports, the section ranges that describe them, an optional
//! source-map fragment, and any fallback marks observed while emitting. It is
//! the owner of a borrowed [`AtelierOutputView`], and the shared shape
//! DOM/SSR/Vapor/SFC assembly can grow onto so structure is registered while
//! emitting instead of recovered by scanning generated code later.

use vize_carton::String;

#[path = "atelier_output/sections.rs"]
mod sections;

pub use sections::{AtelierModuleSections, AtelierRange, AtelierRenderSections};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum AtelierTarget {
    Dom,
    Vdom,
    Sfc,
    Ssr,
    Vapor,
    Jsx,
    Tsx,
    VirtualTs,
    Diagnostics,
    SourceMap,
    Vitrine,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum AtelierFallback {
    SourceMapCompositionSkipped,
    SourceMapFragmentUnavailable,
    VaporSsr,
    UnsupportedVaporShape,
    CacheBypass,
    VirtualTsSkipped,
    LegacyCompatibility,
}

impl AtelierFallback {
    pub const fn profile_counter(self) -> &'static str {
        match self {
            Self::SourceMapCompositionSkipped => {
                "atelier.profile.fallback.source_map_composition_skipped"
            }
            Self::SourceMapFragmentUnavailable => {
                "atelier.profile.fallback.source_map_fragment_unavailable"
            }
            Self::VaporSsr => "atelier.profile.fallback.vapor_ssr",
            Self::UnsupportedVaporShape => "atelier.profile.fallback.vapor_unsupported_shape",
            Self::CacheBypass => "atelier.profile.fallback.cache_bypass",
            Self::VirtualTsSkipped => "atelier.profile.fallback.virtual_ts_skipped",
            Self::LegacyCompatibility => "atelier.profile.fallback.legacy_compatibility",
        }
    }

    const fn bit(self) -> u16 {
        1 << self as u8
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub struct AtelierFallbackSet(u16);

impl AtelierFallbackSet {
    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn with(mut self, fallback: AtelierFallback) -> Self {
        self.0 |= fallback.bit();
        self
    }

    pub const fn contains(self, fallback: AtelierFallback) -> bool {
        self.0 & fallback.bit() != 0
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AtelierOutputChunk {
    Imports,
    Hoists,
    Functions,
    Exports,
}

/// Borrowed view of a structured emitted module.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct AtelierOutputView<'a> {
    pub target: AtelierTarget,
    pub imports: &'a str,
    pub hoists: &'a str,
    pub functions: &'a str,
    pub exports: &'a str,
    pub module_sections: AtelierModuleSections,
    pub render_sections: Option<AtelierRenderSections>,
    pub source_map: Option<&'a str>,
}

impl<'a> AtelierOutputView<'a> {
    pub const fn new(
        target: AtelierTarget,
        imports: &'a str,
        hoists: &'a str,
        functions: &'a str,
        exports: &'a str,
    ) -> Self {
        Self {
            target,
            imports,
            hoists,
            functions,
            exports,
            module_sections: AtelierModuleSections::from_chunk_lengths(
                imports.len(),
                hoists.len(),
                functions.len(),
                exports.len(),
            ),
            render_sections: None,
            source_map: None,
        }
    }

    pub const fn with_render_sections(mut self, sections: Option<AtelierRenderSections>) -> Self {
        self.render_sections = sections;
        self
    }

    pub const fn with_source_map(mut self, source_map: Option<&'a str>) -> Self {
        self.source_map = source_map;
        self
    }

    pub const fn chunk(self, chunk: AtelierOutputChunk) -> &'a str {
        match chunk {
            AtelierOutputChunk::Imports => self.imports,
            AtelierOutputChunk::Hoists => self.hoists,
            AtelierOutputChunk::Functions => self.functions,
            AtelierOutputChunk::Exports => self.exports,
        }
    }
}

/// Structured target output before it is flattened to a single module string.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct AtelierOutput {
    pub imports: String,
    pub hoists: String,
    pub functions: String,
    pub exports: String,
    source_map: Option<String>,
    fallbacks: AtelierFallbackSet,
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
            fallbacks: AtelierFallbackSet::empty(),
        }
    }

    /// Attach a source-map fragment for this output.
    pub fn with_source_map(mut self, source_map: String) -> Self {
        self.source_map = Some(source_map);
        self
    }

    /// Record a fallback observed while emitting this output.
    pub fn with_fallback(mut self, fallback: AtelierFallback) -> Self {
        self.fallbacks = self.fallbacks.with(fallback);
        self
    }

    /// The source-map fragment, if one was attached.
    pub fn source_map(&self) -> Option<&str> {
        self.source_map.as_deref()
    }

    /// The fallback marks recorded for this output.
    pub fn fallbacks(&self) -> AtelierFallbackSet {
        self.fallbacks
    }

    /// Section ranges describing the chunks once flattened.
    ///
    /// Matches [`flatten`](Self::flatten)'s layout, so a consumer can slice the
    /// flat string by section without rescanning it.
    pub fn sections(&self) -> AtelierModuleSections {
        AtelierModuleSections::from_chunk_lengths(
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
    /// the marks [`AtelierModuleSections::from_chunk_lengths`] records.
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

    /// A borrowed module view of this output for a target lane.
    pub fn view(&self, target: AtelierTarget) -> AtelierOutputView<'_> {
        AtelierOutputView::new(
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
            .with_fallback(AtelierFallback::SourceMapCompositionSkipped);

        assert_eq!(output.source_map(), Some("{\"version\":3}"));
        let fallbacks = output.fallbacks();
        assert!(fallbacks.contains(AtelierFallback::SourceMapCompositionSkipped));

        // The borrowed plate exposes the same chunks and map for a target lane.
        let view = output.view(AtelierTarget::Ssr);
        assert_eq!(view.functions, "function render() {}");
        assert_eq!(view.source_map, Some("{\"version\":3}"));
    }
}
