//! Owned compound-text facts shared by canonical L2 producers and consumers.
//!
//! Rows preserve authored ranges and consumed rendered/expression text.
//! Whitespace, comment and dialect legalization policy stays with the producer;
//! the rebuild is a display spelling, never executable JavaScript or admission.
//! This module does not claim that transitional tables belong to Artifact yet.

use alloc::vec::Vec as StdVec;
use vize_l0::id::NodeId;
use vize_l0::side_table::SideTable;
use vize_l0::{Span, String};

/// One part of a merged run, owned (the fact crosses compile boundaries
/// with its artifact, P1-11).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextPart {
    /// The part's rendered text: condensed content for a static part,
    /// the trimmed expression for a dynamic one.
    pub text: String,
    /// The authored range (the raw token for a static part, the whole
    /// `{{ … }}` for a dynamic one).
    pub span: Span,
    /// Whether the part renders an expression.
    pub dynamic: bool,
    /// Whether a dynamic part is an unescaped raw-HTML interpolation.
    ///
    /// This is always false for static parts.
    pub raw_html: bool,
}

/// The recorded parts of one merged mixed run, keyed by the compound
/// `ui.interpolation` op's id. These are the consumed view: a producer
/// validates every entry against the owning op span when it attaches the fact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextParts {
    /// The parts, in document order.
    pub parts: StdVec<TextPart>,
}

/// One compound op's consumed view: the merged run's parts, validated at
/// the lowering/mutation sites before DOM realization compiles from it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextFacts {
    /// The parts, in document order ([`TextPart`]'s field docs).
    pub parts: StdVec<TextPart>,
}

impl TextParts {
    /// Number of dynamic expression parts.
    #[doc(hidden)]
    #[must_use]
    pub fn dynamic_count(&self) -> usize {
        self.parts.iter().filter(|part| part.dynamic).count()
    }

    /// Validate the compound laws while the owning op span is available.
    ///
    /// A failure here is a compiler bug, not a user diagnostic: every
    /// mixed run must hold at least two parts with one dynamic member,
    /// static neighbours are fused, parts tile the owning op span, and
    /// the canonical rebuild is the spellable compound source.
    ///
    /// # Panics
    ///
    /// Panics when the producer violates the compound shape, tiling or rebuild laws.
    #[doc(hidden)]
    pub fn assert_compound_laws(&self, id: NodeId, span: Span, source: &str) {
        let dynamic = self.dynamic_count();
        assert!(
            self.parts.len() >= 2 && dynamic >= 1,
            "compound law broken: op {id} records {} parts ({dynamic} dynamic) — a lone node never merges",
            self.parts.len(),
        );
        assert!(
            self.parts
                .windows(2)
                .all(|pair| pair.iter().any(|part| part.dynamic)),
            "compound law broken: op {id} holds adjacent static parts — the lowering fuses them",
        );
        let mut cursor = span.start;
        for part in &self.parts {
            assert!(
                part.dynamic || !part.raw_html,
                "compound law broken: op {id} marks a static part as raw HTML",
            );
            assert!(
                part.span.start == cursor,
                "compound law broken: op {id}'s parts do not tile its span (gap at {cursor})",
            );
            cursor = part.span.end;
        }
        assert!(
            cursor == span.end,
            "compound law broken: op {id}'s parts end at {cursor}, its span at {}",
            span.end,
        );
        let rebuilt = rebuild_source(&self.parts);
        assert!(
            rebuilt.as_str() == source,
            "compound law broken: op {id}'s source {:?} does not rebuild from its parts {:?}",
            source,
            rebuilt,
        );
    }
}

const _: () = {
    const fn assert_owned<T: 'static>() {}
    assert_owned::<TextPart>();
    assert_owned::<TextParts>();
    assert_owned::<TextFacts>();
};

/// 64-bit footprints, guarded like every node-size assert (the wasm32
/// lane is 32-bit).
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<TextPart>() == 40);
    assert!(core::mem::size_of::<TextParts>() == 24);
    assert!(core::mem::size_of::<TextFacts>() == 24);
};

/// The rebuild rule: the canonical template spelling of a merged run —
/// static parts verbatim, escaped dynamic parts as `{{ <text> }}`, and
/// raw-HTML dynamic parts as `{{{ <text> }}}`. One home,
/// used by the lowering to mint [`OpaqueExpr::source`] and by the pass
/// to re-derive it (`pass::text`), so the two can only disagree when one
/// of them changed.
///
/// [`OpaqueExpr::source`]: crate::expr::OpaqueExpr::source
pub fn rebuild_source(parts: &[TextPart]) -> String {
    // Escaped delimiters add six bytes and raw delimiters eight; one
    // allocation suffices.
    let capacity = (parts.iter())
        .map(|part| {
            part.text.len()
                + if part.raw_html {
                    8
                } else if part.dynamic {
                    6
                } else {
                    0
                }
        })
        .sum();
    let mut out = String::with_capacity(capacity);
    for part in parts {
        if part.raw_html {
            out.push_str("{{{ ");
            out.push_str(part.text.as_str());
            out.push_str(" }}}");
        } else if part.dynamic {
            out.push_str("{{ ");
            out.push_str(part.text.as_str());
            out.push_str(" }}");
        } else {
            out.push_str(part.text.as_str());
        }
    }
    out
}

/// Publish already-validated compound text parts without walking an op tree.
///
/// This copies the consumed view beside the artifact, retaining every node id
/// and owned part in the same order. Producer validation and dialect policy
/// remain with the caller.
#[must_use]
pub fn publish_text_facts(texts: &SideTable<TextParts>) -> SideTable<TextFacts> {
    let mut facts = SideTable::with_capacity(texts.len());
    for (id, parts) in texts.iter() {
        facts.insert(
            id,
            TextFacts {
                parts: parts.parts.clone(),
            },
        );
    }
    facts
}
