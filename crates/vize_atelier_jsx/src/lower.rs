//! Lowering OXC JSX nodes into Vize's shared template IR.
//!
//! The [`Lowerer`] walks OXC JSX/TSX nodes and produces
//! [`vize_relief::RootNode`]s. Owned strings are copied out of the OXC
//! arena (Vize uses `CompactString`), and the tree structure is built in the
//! caller-supplied [`Allocator`] arena, so the lowered IR does not borrow the OXC
//! allocator and outlives parsing.

mod attr;
mod babel_slot;
mod child;
mod control_flow;
mod element;
mod expr;
mod name;
mod slot;
mod style;
mod text;
mod v_custom;
mod v_model;
mod v_models;
mod v_slots;

pub(crate) use style::{RawScopedStyle, ScopedStyleExpr};

use oxc_ast::ast::{JSXElement, JSXElementName, JSXFragment};
use oxc_semantic::Scoping;
use oxc_span::Span;
use vize_relief::{RootNode, TemplateChildNode};
use vize_s0::{Allocator, Box, String, ToCompactString, Vec};

use crate::BabelIsCustomElement;
use crate::compat::JsxCompatMode;
use crate::diagnostics::JsxDiagnostic;
use crate::span::SpanMapper;

/// The `@vue/babel-plugin-jsx` options that change *lowering* rather than code
/// generation (#3391).
///
/// Grouped into one value so adding a Babel option stays additive at every call
/// site instead of widening three signatures each time.
#[derive(Clone, Copy, Default)]
pub(crate) struct BabelLoweringOptions<'m> {
    /// Whether the current compilation can emit Babel-compatible VDOM output.
    /// SSR has its own backend and must not inherit VDOM-only lowering rules.
    pub vdom_compat: bool,
    /// Collision-free `_transformOn` binding, when `transformOn` is enabled.
    pub transform_on_helper: Option<&'m str>,
    /// Collision-free `_isSlot` binding, when `enableObjectSlots` is enabled.
    pub object_slots_helper: Option<&'m str>,
    /// Project-specific custom-element classifier from `isCustomElement`.
    pub is_custom_element: Option<&'m BabelIsCustomElement>,
    /// Whether the Babel VDOM lane may apply at all. Babel's options are a
    /// client-VDOM contract, so the compiler switches this off for SSR exactly
    /// as it withholds the `transformOn` / `enableObjectSlots` /
    /// `isCustomElement` inputs there.
    pub vdom_lane: bool,
}

/// Arena box, for call sites that cannot hold a `&self` borrow across the
/// value they are boxing. The arena containers take `&&Allocator` (P1-10).
pub(crate) fn boxed_in<T>(allocator: &Allocator, value: T) -> Box<'_, T> {
    Box::new_in(value, &allocator)
}

/// Lowers OXC JSX nodes into Vize IR against a single source text.
pub struct Lowerer<'a, 'm, 's: 'a> {
    bump: &'a Allocator,
    mapper: &'m SpanMapper<'s>,
    compat: JsxCompatMode,
    is_custom_element: Option<&'m BabelIsCustomElement>,
    scoping: Option<Scoping>,
    custom_element_spans: std::vec::Vec<(u32, u32)>,
    babel_vdom_lane: bool,
    transform_on_helper: Option<String>,
    object_slots_helper: Option<String>,
    babel_vdom_compat_allowed: bool,
    babel_vdom_compat_active: bool,
    diagnostics: std::vec::Vec<JsxDiagnostic>,
    /// `<style scoped>` blocks extracted from the render root currently being
    /// lowered, in source order. Drained by [`Self::take_scoped_styles`] once
    /// the root is built.
    pending_styles: std::vec::Vec<RawScopedStyle>,
}

impl<'a, 'm, 's: 'a> Lowerer<'a, 'm, 's> {
    /// Build a lowerer that allocates IR in `bump` and maps spans via `mapper`.
    pub fn new(bump: &'a Allocator, mapper: &'m SpanMapper<'s>) -> Self {
        Self::with_compat(
            bump,
            mapper,
            JsxCompatMode::Native,
            BabelLoweringOptions::default(),
            None,
        )
    }

    /// Build a lowerer using the requested project-level JSX semantics.
    pub(crate) fn with_compat(
        bump: &'a Allocator,
        mapper: &'m SpanMapper<'s>,
        compat: JsxCompatMode,
        babel: BabelLoweringOptions<'m>,
        scoping: Option<Scoping>,
    ) -> Self {
        Self {
            bump,
            mapper,
            compat,
            is_custom_element: babel.is_custom_element,
            scoping,
            custom_element_spans: std::vec::Vec::new(),
            babel_vdom_lane: babel.vdom_lane,
            transform_on_helper: babel.transform_on_helper.map(String::from),
            object_slots_helper: babel.object_slots_helper.map(String::from),
            babel_vdom_compat_allowed: babel.vdom_compat,
            babel_vdom_compat_active: false,
            diagnostics: std::vec::Vec::new(),
            pending_styles: std::vec::Vec::new(),
        }
    }

    /// Record a `<style scoped>` block extracted during child lowering.
    pub(crate) fn push_scoped_style(&mut self, style: RawScopedStyle) {
        self.pending_styles.push(style);
    }

    /// Drain the `<style scoped>` blocks accumulated while lowering the current
    /// render root, concatenating their CSS into one block (multiple `<style
    /// scoped>` elements in one component join, mirroring SFC's multi-`<style>`
    /// behavior) and flattening every template-literal interpolation expression
    /// (`${expr}`) across them, in source order. Returns `None` when no scoped
    /// style was present.
    pub(crate) fn take_scoped_styles(
        &mut self,
    ) -> Option<(String, std::vec::Vec<ScopedStyleExpr>)> {
        if self.pending_styles.is_empty() {
            return None;
        }
        let styles = std::mem::take(&mut self.pending_styles);
        let mut css = String::default();
        let mut exprs = std::vec::Vec::new();
        for (index, style) in styles.into_iter().enumerate() {
            if index > 0 {
                css.push('\n');
            }
            css.push_str(style.css.trim());
            exprs.extend(style.exprs);
        }
        Some((css, exprs))
    }

    /// Diagnostics accumulated so far.
    pub fn diagnostics(&self) -> &[JsxDiagnostic] {
        &self.diagnostics
    }

    /// Consume the lowerer and return its diagnostics.
    pub fn into_diagnostics(self) -> std::vec::Vec<JsxDiagnostic> {
        self.diagnostics
    }

    pub(crate) fn into_compat_parts(
        self,
    ) -> (std::vec::Vec<JsxDiagnostic>, std::vec::Vec<(u32, u32)>) {
        (self.diagnostics, self.custom_element_spans)
    }

    /// Record a diagnostic.
    pub fn report(&mut self, diagnostic: JsxDiagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Report a fixed-text error over `span`.
    ///
    /// Shared by the built-in attribute lowerings (`v_models`, `v_slots`), which
    /// reject far more shapes than they accept and would otherwise each repeat
    /// the same `JsxDiagnostic::error` plumbing.
    pub(crate) fn reject(&mut self, span: Span, message: &'static str) {
        self.report(JsxDiagnostic::error(message, span.start, span.end));
    }

    /// Report a formatted error over `span`, for messages that quote the
    /// offending source text.
    pub(crate) fn reject_at(&mut self, span: Span, message: std::fmt::Arguments<'_>) {
        self.report(JsxDiagnostic::error(
            message.to_compact_string(),
            span.start,
            span.end,
        ));
    }

    /// Lower a JSX element as the single root of a render output.
    pub fn lower_element_root(&mut self, element: &JSXElement<'_>) -> RootNode<'a> {
        let mut root = RootNode::new(self.bump, self.mapper.slice(element.span));
        root.loc = self.mapper.location(element.span);
        let node = self.lower_element_node(element);
        root.children
            .push(TemplateChildNode::Element(Box::new_in(node, &self.bump)));
        root
    }

    /// Lower a JSX fragment (`<>...</>`) as a render root whose children become
    /// the root children directly (no wrapper element).
    pub fn lower_fragment_root(&mut self, fragment: &JSXFragment<'_>) -> RootNode<'a> {
        let mut root = RootNode::new(self.bump, self.mapper.slice(fragment.span));
        root.loc = self.mapper.location(fragment.span);
        root.children = self.lower_children(&fragment.children);
        root
    }

    /// Shared accessor used by sibling lowering modules.
    pub(crate) fn bump(&self) -> &'a Allocator {
        self.bump
    }

    /// Arena box; the arena containers take `&&Allocator` (Davinci P1-10).
    pub(crate) fn boxed<T>(&self, value: T) -> Box<'a, T> {
        boxed_in(self.bump, value)
    }

    /// Empty arena vector.
    pub(crate) fn vec<T>(&self) -> Vec<'a, T> {
        Vec::new_in(&self.bump)
    }

    /// Shared accessor used by sibling lowering modules.
    pub(crate) fn mapper(&self) -> &'m SpanMapper<'s> {
        self.mapper
    }

    /// Whether the project opted into `@vue/babel-plugin-jsx` semantics.
    pub(crate) fn uses_babel_compat(&self) -> bool {
        self.compat.is_babel()
    }

    /// Whether Babel-only VDOM child semantics apply to the current render root.
    pub(crate) fn uses_babel_vdom_compat(&self) -> bool {
        self.uses_babel_compat() && self.babel_vdom_compat_allowed && self.babel_vdom_compat_active
    }

    /// Select whether the current render root may use VDOM-only Babel options.
    pub(crate) fn set_current_output_mode(&mut self, mode: crate::JsxOutputMode) {
        self.babel_vdom_compat_active = self.babel_vdom_lane && mode == crate::JsxOutputMode::Vdom;
    }

    /// Whether Babel's `isCustomElement` predicate selects this JSX tag.
    pub(crate) fn is_babel_custom_element(&self, tag: &str) -> bool {
        tag != "Fragment"
            && self.uses_babel_vdom_compat()
            && self
                .is_custom_element
                .is_some_and(|predicate| predicate(tag))
    }

    /// Whether an authored JSX identifier resolves to a lexical JavaScript binding.
    pub(crate) fn is_bound_jsx_identifier(&self, name: &JSXElementName<'_>) -> bool {
        let JSXElementName::IdentifierReference(reference) = name else {
            return false;
        };
        let Some(reference_id) = reference.reference_id.get() else {
            return false;
        };
        self.scoping
            .as_ref()
            .is_some_and(|scoping| scoping.has_binding(reference_id))
    }

    /// Collision-free helper binding for Babel's opt-in `transformOn` lowering.
    pub(crate) fn transform_on_helper(&self) -> Option<&str> {
        if self.babel_vdom_compat_active {
            self.transform_on_helper.as_deref()
        } else {
            None
        }
    }

    /// Collision-free helper binding for Babel's default object-slot check.
    pub(crate) fn object_slots_helper(&self) -> Option<&str> {
        if self.babel_vdom_compat_active {
            self.object_slots_helper.as_deref()
        } else {
            None
        }
    }
}
