//! Span-carrying SSR emission (Davinci P3-9, S4).
//!
//! SSR code is written in two ways: straight into the context's
//! [`EmitDocument`] (control flow, component calls) and through
//! template-literal parts, fragments that are coalesced and flushed later.
//! Both carry authored links: direct writes record into the document as they
//! happen, and a part's links are rebased when it is flushed. When no map is
//! requested nothing is recorded and every part stays link-free, so the
//! emitted bytes are identical either way (TS-11).

use vize_atelier_core::{DirectiveNode, codegen::document::EmitDocument};
use vize_s0::Span;

use super::{SsrCodegenContext, TemplatePart};

impl SsrCodegenContext<'_> {
    /// Whether this compile records a source map.
    pub(crate) fn spans_enabled(&self) -> bool {
        self.out.is_recording()
    }

    /// Write `text` directly, anchored at the authored byte `source`.
    pub(crate) fn push_mapped(&mut self, text: &str, source: u32) {
        self.out.push_mapped(text, source);
    }

    /// Write `text` directly, anchored at `source` when there is one.
    pub(crate) fn push_optionally_mapped(&mut self, text: &str, source: Option<u32>) {
        match source {
            Some(source) => self.push_mapped(text, source),
            None => self.push(text),
        }
    }

    /// Write `lead` unanchored, then `text` anchored at `source`.
    pub(crate) fn push_then_mapped(&mut self, lead: &str, text: &str, source: u32) {
        self.push(lead);
        self.push_mapped(text, source);
    }

    /// Write a spanned piece directly, rebasing its anchors.
    pub(crate) fn push_spanned(&mut self, piece: &EmitDocument) {
        self.out.push_spanned(piece);
    }

    /// Write an emitted expression authored at `span` directly.
    pub(crate) fn push_expression_text(&mut self, code: &str, span: Span) {
        self.out.push_expression(code, span, self.source);
    }

    /// An emitted expression authored at `span`, anchored when maps are on.
    pub(crate) fn spanned_expression(&self, code: &str, span: Span) -> EmitDocument {
        let mut piece = EmitDocument::default();
        if self.spans_enabled() {
            piece.push_expression(code, span, self.source);
        } else {
            piece.push_str(code);
        }
        piece
    }

    /// `prefix` + expression authored at `span` + `suffix`.
    pub(crate) fn spanned_wrap(
        &self,
        prefix: &str,
        code: &str,
        suffix: &str,
        span: Span,
    ) -> EmitDocument {
        let mut piece = EmitDocument::plain(prefix);
        piece.push_spanned(&self.spanned_expression(code, span));
        piece.push_str(suffix);
        piece
    }

    /// Push a template-literal part wrapping an expression authored at `span`.
    pub(crate) fn push_wrapped_expression_part(
        &mut self,
        prefix: &str,
        code: &str,
        suffix: &str,
        span: Span,
    ) {
        let piece = self.spanned_wrap(prefix, code, suffix, span);
        self.push_string_part_dynamic_spanned(piece);
    }

    /// Push `_ssrRenderAttr("name", exp)` for a statically named `v-bind`.
    pub(crate) fn push_bound_attr_part(&mut self, name: &str, dir: &DirectiveNode<'_>, exp: &str) {
        let mut piece = EmitDocument::plain("_ssrRenderAttr(\"");
        match (&dir.arg, self.spans_enabled()) {
            (Some(arg), true) => piece.push_mapped(name, arg.loc().span.start),
            _ => piece.push_str(name),
        }
        piece.push_str("\", ");
        let span = dir.exp.as_ref().map_or(Span::new(0, 0), |e| e.loc().span);
        piece.push_spanned(&self.spanned_expression(exp, span));
        piece.push_str(")");
        self.push_string_part_dynamic_spanned(piece);
    }

    /// Append static template-literal text copied from the authored byte
    /// `source`, coalescing with a preceding static part.
    pub(crate) fn push_string_part_static_mapped(&mut self, text: &str, source: u32) {
        if !self.spans_enabled() {
            self.push_string_part_static(text);
            return;
        }
        if let Some(TemplatePart::Static(last)) = self.current_template_parts.last_mut() {
            last.push_mapped(text, source);
        } else {
            self.current_template_parts
                .push(TemplatePart::Static(EmitDocument::mapped(text, source)));
        }
    }

    /// Append a dynamic template-literal part that carries its own anchors.
    pub(crate) fn push_string_part_dynamic_spanned(&mut self, piece: EmitDocument) {
        self.current_template_parts
            .push(TemplatePart::Dynamic(piece));
    }

    /// Write a static template-literal part, escaping `` ` `` and `${`, and
    /// rebasing its links onto the escaped output.
    pub(super) fn push_template_static(&mut self, part: &EmitDocument) {
        self.out.push_escaped(part, &[("`", "\\`"), ("${", "\\${")]);
    }
}
