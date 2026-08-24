//! Lowering provenance: before/after pairs per lowering decision,
//! surviving failure (P2-8).
//!
//! # The imported practice
//!
//! Lean 4's `InfoTree` (`davinci-road/prior-art-toolchains.md`): every
//! elaboration step records which rule produced what, from which syntax,
//! with before/after pairs for macro expansion - **and partial results are
//! kept when elaboration fails**, so hover works in broken code. The
//! Davinci form: a [`ProvenanceRecord`] per lowering decision carrying
//! `(rule name, produced node, before, after, span)`, in decision order.
//!
//! # The survival law
//!
//! Provenance survives failure by construction, in both senses:
//!
//! - **Per decision:** a decision that produced no op (a dropped comment,
//!   a directive the op family cannot carry yet, a grammar violation) is
//!   still a record - [`ProvenanceRecord::node`] is `None` and
//!   [`ProvenanceRecord::after`] is empty. Nothing the lowering saw is
//!   unaccounted for.
//! - **Per artifact:** every field is owned and `'static` (asserted
//!   below, the P1-11 contract), so the records outlive the compile arena
//!   exactly as diagnostics do. On a broken SFC the kept S2 fragments plus
//!   these records are what the LSP and Spolvero stay live on.
//!
//! # Materialization policy
//!
//! P2-8 materializes provenance **fully** - every decision, owned text.
//! That is deliberate and deliberately not the last word: the InfoTree
//! anti-lesson is memory weight, and whether the fused CLI walk runs with
//! provenance off or ring-buffered is P2-12b's measurement (the P2-3
//! record already scopes it there). This type is the record's shape, not
//! its retention policy.

use vize_davinci::id::NodeId;
use vize_s0::{Span, String};

/// One lowering decision: what rule fired, on what, producing what.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvenanceRecord {
    /// The deciding rule, by name (`lower.element`, `defer.v-bind`,
    /// `drop.comment`, ...). A name rather than an index so out-of-tree
    /// producers get first-class provenance (`architecture.md`).
    pub rule: String,
    /// The op the decision produced (dense page-order id), or `None` when
    /// the decision produced no op - the failure half of the survival law.
    pub node: Option<NodeId>,
    /// The authored text the decision consumed.
    pub before: String,
    /// The produced form, in op-mnemonic vocabulary (`ui.for source=js
    /// value=js`); empty when nothing was produced.
    pub after: String,
    /// The authored range of `before`.
    pub span: Span,
}

/// Provenance crosses compile boundaries with its artifact (P1-11; the
/// same enforcement `Diagnostic` carries).
const _: () = {
    const fn assert_owned<T: 'static>() {}
    assert_owned::<ProvenanceRecord>();
};

/// A 64-bit figure of a pointer-containing struct, so it carries the
/// `vize_relief` guard (see `crate::op`): three owned strings (24 each) +
/// span (8) + the 4-byte id option, padded.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<ProvenanceRecord>() == 88);

#[cfg(test)]
mod tests {
    use super::ProvenanceRecord;
    use vize_davinci::id::NodeId;
    use vize_s0::{Span, String};

    #[test]
    fn a_producing_decision_links_its_op() {
        let record = ProvenanceRecord {
            rule: String::from("lower.for"),
            node: NodeId::from_index(3),
            before: String::from("item of items"),
            after: String::from("ui.for source=js value=js"),
            span: Span::new(12, 25),
        };
        assert_eq!(record.node, NodeId::from_index(3));
        assert_eq!(record.before.as_str(), "item of items");
        assert!(!record.after.is_empty());
    }

    #[test]
    fn a_failed_decision_is_still_a_record() {
        // The survival law's failure half: no op, empty `after`, and the
        // record still names the rule, the text, and the span.
        let record = ProvenanceRecord {
            rule: String::from("defer.v-bind"),
            node: None,
            before: String::from(":id=\"x\""),
            after: String::default(),
            span: Span::new(5, 12),
        };
        assert_eq!(record.node, None);
        assert!(record.after.is_empty());
        assert_eq!(record.rule.as_str(), "defer.v-bind");
    }
}
