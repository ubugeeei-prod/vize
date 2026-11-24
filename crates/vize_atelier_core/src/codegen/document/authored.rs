//! Anchors for text an emitter writes without a location of its own.
//!
//! A `v-for` alias is the case: its node has no span. Each entry point here
//! makes the one recording check its plain counterpart makes and does all of
//! its work behind it, so a document that records no map does what it did.

use vize_l0::Span;

use super::{EmitDocument, SpanLink, expression_links};

/// Where `authored` starts in `source_text`, when it is a slice of it: the
/// same bytes at the same addresses. A node holds such a slice until a
/// transform rewrites its text into an arena copy, which is not one.
pub fn authored_offset(authored: &str, source_text: &str) -> Option<u32> {
    let start = (authored.as_ptr() as usize).checked_sub(source_text.as_ptr() as usize)?;
    let inside = !authored.is_empty() && start + authored.len() <= source_text.len();
    inside.then(|| u32::try_from(start).ok()).flatten()
}

impl EmitDocument {
    /// [`Self::push_expression`], then `follow` with the document and the
    /// offset the expression ends at. `follow` runs only while a map is
    /// recorded, inside the check the expression already makes: it is where
    /// an emitter anchors the text it writes next with no check of its own.
    #[inline]
    pub fn push_expression_then(
        &mut self,
        code: &str,
        span: Span,
        source_text: &str,
        follow: impl FnOnce(&mut Self, u32),
    ) {
        if self.recording {
            let base = self.cursor();
            self.links.extend(
                expression_links(code, span, source_text)
                    .iter()
                    .map(|link| link.rebased(base)),
            );
            follow(self, base + code.len() as u32);
        }
        self.text.push_str(code);
    }

    /// Append `code`, written from the node text `authored`. Written
    /// unchanged from a slice of `source_text`, it maps to where that slice
    /// was authored.
    #[inline]
    pub fn push_authored(&mut self, code: &str, authored: &str, source_text: &str) {
        if self.recording {
            self.anchor_authored(code, authored, source_text);
        }
        self.text.push_str(code);
    }

    #[cold]
    #[inline(never)]
    fn anchor_authored(&mut self, code: &str, authored: &str, source_text: &str) {
        if let Some(source) = authored_offset(authored, source_text).filter(|_| code == authored) {
            self.anchor(source);
        }
    }

    /// Anchor the byte that will be written at `generated`, an offset this
    /// document has not reached yet, at the authored byte `source`.
    #[cold]
    #[inline(never)]
    pub fn anchor_ahead(&mut self, generated: u32, source: u32) {
        self.record(
            Span::new(generated, generated),
            Span::new(source, source),
            None,
        );
    }
}

pub(super) fn append_member_links(links: &mut Vec<SpanLink>, text: &str, code: &str, start: u32) {
    vize_l0::source_range::visit_static_member_spans(text, code, start, |generated, authored| {
        links.push(SpanLink {
            generated,
            authored,
            name: None,
            segment: true,
        });
    });
}
