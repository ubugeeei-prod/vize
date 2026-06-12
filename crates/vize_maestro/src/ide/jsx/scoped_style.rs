//! Embedded CSS virtual documents for JSX `<style scoped>` blocks (#1495,
//! #1498).
//!
//! A Vize JSX/TSX component may carry a `<style scoped>` intrinsic element whose
//! body is the component's scoped CSS (`vize_atelier_jsx` extracts this at
//! compile time, see [`vize_atelier_jsx::ScopedStyle`]). For the editor this is
//! exactly an embedded language: the SFC path exposes each `<style>` block as a
//! CSS [`VirtualDocument`](crate::virtual_code::VirtualDocument) with a 1:1
//! source map so the editor's CSS language service can attach CSS diagnostics
//! and the positions map straight back to the `.vue` source. This module mirrors
//! that for JSX so CSS inside a JSX `<style scoped>` gets the same treatment.
//!
//! The CSS content is taken as the **raw source slice** spanning the `<style>`
//! element's children, which keeps the generated→source map a true byte-for-byte
//! 1:1 (no escape cooking), matching the SFC [`StyleCodeGenerator`] precisely.
//! Detection mirrors `vize_atelier_jsx`'s extractor: a lowercase intrinsic
//! `style` element carrying a bare `scoped` attribute.
#![allow(clippy::disallowed_methods)]

use oxc_allocator::Allocator;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeName, JSXChild, JSXElement, JSXElementName};
use oxc_ast_visit::{Visit, walk};
use oxc_span::GetSpan;
use tower_lsp::lsp_types::Url;
use vize_atelier_jsx::{JsxLang, parse_module};

use crate::virtual_code::{
    MappingFeatures, SourceMap, SourceMapping, SourceRange, VirtualDocument, VirtualLanguage,
};

/// One JSX `<style scoped>` block's CSS content and its byte range in the
/// original `.jsx`/`.tsx` source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::ide) struct JsxScopedStyle {
    /// Raw CSS text, exactly as it appears between the `<style>` tags.
    pub(in crate::ide) css: String,
    /// Inclusive-start byte offset of the CSS in the original source.
    pub(in crate::ide) start: u32,
    /// Exclusive-end byte offset of the CSS in the original source.
    pub(in crate::ide) end: u32,
}

/// Embedded-CSS provider for JSX `<style scoped>` blocks.
pub struct JsxScopedStyleService;

impl JsxScopedStyleService {
    /// Extract every JSX `<style scoped>` block's CSS content + source span from
    /// a `.jsx`/`.tsx` document, in source order.
    pub(in crate::ide) fn extract(content: &str, lang: JsxLang) -> Vec<JsxScopedStyle> {
        let allocator = Allocator::default();
        let parsed = parse_module(&allocator, content, lang);

        let mut collector = ScopedStyleCollector {
            source: content,
            styles: Vec::new(),
        };
        collector.visit_program(&parsed.program);
        collector.styles
    }

    /// Build embedded CSS virtual documents for a `.jsx`/`.tsx` document's
    /// `<style scoped>` blocks, one per block, each with a 1:1 source map back
    /// into the original source. Returns an empty vector when the component has
    /// no scoped style. Mirrors the SFC style virtual-document path so the
    /// editor's CSS service sees JSX scoped CSS identically.
    pub fn virtual_css_documents(content: &str, uri: &Url) -> Vec<VirtualDocument> {
        let lang = JsxLang::from_path(uri.path());
        let styles = Self::extract(content, lang);

        styles
            .into_iter()
            .enumerate()
            .map(|(index, style)| Self::build_virtual_document(uri.path(), index, &style))
            .collect()
    }

    /// Build one 1:1-mapped CSS [`VirtualDocument`] for a scoped-style block.
    fn build_virtual_document(
        base_path: &str,
        index: usize,
        style: &JsxScopedStyle,
    ) -> VirtualDocument {
        let content_len = style.css.len() as u32;

        // A single 1:1 mapping for the whole CSS body (generated == source
        // bytes), exactly like the SFC `StyleCodeGenerator`.
        let mappings = if content_len > 0 {
            vec![SourceMapping::with_features(
                SourceRange::new(0, content_len),
                SourceRange::new(0, content_len),
                MappingFeatures::all(),
            )]
        } else {
            Vec::new()
        };

        let mut source_map = SourceMap::from_mappings(mappings);
        // The block offset is where the CSS starts in the original source, so the
        // CSS service's positions resolve back to the right `.jsx`/`.tsx` bytes.
        source_map.set_block_offset(style.start);

        VirtualDocument {
            uri: vize_carton::cstr!("{base_path}.__jsx_style_{index}.css").to_string(),
            content: style.css.clone(),
            language: VirtualLanguage::Style,
            source_map,
        }
    }
}

/// Walks the parsed program collecting `<style scoped>` CSS spans. Only
/// `visit_jsx_element` is overridden, so no scope bookkeeping (and hence no
/// `oxc_syntax`) is needed — `walk` still descends into nested elements.
struct ScopedStyleCollector<'s> {
    source: &'s str,
    styles: Vec<JsxScopedStyle>,
}

impl<'a, 's> Visit<'a> for ScopedStyleCollector<'s> {
    fn visit_jsx_element(&mut self, element: &JSXElement<'a>) {
        if let Some(style) = self.try_extract(element) {
            self.styles.push(style);
        }
        // Descend so a `<style scoped>` nested in fragments/children is still
        // found, and so multiple components in one module each contribute.
        walk::walk_jsx_element(self, element);
    }
}

impl ScopedStyleCollector<'_> {
    /// If `element` is an intrinsic `<style scoped>`, return its CSS slice + span.
    fn try_extract(&self, element: &JSXElement<'_>) -> Option<JsxScopedStyle> {
        let opening = &element.opening_element;
        if !is_intrinsic_style(&opening.name) || !has_scoped_attr(&opening.attributes) {
            return None;
        }
        self.children_css_span(element)
    }

    /// Byte range covering the element's CSS, as a raw source slice (positions
    /// map 1:1).
    ///
    /// For the idiomatic single template-literal (`` {`…`} ``) or string-literal
    /// (`{'…'}`) body the span is narrowed to the literal's inner text, so the
    /// CSS document is pure CSS with no JSX braces/quotes. For bare-text or mixed
    /// children it falls back to first-child-start..last-child-end.
    fn children_css_span(&self, element: &JSXElement<'_>) -> Option<JsxScopedStyle> {
        let children = &element.children;
        // For the idiomatic single template-literal / string body, narrow the
        // span to the literal's inner text so the CSS document is pure CSS.
        if let [JSXChild::ExpressionContainer(container)] = children.as_slice() {
            use oxc_ast::ast::JSXExpression;
            match &container.expression {
                JSXExpression::TemplateLiteral(template) => {
                    // Cover from the first quasi to the last quasi (the static
                    // CSS text), excluding the backticks.
                    let first = template.quasis.first()?;
                    let last = template.quasis.last()?;
                    let start = first.span.start;
                    let end = last.span.end;
                    return self.slice_span(start, end);
                }
                JSXExpression::StringLiteral(string) => {
                    // Inside the quotes.
                    let span = string.span;
                    let start = span.start.saturating_add(1);
                    let end = span.end.saturating_sub(1).max(start);
                    return self.slice_span(start, end);
                }
                _ => {}
            }
        }

        // Fallback: bare JSX text and/or mixed children — cover from the first
        // child's start to the last child's end.
        let first = children.first()?;
        let last = children.last()?;
        let start = first.span().start;
        let end = last.span().end;
        self.slice_span(start, end)
    }

    /// Materialize a [`JsxScopedStyle`] from a `[start, end)` source byte range.
    fn slice_span(&self, start: u32, end: u32) -> Option<JsxScopedStyle> {
        let s = start as usize;
        let e = (end as usize).min(self.source.len());
        if s >= e {
            return None;
        }
        let css = self.source.get(s..e)?;
        // Skip whitespace-only bodies — there is no CSS to diagnose.
        if css.trim().is_empty() {
            return None;
        }
        #[allow(clippy::disallowed_methods)]
        Some(JsxScopedStyle {
            css: css.to_string(),
            start,
            end: e as u32,
        })
    }
}

/// Whether a JSX element name is the intrinsic lowercase `style` tag.
fn is_intrinsic_style(name: &JSXElementName<'_>) -> bool {
    match name {
        JSXElementName::Identifier(id) => id.name.as_str() == "style",
        JSXElementName::IdentifierReference(reference) => reference.name.as_str() == "style",
        _ => false,
    }
}

/// Whether the opening element carries a bare `scoped` attribute.
fn has_scoped_attr(attributes: &[JSXAttributeItem<'_>]) -> bool {
    attributes.iter().any(|item| match item {
        JSXAttributeItem::Attribute(attr) => match &attr.name {
            JSXAttributeName::Identifier(id) => id.name.as_str() == "scoped",
            JSXAttributeName::NamespacedName(_) => false,
        },
        JSXAttributeItem::SpreadAttribute(_) => false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn extract(source: &str) -> Vec<JsxScopedStyle> {
        JsxScopedStyleService::extract(source, JsxLang::Tsx)
    }

    #[test]
    fn extracts_template_literal_scoped_css() {
        let source = "const C = () => (\n  <>\n    <div class=\"box\">hi</div>\n    <style scoped>{`\n      .box { color: red; }\n    `}</style>\n  </>\n);\n";
        let styles = extract(source);
        assert_eq!(styles.len(), 1, "expected one scoped style block");
        let style = &styles[0];
        assert!(style.css.contains(".box"), "css: {:?}", style.css);
        assert!(style.css.contains("color: red"));
        // The template literal's backticks are excluded, so the body is pure CSS.
        assert!(
            !style.css.contains('`'),
            "backtick leaked into CSS: {:?}",
            style.css
        );
        // The captured span is a verbatim slice of the source (1:1 mapping
        // invariant).
        assert_eq!(&source[style.start as usize..style.end as usize], style.css);
    }

    #[test]
    fn extracts_string_literal_scoped_css() {
        let source = "const C = () => <style scoped>{'.a{color:blue}'}</style>;\n";
        let styles = extract(source);
        assert_eq!(styles.len(), 1);
        assert_eq!(styles[0].css, ".a{color:blue}");
        assert_eq!(
            &source[styles[0].start as usize..styles[0].end as usize],
            styles[0].css
        );
    }

    #[test]
    fn ignores_non_scoped_style() {
        let source = "const C = () => <style>{`.a{color:red}`}</style>;\n";
        assert!(extract(source).is_empty());
    }

    #[test]
    fn ignores_component_without_style() {
        let source = "const C = () => <div class=\"a\">hi</div>;\n";
        assert!(extract(source).is_empty());
    }

    #[test]
    fn virtual_document_has_one_to_one_source_map() {
        let source = "const C = () => <style scoped>{`.box{color:red}`}</style>;\n";
        let uri = Url::parse("file:///tmp/Comp.tsx").unwrap();
        let docs = JsxScopedStyleService::virtual_css_documents(source, &uri);
        assert_eq!(docs.len(), 1);
        let doc = &docs[0];
        assert_eq!(doc.language, VirtualLanguage::Style);
        assert!(doc.uri.as_str().ends_with(".css"));
        assert_eq!(doc.content, ".box{color:red}");
        // The 1:1 map round-trips: generated offset 0 maps back to the CSS start.
        let css_start = source.find(".box").unwrap() as u32;
        assert_eq!(doc.source_map.to_source(0), Some(css_start));
    }

    #[test]
    fn no_virtual_documents_without_scoped_style() {
        let source = "const C = () => <div>hi</div>;\n";
        let uri = Url::parse("file:///tmp/Comp.tsx").unwrap();
        assert!(JsxScopedStyleService::virtual_css_documents(source, &uri).is_empty());
    }
}
