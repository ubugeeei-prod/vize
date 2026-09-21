//! Span-carrying SSR emission (Davinci P3-9, S4).
//!
//! SSR code is written in two ways: straight into the code buffer (control
//! flow, component calls) and through template-literal parts that are
//! coalesced and flushed later. Both carry authored spans: direct writes
//! record into the shared [`SourceMapBuilder`] as they happen, and parts are
//! [`SpannedText`] whose anchors are rebased when the part is flushed. When no
//! map is requested nothing is recorded and every part stays anchor-free, so
//! the emitted bytes are identical either way (TS-11).

use vize_atelier_core::{
    DirectiveNode,
    codegen::spanned::{SpannedText, expression_anchors},
};
use vize_s0::Span;

use super::{SsrCodegenContext, TemplatePart};

impl SsrCodegenContext<'_> {
    /// Whether this compile records a source map.
    pub(crate) fn spans_enabled(&self) -> bool {
        self.map.is_some()
    }

    /// Anchor the next directly written byte at the authored byte `source`.
    pub(crate) fn record_anchor(&mut self, source: u32) {
        if let Some(map) = self.map.as_mut() {
            map.add_raw(self.code.len(), source);
        }
    }

    /// Write `text` directly, anchored at the authored byte `source`.
    pub(crate) fn push_mapped(&mut self, text: &str, source: u32) {
        self.record_anchor(source);
        self.push(text);
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
    pub(crate) fn push_spanned(&mut self, piece: &SpannedText) {
        if let Some(map) = self.map.as_mut() {
            map.add_anchors(self.code.len(), piece.anchors());
        }
        self.push(piece.as_str());
    }

    /// Write an emitted expression authored at `span` directly.
    pub(crate) fn push_expression_text(&mut self, code: &str, span: Span) {
        if let Some(map) = self.map.as_mut() {
            map.add_anchors(
                self.code.len(),
                &expression_anchors(code, span, self.source),
            );
        }
        self.push(code);
    }

    /// An emitted expression authored at `span`, anchored when maps are on.
    pub(crate) fn spanned_expression(&self, code: &str, span: Span) -> SpannedText {
        let mut piece = SpannedText::default();
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
    ) -> SpannedText {
        let mut piece = SpannedText::plain(prefix);
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
        let mut piece = SpannedText::plain("_ssrRenderAttr(\"");
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
                .push(TemplatePart::Static(SpannedText::mapped(text, source)));
        }
    }

    /// Append a dynamic template-literal part that carries its own anchors.
    pub(crate) fn push_string_part_dynamic_spanned(&mut self, piece: SpannedText) {
        self.current_template_parts
            .push(TemplatePart::Dynamic(piece));
    }

    /// Write a static template-literal part, escaping `` ` `` and `${`, and
    /// rebasing its anchors onto the escaped output.
    pub(super) fn push_template_static(&mut self, part: &SpannedText) {
        let bytes = part.as_str().as_bytes();
        let mut anchors = part.anchors().iter().peekable();
        let mut start = 0;
        let mut index = 0;

        while index <= bytes.len() {
            while let Some(anchor) = anchors.next_if(|anchor| anchor.offset <= index) {
                self.code.extend_from_slice(&bytes[start..index]);
                start = index;
                if let Some(map) = self.map.as_mut() {
                    map.add_raw(self.code.len(), anchor.source);
                }
            }
            if index == bytes.len() {
                break;
            }
            match bytes[index] {
                b'`' => {
                    self.code.extend_from_slice(&bytes[start..index]);
                    self.code.extend_from_slice(b"\\`");
                    index += 1;
                    start = index;
                }
                b'$' if index + 1 < bytes.len() && bytes[index + 1] == b'{' => {
                    self.code.extend_from_slice(&bytes[start..index]);
                    self.code.extend_from_slice(b"\\${");
                    index += 2;
                    start = index;
                }
                _ => {
                    index += 1;
                }
            }
        }

        self.code.extend_from_slice(&bytes[start..]);
    }
}
