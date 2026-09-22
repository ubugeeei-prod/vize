//! Links over already-written text and generated↔generated edges (P4-5b).
//!
//! A projection target writes one authored construct as several pushes, some
//! of which carry their own links, and then links the whole run to the
//! authored construct ([`EmitDocument::link_since`]). It also relates two
//! generated ranges that stand for one authored binding
//! ([`EmitDocument::push_edge`]); edges move with their fragment exactly like
//! links do and never become source-map segments.

use vize_s0::Span;

use super::EmitDocument;

/// A generated↔generated edge: the two generated ranges and a
/// target-defined kind.
pub type GeneratedEdge = (Span, Span, u8);

/// `edge` with both generated ranges moved through `offset`.
pub(super) fn rebase_edge(edge: &GeneratedEdge, offset: impl Fn(u32) -> u32) -> GeneratedEdge {
    let (from, to, kind) = *edge;
    (
        Span::new(offset(from.start), offset(from.end)),
        Span::new(offset(to.start), offset(to.end)),
        kind,
    )
}

impl EmitDocument {
    /// Link the text written since `generated_start` to the authored range
    /// `authored`. Links recorded inside that text stay as they are; the new
    /// link encloses them.
    pub fn link_since(&mut self, generated_start: u32, authored: Span) {
        self.link_since_inner(generated_start, authored, None);
    }

    /// [`Self::link_since`] for an authored range that spells the symbol
    /// `name`.
    pub fn link_since_named(&mut self, generated_start: u32, authored: Span, name: &str) {
        self.link_since_inner(generated_start, authored, Some(name));
    }

    fn link_since_inner(&mut self, generated_start: u32, authored: Span, name: Option<&str>) {
        let end = self.cursor();
        debug_assert!(generated_start <= end, "link_since starts past the cursor");
        self.record(Span::new(generated_start.min(end), end), authored, name);
    }

    /// Relate the generated ranges `from` and `to` with a target-defined
    /// `kind`. Recorded only while the document records.
    pub fn push_edge(&mut self, from: Span, to: Span, kind: u8) {
        if self.recording {
            self.edges.push((from, to, kind));
        }
    }

    /// Every generated↔generated edge, in recording order.
    pub fn edges(&self) -> &[GeneratedEdge] {
        &self.edges
    }
}

#[cfg(test)]
#[path = "links_tests.rs"]
mod tests;
