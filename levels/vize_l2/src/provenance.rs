//! Lowering provenance: before/after pairs per lowering decision,
//! surviving failure (P2-8).
//!
//! # The imported practice
//!
//! Lean 4's `InfoTree` (`docs/davinci/prior-art-toolchains.md`): every
//! elaboration step records which rule produced what, from which syntax,
//! with before/after pairs for macro expansion - **and partial results are
//! kept when elaboration fails**, so hover works in broken code. The
//! Levels form: a [`ProvenanceRecord`] per lowering decision carrying
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
//!   exactly as diagnostics do. On a broken SFC the kept L2 fragments plus
//!   these records are what the LSP and the stage view stay live on.
//!
//! # Materialization policy
//!
//! A record is built only for a consumer that reads it. The consumer
//! chooses the [`ProvenanceSink`] its producers write into: the record
//! vector keeps every decision with owned text, while [`NoProvenance`]
//! keeps nothing, so a compile nobody inspects never formats a record.
//! The choice is a type, so a product cannot read records nobody kept.

use alloc::vec::Vec;

use vize_l0::id::NodeId;
use vize_l0::{Span, String};

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

/// Where producers put their decisions, chosen by the consumer.
pub trait ProvenanceSink {
    /// The kept records, or `None` when no consumer asked for any.
    fn kept(&mut self) -> Option<&mut Vec<ProvenanceRecord>>;

    /// Append one decision. `build` runs only when records are kept.
    #[inline]
    fn record(&mut self, build: impl FnOnce() -> ProvenanceRecord) {
        if let Some(records) = self.kept() {
            records.push(build());
        }
    }
}

impl ProvenanceSink for Vec<ProvenanceRecord> {
    #[inline]
    fn kept(&mut self) -> Option<&mut Vec<ProvenanceRecord>> {
        Some(self)
    }
}

/// The sink of a compile no consumer inspects: it keeps no record.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoProvenance;

impl ProvenanceSink for NoProvenance {
    #[inline]
    fn kept(&mut self) -> Option<&mut Vec<ProvenanceRecord>> {
        None
    }
}

/// A 64-bit figure of a pointer-containing struct, so it carries the
/// `vize_relief` guard (see `crate::op`): three owned strings (24 each) +
/// span (8) + the 4-byte id option, padded.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<ProvenanceRecord>() == 88);

#[cfg(test)]
mod tests {
    use super::{NoProvenance, ProvenanceRecord, ProvenanceSink};
    use alloc::vec::Vec;
    use core::cell::Cell;
    use vize_l0::id::NodeId;
    use vize_l0::{Span, String};

    fn decision(built: &Cell<u32>) -> impl FnOnce() -> ProvenanceRecord + '_ {
        || {
            built.set(built.get() + 1);
            ProvenanceRecord {
                rule: String::from("lower.text"),
                node: NodeId::from_index(0),
                before: String::from("t"),
                after: String::from("ui.text"),
                span: Span::new(0, 1),
            }
        }
    }

    #[test]
    fn only_a_kept_sink_builds_records() {
        let built = Cell::new(0);
        let mut none = NoProvenance;
        none.record(decision(&built));
        assert_eq!(built.get(), 0);
        assert_eq!(none.kept(), None);

        let mut kept = Vec::new();
        kept.record(decision(&built));
        assert_eq!(built.get(), 1);
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].after.as_str(), "ui.text");
    }

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
