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

use vize_carton::String;

use super::Lowerer;

/// Raw (un-rewritten) CSS extracted from a `<style scoped>` JSX element, kept in
/// source order so the backend can rewrite + scope it once the scope id exists.
pub(crate) struct RawScopedStyle {
    /// The CSS text exactly as authored between the `<style>` tags.
    pub css: String,
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
        let css = collect_style_css(&element.children);
        self.push_scoped_style(RawScopedStyle { css });
        true
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

/// Concatenate the CSS text from a `<style>` element's children. Supports the
/// idiomatic template-literal form (`{`…`}`), a plain string literal
/// (`{'…'}`), and bare JSX text. Interpolations and other expressions are
/// skipped (CSS `v-bind()` / style expressions are a deferred follow-up).
fn collect_style_css(children: &[JSXChild<'_>]) -> String {
    let mut css = String::default();
    for child in children {
        match child {
            JSXChild::Text(text) => css.push_str(text.value.as_str()),
            JSXChild::ExpressionContainer(container) => match &container.expression {
                JSXExpression::StringLiteral(string) => css.push_str(string.value.as_str()),
                JSXExpression::TemplateLiteral(template) => {
                    push_template_css(&mut css, template);
                }
                _ => {}
            },
            _ => {}
        }
    }
    css
}

/// Append a template literal's cooked text. A `<style scoped>` body is expected
/// to be a static template literal with no `${}` substitutions; if any are
/// present we still emit the literal quasis so the static rules survive.
fn push_template_css(css: &mut String, template: &oxc_ast::ast::TemplateLiteral<'_>) {
    for quasi in &template.quasis {
        let text = quasi
            .value
            .cooked
            .as_ref()
            .map(|cooked| cooked.as_str())
            .unwrap_or_else(|| quasi.value.raw.as_str());
        css.push_str(text);
    }
}
