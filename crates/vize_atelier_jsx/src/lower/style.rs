//! Extracting `<style scoped>` JSX blocks (#1495).
//!
//! Vize JSX/TSX components may carry a single `<style scoped>` intrinsic element
//! (idiomatically at the bottom of the markup, mirroring an SFC
//! `<template>`→`<style>` layout):
//!
//! ```tsx
//! const Comp = () => (
//!   <>
//!     <div class="box">hi</div>
//!     <style scoped>{`
//!       .box {
//!         color: red;
//!       }
//!     `}</style>
//!   </>
//! )
//! ```
//!
//! Such an element is **not** rendered as a runtime `<style>` vnode: it is
//! extracted at compile time, its CSS content captured, and the backends
//! ([`crate::dom`] / [`crate::vapor`]) run the SFC scoped-CSS rewrite + scope-id
//! injection over it. This module performs only the detection + extraction; the
//! scoped transform lives in the backends so it can reuse `vize_atelier_sfc`.

use oxc_ast::ast::{
    JSXAttributeItem, JSXAttributeName, JSXChild, JSXElement, JSXElementName, JSXExpression,
};
use oxc_span::GetSpan;

use vize_carton::String;

use super::Lowerer;

/// Raw (un-rewritten) CSS extracted from a `<style scoped>` JSX element, kept in
/// source order so the backend can rewrite + scope it once the scope id exists.
pub(crate) struct RawScopedStyle {
    /// The CSS text exactly as authored between the `<style>` tags.
    pub css: String,
    /// The template-literal interpolation expressions (`${expr}`) embedded in
    /// the style block, in source order, each paired with its byte range in the
    /// original `.jsx`/`.tsx` source. These are consumed by the style extractor
    /// (they are *not* CSS text), but the type checker re-emits them so they
    /// type-check against the component scope (#1497).
    pub exprs: std::vec::Vec<ScopedStyleExpr>,
}

/// One interpolation expression (`${expr}`) recovered from a `<style scoped>`
/// template literal: its source text and the byte range it occupied.
pub(crate) struct ScopedStyleExpr {
    /// The expression source text, exactly as authored between `${` and `}`.
    pub content: String,
    /// Byte offset of the expression's start in the original source.
    pub start: u32,
    /// Byte offset of the expression's end in the original source.
    pub end: u32,
}

impl<'a, 'm, 's> Lowerer<'a, 'm, 's> {
    /// If `element` is an intrinsic `<style>` carrying a `scoped` attribute,
    /// capture its CSS into the pending-styles buffer and return `true` so the
    /// caller drops it from the rendered children. Returns `false` for any other
    /// element (including a non-`scoped` `<style>`, which renders normally).
    pub(crate) fn try_extract_scoped_style(&mut self, element: &JSXElement<'_>) -> bool {
        let opening = &element.opening_element;
        if !is_intrinsic_style(&opening.name) || !has_scoped_attr(&opening.attributes) {
            return false;
        }
        let mut css = String::default();
        let mut exprs = std::vec::Vec::new();
        self.collect_style(&element.children, &mut css, &mut exprs);
        self.push_scoped_style(RawScopedStyle { css, exprs });
        true
    }

    /// Concatenate the CSS text from a `<style>` element's children, recording
    /// each template-literal interpolation expression's source text and byte
    /// range into `exprs`. Supports the idiomatic template-literal form
    /// (`{`…`}`), a plain string literal (`{'…'}`), and bare JSX text.
    fn collect_style(
        &self,
        children: &[JSXChild<'_>],
        css: &mut String,
        exprs: &mut std::vec::Vec<ScopedStyleExpr>,
    ) {
        for child in children {
            match child {
                JSXChild::Text(text) => css.push_str(text.value.as_str()),
                JSXChild::ExpressionContainer(container) => match &container.expression {
                    JSXExpression::StringLiteral(string) => css.push_str(string.value.as_str()),
                    JSXExpression::TemplateLiteral(template) => {
                        self.push_template_css(css, exprs, template);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    /// Append a template literal's cooked CSS text and record each `${expr}`
    /// interpolation's source text + byte range.
    ///
    /// A `<style scoped>` body is typically a static template literal, but any
    /// `${expr}` substitutions (e.g. `color: ${props.color}`) reference script
    /// values that must type-check against the component scope (#1497); their
    /// spans are captured here so the type checker can re-emit them. The cooked
    /// quasis are still concatenated so the static rules survive for the CSS
    /// scoping backends.
    fn push_template_css(
        &self,
        css: &mut String,
        exprs: &mut std::vec::Vec<ScopedStyleExpr>,
        template: &oxc_ast::ast::TemplateLiteral<'_>,
    ) {
        for quasi in &template.quasis {
            let text = quasi
                .value
                .cooked
                .as_ref()
                .map(|cooked| cooked.as_str())
                .unwrap_or_else(|| quasi.value.raw.as_str());
            css.push_str(text);
        }
        for expression in &template.expressions {
            let span = expression.span();
            let content = self.mapper().slice(span);
            if content.trim().is_empty() {
                continue;
            }
            exprs.push(ScopedStyleExpr {
                content: String::from(content),
                start: span.start,
                end: span.end,
            });
        }
    }
}

/// Whether a JSX element name is the intrinsic `style` tag (lowercase, the Vue
/// JSX intrinsic convention).
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
