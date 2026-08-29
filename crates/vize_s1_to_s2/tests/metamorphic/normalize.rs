//! The declared normalization of the TS-21 oracle.
//!
//! The oracle compares S2 folios of original and mutant as **full
//! normalized artifacts** — whole-document `Display`-mode prints, whole
//! value equality, never a substring (TS-13). `Display` is the
//! `folio-format.md` elision (every ` @s:e` span, line tails and
//! expression payloads alike, and nothing else), which is exactly the
//! quotient a byte-shifting mutation needs; the `ops=` header is
//! recomputed by the printer from the normalized tree. On top of that,
//! each mutator declares the *minimal* structural quotient its
//! justification licenses (`mutators.rs` for the arguments):
//!
//! - `sort_attrs` (reorder only): `attr` lines under one owner sort by
//!   name, then value — the canonical quotient by the permutation the
//!   mutator induces.
//!
//! The P2-15 `merge_text` and `condense` rules are **gone — the P2-9
//! ratchet**: installment 4 absorbed the text merge and the condense
//! into the lowering itself (`crates/vize_s1_to_s2/src/lower/text.rs`),
//! so split/merge and whitespace mutants lower to the original's
//! artifact with no declared rule at all, exactly as the P2-15 record's
//! "left open" clause demanded ("turn them off and watch the suite stay
//! green"). Their sensitivity pins moved with them (`main.rs`): the
//! same real differences must survive the *lowering's* canonical
//! operations, no normalization involved.
//!
//! A normalization applied to both sides can only mask differences,
//! never invent one; the sensitivity pins in `main.rs` prove the
//! remaining quotient does not collapse real differences.

use vize_davinci::folio::{Folio, FolioMode};
use vize_s0::String;
use vize_s2::folio::{DisegnoFolio, FolioAttribute, FolioBinding, FolioOp};

/// Which structural rules a comparison applies (spans always elide).
#[derive(Debug, Clone, Copy, Default)]
pub struct Normalization {
    pub sort_attrs: bool,
}

impl Normalization {
    pub const REORDER: Self = Self { sort_attrs: true };
    /// Wrap, split/merge and whitespace mutants compare with no
    /// structural rule at all (span elision only) since the P2-9
    /// ratchet.
    pub const NONE: Self = Self { sort_attrs: false };
}

/// The full normalized artifact: apply `rules`, print `Display`.
pub fn normalized_display(folio: &DisegnoFolio, rules: Normalization) -> String {
    let mut folio = folio.clone();
    if rules.sort_attrs {
        walk_region(&mut folio.ops);
    }
    folio.print_to_string(FolioMode::Display)
}

fn walk_region(ops: &mut [FolioOp]) {
    for op in ops.iter_mut() {
        match op {
            FolioOp::Element(element) => {
                sort_attrs(&mut element.attributes);
                sort_binding_attrs(&mut element.bindings);
                walk_region(&mut element.children);
            }
            FolioOp::Component(component) => {
                sort_attrs(&mut component.attributes);
                sort_binding_attrs(&mut component.bindings);
                walk_region(&mut component.children);
            }
            FolioOp::If(if_op) => {
                for branch in &mut if_op.branches {
                    walk_region(&mut branch.ops);
                }
            }
            FolioOp::For(for_op) => walk_region(&mut for_op.ops),
            FolioOp::Slot(slot) => {
                // The outlet's props surface (P2-9 series 5): static
                // slot props sort under the same reorder quotient as
                // element attributes.
                sort_attrs(&mut slot.attributes);
                sort_binding_attrs(&mut slot.bindings);
                walk_region(&mut slot.fallback);
            }
            FolioOp::Text(_) | FolioOp::Interpolation(_) => {}
        }
    }
}

fn sort_attrs(attrs: &mut [FolioAttribute]) {
    attrs.sort_by(|a, b| {
        (a.name.as_str(), a.value.as_deref()).cmp(&(b.name.as_str(), b.value.as_deref()))
    });
}

fn sort_binding_attrs(bindings: &mut [FolioBinding]) {
    for binding in bindings {
        if let FolioBinding::Model(model) = binding {
            sort_attrs(&mut model.attributes);
        }
    }
}
