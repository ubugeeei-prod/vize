//! Links only a map composed into a Vue file carries.
//!
//! The SFC assembly composes a template's render map into the component's,
//! and that map links more than a template's own: setup-binding reads at
//! their tags, and text an emitter writes without a location of its own (a
//! `v-for` alias, whose node has no span). Emitters write those links only
//! while the document [`composes`](EmitDocument::composes), and each check
//! here is the only work a document that records no map does.

use vize_l0::Span;

use super::EmitDocument;

/// Where `authored` starts in `source_text`, when it is a slice of it: the
/// same bytes at the same addresses. A node holds such a slice until a
/// transform rewrites its text into an arena copy, which is not one.
pub fn authored_offset(authored: &str, source_text: &str) -> Option<u32> {
    let start = (authored.as_ptr() as usize).checked_sub(source_text.as_ptr() as usize)?;
    let inside = !authored.is_empty() && start + authored.len() <= source_text.len();
    inside.then(|| u32::try_from(start).ok()).flatten()
}

impl EmitDocument {
    /// Whether this document's links are composed into a Vue file's map.
    /// Only a recording document composes.
    #[inline(always)]
    pub fn composes(&self) -> bool {
        self.composes
    }

    /// Compose this document's links into a Vue file's map, when it records.
    pub fn compose(&mut self) {
        self.composes = self.recording;
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
