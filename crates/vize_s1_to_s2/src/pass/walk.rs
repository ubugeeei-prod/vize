//! The page-order re-derivation walk the S2 passes share.
//!
//! S2 ids are positional (dense page order: op line, attached bindings,
//! then children — `folio-format.md`, "Node numbering"), so a pass that
//! keys facts re-derives them by walking in print order. The numbering
//! law has exactly one home here: a second walk with its own skip
//! arithmetic is the drift the P1-8 scanner split taught, and the
//! per-pass count assertion ([`assert_accounting`]) is what catches a
//! divergence between this walk and the lowering's minted accounting on
//! every run.
//!
//! [`visit_ops`] is the flat pre-order visitation the local passes
//! (`vif`, `vfor`) share. The `vslot` pass needs owner context — a
//! component's grouping reads its children's carriers and every
//! binding's own id — so it drives its own shaped recursion instead,
//! over the same [`PageWalk`]: the mint/skip arithmetic still lives only
//! here, and both shapes end at the same accounting assertion.

use vize_davinci::id::NodeId;
use vize_s2::op::Op;

/// The page-order id state of one pass run: a mirror of `Cx::mint_op`'s
/// numbering, saturation included.
#[derive(Clone)]
pub(crate) struct PageWalk {
    next: u32,
    exhausted: bool,
}

impl PageWalk {
    pub(crate) fn new() -> Self {
        Self {
            next: 0,
            exhausted: false,
        }
    }

    /// The current op's page-order id; `None` once the id space is
    /// exhausted (mirroring `Cx::mint_op`'s saturation).
    pub(crate) fn mint(&mut self) -> Option<NodeId> {
        if self.exhausted {
            return None;
        }
        match NodeId::from_index(self.next) {
            Some(id) => {
                self.next += 1;
                Some(id)
            }
            None => {
                self.exhausted = true;
                None
            }
        }
    }

    /// How many ids this walk has minted (the page's `ops=` count).
    pub(crate) fn minted(&self) -> u32 {
        self.next
    }

    /// Skip `count` ids (an owner's attached bindings, numbered between
    /// its line and its children).
    pub(crate) fn skip(&mut self, count: usize) {
        if self.exhausted {
            return;
        }
        match u32::try_from(count)
            .ok()
            .and_then(|count| self.next.checked_add(count))
        {
            Some(next) => self.next = next,
            None => self.exhausted = true,
        }
    }
}

/// Visit `ops` in page order, calling `visit` with each op's re-derived
/// id **before** descending into its regions (passes mutate an op's
/// surface, never its children's order, so pre-order is the one order
/// every pass needs).
pub(crate) fn visit_ops<'a>(
    walk: &mut PageWalk,
    ops: &mut [Op<'a>],
    visit: &mut impl FnMut(Option<NodeId>, &mut Op<'a>),
) {
    for op in ops.iter_mut() {
        let id = walk.mint();
        visit(id, op);
        match op {
            Op::Element(element) => {
                walk.skip(element.bindings.len());
                visit_ops(walk, &mut element.children.ops, visit);
            }
            Op::Component(component) => {
                walk.skip(component.bindings.len());
                visit_ops(walk, &mut component.children.ops, visit);
            }
            Op::Text(_) | Op::Interpolation(_) => {}
            Op::If(if_op) => {
                for branch in if_op.branches.iter_mut() {
                    visit_ops(walk, &mut branch.region.ops, visit);
                }
            }
            Op::For(for_op) => visit_ops(walk, &mut for_op.region.ops, visit),
            Op::Slot(slot) => {
                walk.skip(slot.bindings.len());
                visit_ops(walk, &mut slot.fallback.ops, visit);
            }
        }
    }
}

/// Assert the re-derived count against the lowering's minted accounting.
///
/// # Panics
///
/// Panics when the walk and the lowering disagree — a compiler bug by
/// the id law, never an input property.
pub(crate) fn assert_accounting(walk: &PageWalk, op_count: u32, pass: &str) {
    assert!(
        walk.exhausted || walk.next == op_count,
        "{pass} pass renumbering diverged from the minted accounting: \
         walked {} ops, the lowering minted {op_count}",
        walk.next,
    );
}
