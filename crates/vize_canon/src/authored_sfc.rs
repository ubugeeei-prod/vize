//! The SFC text Canon analyzes, and the way back to the text the user wrote.
//!
//! Canon reads every template through one HTML lane. An HTML template is
//! analyzed as authored. A Pug template is analyzed through its derived Vue
//! template (`PugBlockView`, the lane the compiler and linter use), so its
//! attribute and directive expressions are checked exactly like HTML ones.
//! Every position Canon reports for the analyzed text goes back through
//! [`AuthoredSfc::to_authored`]; a template the Pug lowering refuses keeps the
//! authored text.

use std::ops::Range;

#[cfg(feature = "native")]
use vize_l1_to_l2::lower::pug::PugBlockView;

use crate::sfc_typecheck::SfcTypeCheckResult;
use crate::virtual_ts::VizeMapping;

/// Without the native levels (WASM), Pug has no derived view.
#[cfg(not(feature = "native"))]
struct PugBlockView {
    source: vize_l0::String,
}

#[cfg(not(feature = "native"))]
impl PugBlockView {
    fn to_host(&self, start: u32, end: u32) -> (u32, u32) {
        (start, end)
    }
}

pub(crate) struct AuthoredSfc<'a> {
    authored: &'a str,
    view: Option<PugBlockView>,
}

impl<'a> AuthoredSfc<'a> {
    pub(crate) fn new(authored: &'a str) -> Self {
        Self {
            authored,
            view: pug_view(authored),
        }
    }

    /// The text Canon parses and generates virtual TypeScript from.
    pub(crate) fn analyzed(&self) -> &str {
        self.view
            .as_ref()
            .map_or(self.authored, |view| view.source.as_str())
    }

    /// Whether the analyzed text differs from the authored text.
    #[cfg(feature = "native")]
    pub(crate) fn is_derived(&self) -> bool {
        self.view.is_some()
    }

    /// The authored range of an analyzed range.
    pub(crate) fn to_authored(&self, range: Range<usize>) -> Range<usize> {
        let Some(view) = &self.view else {
            return range;
        };
        let (start, end) = view.to_host(range.start as u32, range.end as u32);
        start as usize..end as usize
    }

    /// Point every source range of `mappings` at the authored text.
    pub(crate) fn remap_mappings(&self, mappings: &mut [VizeMapping]) {
        if self.view.is_none() {
            return;
        }
        for mapping in mappings {
            mapping.src_range = self.to_authored(mapping.src_range.clone());
            for sub_span in &mut mapping.sub_spans {
                sub_span.src_range = self.to_authored(sub_span.src_range.clone());
            }
        }
    }

    /// Point every position of a single-file check result at the authored text.
    pub(crate) fn remap_type_check_result(&self, result: &mut SfcTypeCheckResult) {
        if self.view.is_none() {
            return;
        }
        let remap = |start: &mut u32, end: &mut u32| {
            let range = self.to_authored(*start as usize..*end as usize);
            (*start, *end) = (range.start as u32, range.end as u32);
        };
        for diagnostic in &mut result.diagnostics {
            remap(&mut diagnostic.start, &mut diagnostic.end);
            for related in &mut diagnostic.related {
                if related.filename.is_none() {
                    remap(&mut related.start, &mut related.end);
                }
            }
        }
        self.remap_mappings(&mut result.virtual_ts_mappings);
    }

    /// Point SFC block ranges at the authored text.
    #[cfg(feature = "native")]
    pub(crate) fn remap_block_ranges(
        &self,
        mut blocks: Vec<crate::batch::SfcBlockRange>,
    ) -> Vec<crate::batch::SfcBlockRange> {
        for block in &mut blocks {
            let range = self.to_authored(block.start as usize..block.end as usize);
            (block.start, block.end) = (range.start as u32, range.end as u32);
        }
        blocks
    }

    /// The authored 0-based line and UTF-16 column of an analyzed one.
    #[cfg(feature = "native")]
    pub(crate) fn to_authored_line_column(&self, line: u32, column: u32) -> (u32, u32) {
        let Some(view) = &self.view else {
            return (line, column);
        };
        let Some(offset) =
            vize_l0::line_index::LineIndex::new(&view.source).line_col_to_offset(line, column)
        else {
            return (line, column);
        };
        let authored = self.to_authored(offset..offset).start;
        vize_l0::line_index::offset_to_line_col(self.authored, authored)
    }
}

/// The derived view of an inline `<template lang="pug">`, if the SFC has one
/// the lowering accepts.
#[cfg(not(feature = "native"))]
fn pug_view(_source: &str) -> Option<PugBlockView> {
    None
}

#[cfg(feature = "native")]
fn pug_view(source: &str) -> Option<PugBlockView> {
    // Cheap gate: only sources that mention pug or jade pay for the SFC parse.
    if !mentions_pug_language(source) {
        return None;
    }
    let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).ok()?;
    let template = descriptor.template.as_ref()?;
    let pug = template
        .lang
        .as_deref()
        .is_some_and(|lang| lang.eq_ignore_ascii_case("pug") || lang.eq_ignore_ascii_case("jade"));
    if template.src.is_some() || !pug {
        return None;
    }
    PugBlockView::new(source, template.loc.start..template.loc.end).ok()
}

#[cfg(feature = "native")]
fn mentions_pug_language(source: &str) -> bool {
    source
        .as_bytes()
        .windows(3)
        .any(|window| window.eq_ignore_ascii_case(b"pug"))
        || source
            .as_bytes()
            .windows(4)
            .any(|window| window.eq_ignore_ascii_case(b"jade"))
}

#[expect(clippy::string_slice, reason = "tests assert by panicking")]
#[cfg(test)]
mod tests {
    use super::AuthoredSfc;

    #[test]
    fn html_templates_are_analyzed_as_authored() {
        let source =
            "<script setup>const a = 1</script>\n<template><p :title=\"a\" /></template>\n";
        let sfc = AuthoredSfc::new(source);
        assert_eq!(sfc.analyzed(), source);
        assert_eq!(sfc.to_authored(3..9), 3..9);
    }

    #[cfg(feature = "native")]
    #[test]
    fn pug_expressions_map_back_to_their_authored_bytes() {
        let source = "<script setup>const count = 1</script>\n<template lang=\"pug\">\np(:title=\"count\") {{ count }}\n</template>\n<style>.a{}</style>\n";
        let sfc = AuthoredSfc::new(source);
        let analyzed = sfc.analyzed();
        assert_eq!(
            analyzed,
            "<script setup>const count = 1</script>\n<template lang=\"pug\"><p :title=\"count\">{{ count }}</p></template>\n<style>.a{}</style>\n"
        );
        let attribute = analyzed.find("\"count\"").unwrap() + 1;
        let authored = sfc.to_authored(attribute..attribute + 5);
        assert_eq!(&source[authored.clone()], "count");
        assert_eq!(authored.start, source.find("\"count\"").unwrap() + 1);
        let style = analyzed.find(".a{}").unwrap();
        assert_eq!(
            sfc.to_authored(style..style + 4),
            source.find(".a{}").unwrap()..source.find(".a{}").unwrap() + 4
        );
        let (line, column) = vize_l0::line_index::offset_to_line_col(analyzed, attribute);
        assert_eq!(
            sfc.to_authored_line_column(line, column),
            vize_l0::line_index::offset_to_line_col(source, authored.start)
        );
    }

    #[cfg(feature = "native")]
    #[test]
    fn jade_alias_uses_the_same_authored_mapping_as_pug() {
        for language in ["pug", "jade", "JADE"] {
            let language = ["lang=\"", language, "\""].concat();
            let source = "<script setup>const count = 1</script>\n<template lang=\"pug\">\np(:title=\"count\") {{ count }}\n</template>\n"
                .replace("lang=\"pug\"", &language);
            let sfc = AuthoredSfc::new(&source);
            let analyzed = sfc.analyzed();
            let attribute = analyzed.find("\"count\"").unwrap() + 1;
            let authored = sfc.to_authored(attribute..attribute + 5);
            assert_eq!(&source[authored], "count", "{language}");
        }
    }

    #[cfg(feature = "native")]
    #[test]
    fn refused_pug_keeps_the_authored_text() {
        let source = "<template lang=\"pug\">\nmixin m\n  p x\n+m\n</template>\n";
        assert_eq!(AuthoredSfc::new(source).analyzed(), source);
    }
}
