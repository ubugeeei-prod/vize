//! Owned compound-text facts shared by canonical L2 producers and consumers.
//!
//! Rows preserve authored ranges and consumed rendered/expression text.
//! Whitespace, comment and dialect legalization policy stays with the producer;
//! the rebuild is a display spelling, never executable JavaScript or admission.
//! The table rides on the canonical artifact (`L2TemplateParts::texts`).

use alloc::vec::Vec as StdVec;
use core::mem::MaybeUninit;
use vize_l0::id::NodeId;
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
};

/// 64-bit footprints, guarded like every node-size assert (the wasm32
/// lane is 32-bit).
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<TextPart>() == 40);
    assert!(core::mem::size_of::<TextParts>() == 24);
};

/// The rebuild rule: the canonical template spelling of a merged run —
/// static parts verbatim, escaped dynamic parts as `{{ <text> }}`, and
/// raw-HTML dynamic parts as `{{{ <text> }}}`. One home,
/// used by the lowering to mint [`OpaqueExpr::source`] and by legalization
/// to refresh it, so the two can only disagree when one of them changed.
///
/// [`OpaqueExpr::source`]: crate::expr::OpaqueExpr::source
pub fn rebuild_source(parts: &[TextPart]) -> String {
    let capacity = parts
        .iter()
        .map(|part| {
            let (open, close) = delimiters(part);
            open.len() + part.text.len() + close.len()
        })
        .sum();
    let mut out = String::with_capacity(capacity);
    // One pass of plain copies into the pre-sized buffer, so the cost never
    // depends on whether LLVM inlines `CompactString::push_str` at six call
    // sites (it stopped doing so when unrelated code left this crate).
    //
    // SAFETY: `spare` is `out`'s uninitialized tail (`out` is empty, so it
    // starts at byte 0). `fill` initializes a prefix of it and returns its
    // length, which is at most `spare.len()`. That prefix is a concatenation
    // of whole `&str` pieces, so it is valid UTF-8. An inline string's tag byte
    // is overwritten only when the prefix fills the inline capacity, and then
    // it is the final byte of that complete, valid UTF-8 string, which is what
    // `spare_capacity_mut` allows.
    let written = fill(unsafe { out.spare_capacity_mut() }, parts);
    // SAFETY: `fill` initialized exactly `written <= capacity()` bytes of
    // valid UTF-8 (see above), and nothing touched `out` in between.
    unsafe { out.set_len(written) };
    out
}

/// The delimiters around a part's text in the rebuild.
fn delimiters(part: &TextPart) -> (&'static str, &'static str) {
    if part.raw_html {
        ("{{{ ", " }}}")
    } else if part.dynamic {
        ("{{ ", " }}")
    } else {
        ("", "")
    }
}

/// Copy the rebuild of `parts` into `spare` and return the number of bytes
/// initialized. It copies whole pieces only and stops at the first piece that
/// does not fit (it always fits: the caller sized `spare` from the same parts).
fn fill(spare: &mut [MaybeUninit<u8>], parts: &[TextPart]) -> usize {
    let mut written = 0;
    // A macro, not a helper, so no copy waits on an inlining decision.
    macro_rules! put {
        ($piece:expr) => {{
            let bytes: &[u8] = $piece.as_bytes();
            let end = written + bytes.len();
            let Some(slot) = spare.get_mut(written..end) else {
                return written;
            };
            slot.write_copy_of_slice(bytes);
            written = end;
        }};
    }
    for part in parts {
        let (open, close) = delimiters(part);
        put!(open);
        put!(part.text.as_str());
        put!(close);
    }
    written
}

#[cfg(test)]
mod kani;
