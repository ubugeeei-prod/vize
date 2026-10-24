//! The SFC text Canon analyzes, and the way back to the text the user wrote.
//!
//! Canon reads every template through one HTML lane. An HTML template is
//! analyzed as authored. A Pug template is analyzed through its derived Vue
//! template (`PugBlockView`, the lane the compiler and linter use), so its
//! attribute and directive expressions are checked exactly like HTML ones.
//! Every position Canon reports for the analyzed text goes back through
//! [`AuthoredSfc::to_authored`]. A refused template retains its lowering errors
//! and a blank, byte-preserving body so unrelated script checking can continue.

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
    fn source(&self) -> &str {
        &self.source
    }

    fn to_host(&self, start: u32, end: u32) -> (u32, u32) {
        (start, end)
    }
}

pub(crate) struct AuthoredSfc<'a> {
    authored: &'a str,
    view: Option<SfcView>,
}

#[cfg(not(feature = "native"))]
type SfcView = PugBlockView;

#[cfg(feature = "native")]
enum SfcView {
    Derived(PugBlockView),
    Refused {
        source: vize_l0::String,
        diagnostics: Vec<vize_l0::diag::Diagnostic>,
        body_start: u32,
    },
}

#[cfg(feature = "native")]
impl SfcView {
    fn source(&self) -> &str {
        match self {
            Self::Derived(view) => &view.source,
            #[cfg(feature = "native")]
            Self::Refused { source, .. } => source,
        }
    }

    fn to_host(&self, start: u32, end: u32) -> (u32, u32) {
        match self {
            Self::Derived(view) => view.to_host(start, end),
            #[cfg(feature = "native")]
            Self::Refused { .. } => (start, end),
        }
    }
}

#[cfg(feature = "native")]
pub(crate) struct AuthoredTemplateError<'a> {
    pub(crate) message: &'a str,
    pub(crate) start: u32,
    pub(crate) end: u32,
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
        self.view.as_ref().map_or(self.authored, SfcView::source)
    }

    /// Proven lowering errors, borrowed directly from the shared frontend.
    #[cfg(feature = "native")]
    pub(crate) fn preprocessing_errors(&self) -> impl Iterator<Item = AuthoredTemplateError<'_>> {
        let (diagnostics, offset) = match &self.view {
            Some(SfcView::Refused {
                diagnostics,
                body_start,
                ..
            }) => (diagnostics.as_slice(), *body_start),
            _ => (&[][..], 0),
        };
        diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity() == vize_l0::diag::Severity::Error)
            .map(move |diagnostic| AuthoredTemplateError {
                message: &diagnostic.message,
                start: offset + diagnostic.span.start,
                end: offset + diagnostic.span.end,
            })
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
            vize_l0::line_index::LineIndex::new(view.source()).line_col_to_offset(line, column)
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
fn pug_view(_source: &str) -> Option<SfcView> {
    None
}

#[cfg(feature = "native")]
fn pug_view(source: &str) -> Option<SfcView> {
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
    match vize_l0::profile!(
        "canon.sfc.pug.lower",
        PugBlockView::new(source, template.loc.start..template.loc.end)
    ) {
        Ok(view) => Some(SfcView::Derived(view)),
        Err(refused) => {
            let mut masked = vize_l0::String::with_capacity(source.len());
            masked.push_str(source.get(..template.loc.start)?);
            for byte in source.get(template.loc.start..template.loc.end)?.bytes() {
                masked.push(if matches!(byte, b'\r' | b'\n') {
                    byte as char
                } else {
                    ' '
                });
            }
            masked.push_str(source.get(template.loc.end..)?);
            Some(SfcView::Refused {
                source: masked,
                diagnostics: refused.diagnostics,
                body_start: template.loc.start as u32,
            })
        }
    }
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

#[cfg(all(test, feature = "native"))]
mod refusal_tests;

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
    fn refused_pug_keeps_authored_errors_and_masks_only_its_template_body() {
        let source = "<template lang=\"pug\">\nmixin m\n  p x\n+m\n</template>\n";
        let sfc = AuthoredSfc::new(source);
        assert_eq!(
            sfc.analyzed(),
            "<template lang=\"pug\">\n       \n     \n  \n</template>\n"
        );
        assert_eq!(sfc.analyzed().len(), source.len());
        assert!(sfc.preprocessing_errors().next().is_some());
    }

    #[cfg(feature = "native")]
    #[test]
    fn pug_refusal_is_a_mapper_diagnostic_instead_of_a_clean_template() {
        let source = "<template lang=\"pug\">\ninclude other.pug\n</template>\n";
        let transform = crate::batch::generate_vue_content_mapper_transform(
            std::path::Path::new("/project/Refused.vue"),
            source,
        )
        .unwrap();
        assert!(
            !transform.diagnostics.is_empty(),
            "a refused preprocessor must not look clean"
        );
        let diagnostic = transform.diagnostics.first().unwrap();
        assert!(diagnostic.message_text.contains("include"));
        assert_eq!(diagnostic.start, source.find("include").unwrap());
    }
}
