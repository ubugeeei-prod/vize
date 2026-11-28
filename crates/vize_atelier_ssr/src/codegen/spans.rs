//! Span-carrying SSR emission (Davinci P3-9, L4).
//!
//! SSR code is written in two ways: straight into the context's
//! [`EmitDocument`] (control flow, component calls) and through
//! template-literal parts, fragments that are coalesced and flushed later.
//! Both carry authored links: direct writes record into the document as they
//! happen, and a part's links are rebased when it is flushed. When no map is
//! requested nothing is recorded and every part stays link-free, so the
//! emitted bytes are identical either way (TS-11).

use vize_atelier_core::{DirectiveNode, codegen::document::EmitDocument};
use vize_l0::Span;
use vize_l0::general::quoted_name::{escaped_name_contents, name_needs_escape};

use super::{SsrCodegenContext, TemplatePart};

/// Push the string-literal contents of a bound attribute name that needs
/// escaping, mapped at `start` when there is one.
#[cold]
#[inline(never)]
pub(crate) fn push_escaped_bound_name(piece: &mut EmitDocument, name: &str, start: Option<u32>) {
    let contents = escaped_name_contents(name);
    match start {
        Some(start) => piece.push_mapped(&contents, start),
        None => piece.push_str(&contents),
    }
}

/// What a static part needs to stay inert inside a template literal. The
/// backslash comes first: patterns are tried in order at each byte.
const TEMPLATE_STATIC_ESCAPES: [(&str, &str); 3] = [("\\", "\\\\"), ("`", "\\`"), ("${", "\\${")];

impl SsrCodegenContext<'_> {
    /// Whether this compile records a source map.
    pub(crate) fn spans_enabled(&self) -> bool {
        self.out.is_recording()
    }

    /// Write `text` directly, anchored at the authored byte `source`.
    pub(crate) fn push_mapped(&mut self, text: &str, source: u32) {
        self.out.push_mapped(text, source);
    }

    sfc_map_copy! {
        plain {}
        composed {
            /// Write `text` directly, anchored at the authored byte `source`
            /// while a map is recorded.
            #[cold]
            #[inline(never)]
            pub(crate) fn push_anchored(&mut self, text: &str, source: u32) {
                if self.spans_enabled() {
                    self.out.anchor(source);
                }
                self.push(text);
            }
        }
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
        let mut piece = EmitDocument::with_capacity(prefix.len() + code.len() + suffix.len(), true);
        piece.push_str(prefix);
        if self.spans_enabled() {
            piece.push_expression(code, span, self.source);
        } else {
            piece.push_str(code);
        }
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
        if name_needs_escape(name) {
            let start = dir.arg.as_ref().map(|arg| arg.loc().span.start);
            push_escaped_bound_name(&mut piece, name, start.filter(|_| self.spans_enabled()));
        } else {
            match (&dir.arg, self.spans_enabled()) {
                (Some(arg), true) => piece.push_mapped(name, arg.loc().span.start),
                _ => piece.push_str(name),
            }
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

    /// Write a static template-literal part, escaping `\`, `` ` `` and `${`,
    /// and rebasing its links onto the escaped output. Static parts are
    /// authored HTML, never JavaScript escapes, so a backslash is literal
    /// text: left alone it would start an escape sequence, and one in front
    /// of a backtick would cancel that backtick's own escape.
    pub(super) fn push_template_static(&mut self, part: &EmitDocument) {
        self.out.push_escaped(part, &TEMPLATE_STATIC_ESCAPES);
    }
}

#[cfg(test)]
mod tests {
    use super::super::SsrCodegenContext;
    use crate::{SsrCompilerExperimentalOptions, SsrCompilerOptions};
    use vize_atelier_core::codegen::document::EmitDocument;
    use vize_l0::{Allocator, Span};

    #[test]
    fn wrapped_fragments_keep_complete_text_links_and_maps() {
        let allocator = Allocator::default();
        let options = SsrCompilerOptions::default();
        let source = "<p>{{ a + 雪 }}</p>";
        for source_map in [false, true] {
            let ctx = SsrCodegenContext::new_with_experimental_options(
                &allocator,
                &options,
                source,
                SsrCompilerExperimentalOptions {
                    source_map,
                    ..SsrCompilerExperimentalOptions::default()
                },
            );
            for (code, span) in [
                ("_ctx.a + _ctx.雪", Span::new(6, 13)),
                ("1 + _ctx.雪", Span::new(6, 13)),
                ("", Span::new(6, 13)),
                ("_ctx.a", Span::new(0, 0)),
            ] {
                for (prefix, suffix) in [
                    ("", ""),
                    ("_ssrInterpolate(", ")"),
                    ("xxxxxxxxxxxxxxxxxxxxxxxx", ")"),
                    ("xxxxxxxxxxxxxxxxxxxxxxxxx", "雪"),
                ] {
                    // Retained two-document composition is the complete reference,
                    // including link names/segment flags and serialized source maps.
                    let mut expression = EmitDocument::default();
                    if source_map {
                        expression.push_expression(code, span, source);
                    } else {
                        expression.push_str(code);
                    }
                    let mut expected = EmitDocument::plain(prefix);
                    expected.push_spanned(&expression);
                    expected.push_str(suffix);
                    let actual = ctx.spanned_wrap(prefix, code, suffix, span);
                    assert_eq!(actual, expected);
                    assert_eq!(actual.as_str(), [prefix, code, suffix].concat());
                    assert_eq!(
                        actual.source_map("wrapped.vue", source),
                        expected.source_map("wrapped.vue", source),
                    );
                }
            }
        }
    }
}
