//! Where a block's render effects go relative to the components, slots and
//! control flow it inserts.
//!
//! Vue's compiler (3.6.0-rc.11) places an element's effects where the element
//! opens, so a parent's attributes are set before the blocks inside it are
//! created and hydrated: `<div :hidden="h"><slot/></div>` evaluates `h` before
//! the slot content runs. Each effect operation is keyed by its target's
//! document position, and the effects that open before an inserted block are
//! emitted just before that block.

use crate::ir::{BlockIRNode, InsertionAnchor, OperationNode};
use vize_carton::FxHashMap;

use super::{
    context::GenerateContext, helpers::generate_operation_inline, operations::generate_operation,
};

/// An element's place in its block: the template root it belongs to, its
/// pre-order rank among the referenced nodes, and its parent.
#[derive(Clone, Copy)]
struct Place {
    root: u32,
    rank: u32,
    parent: Option<u32>,
}

/// A document position: the root, then four slots per rank: a node (`4r`), the
/// text run inside it (`4r + 1`), a block appended to it (`4r + 3`, after its
/// subtree) and a block inserted before it (`4r - 2`).
type Key = (u32, u64);

/// The effect operations of one block, emitted in document-order groups.
pub(super) struct EffectOrder<'b, 'a> {
    block: &'b BlockIRNode<'a>,
    /// Indexed by element id; empty when no effect needs to move.
    places: Places,
    /// Effects with a key below this were emitted before an inserted block.
    emitted_below: Option<Key>,
}

impl<'b, 'a> EffectOrder<'b, 'a> {
    pub(super) fn new(block: &'b BlockIRNode<'a>) -> Self {
        let inserts = !block.effect.is_empty()
            && (block.operation.iter())
                .any(|op| insertion(op).is_some_and(|(_, parent, _)| parent.is_some()));
        let mut order = Self {
            block,
            places: Places::Empty,
            emitted_below: None,
        };
        if inserts {
            order.collect_places();
        }
        order
    }

    fn collect_places(&mut self) {
        let block = self.block;
        let largest = (block.returns.iter().copied())
            .chain(block.operation.iter().flat_map(|op| match op {
                OperationNode::ChildRef(child) => [child.child_id, child.parent_id],
                OperationNode::NextRef(next) => [next.child_id, next.prev_id],
                _ => [0, 0],
            }))
            .max()
            .unwrap_or(0);
        self.places = Places::with_len(largest + 1);
        for (root, id) in block.returns.iter().enumerate() {
            if let Some(slot) = self.places.slots_mut().get_mut(*id) {
                *slot = Some(Place {
                    root: root as u32,
                    rank: 0,
                    parent: None,
                });
            }
        }
        let mut rank = 0;
        for op in block.operation.iter() {
            let (child, relative, sibling) = match op {
                OperationNode::ChildRef(child) => (child.child_id, child.parent_id, false),
                OperationNode::NextRef(next) => (next.child_id, next.prev_id, true),
                _ => continue,
            };
            let Some(relative_place) = self.place(relative) else {
                continue;
            };
            rank += 1;
            let place = Place {
                root: relative_place.root,
                rank,
                parent: if sibling {
                    relative_place.parent
                } else {
                    Some(relative as u32)
                },
            };
            if let Some(slot) = self.places.slots_mut().get_mut(child) {
                *slot = Some(place);
            }
        }
    }

    fn place(&self, id: usize) -> Option<Place> {
        self.places.slots().get(id).copied().flatten()
    }

    /// Emits the effects that open before `op` when `op` inserts a block.
    pub(super) fn before(
        &mut self,
        ctx: &mut GenerateContext,
        op: &OperationNode<'_>,
        element_template_map: &FxHashMap<usize, usize>,
    ) {
        let Some(key) = self.insertion_key(op) else {
            return;
        };
        if self.emitted_below.is_some_and(|below| key <= below) {
            return;
        }
        let from = self.emitted_below;
        self.emitted_below = Some(key);
        self.emit(
            ctx,
            |op| {
                self.effect_key(op)
                    .is_some_and(|op_key| from.is_none_or(|from| op_key >= from) && op_key < key)
            },
            element_template_map,
        );
    }

    /// Emits every remaining effect.
    pub(super) fn rest(
        &mut self,
        ctx: &mut GenerateContext,
        element_template_map: &FxHashMap<usize, usize>,
    ) {
        // Without an earlier group (nearly every block) all effects remain.
        let Some(from) = self.emitted_below else {
            self.emit(ctx, |_| true, element_template_map);
            return;
        };
        self.emit(
            ctx,
            |op| self.effect_key(op).is_none_or(|op_key| op_key >= from),
            element_template_map,
        );
    }

    fn insertion_key(&self, op: &OperationNode<'_>) -> Option<Key> {
        if self.places.slots().is_empty() {
            return None;
        }
        let (_, parent, anchor) = insertion(op)?;
        let parent = parent?;
        match anchor {
            Some(InsertionAnchor::Node(placeholder)) => {
                let place = self.place(placeholder)?;
                Some((place.root, (4 * u64::from(place.rank)).saturating_sub(2)))
            }
            Some(InsertionAnchor::Index(_)) | None => {
                let place = self.place(parent)?;
                Some((
                    place.root,
                    4 * u64::from(self.subtree_end(parent, place.rank)) + 3,
                ))
            }
        }
    }

    /// The largest rank inside the element `id` (its own when it has none).
    fn subtree_end(&self, id: usize, rank: u32) -> u32 {
        let mut end = rank;
        for (candidate, place) in self.places.slots().iter().enumerate() {
            let Some(place) = place else { continue };
            if place.rank > end && self.descends_from(candidate, id) {
                end = place.rank;
            }
        }
        end
    }

    fn descends_from(&self, mut id: usize, ancestor: usize) -> bool {
        while let Some(parent) = self.place(id).and_then(|place| place.parent) {
            if parent as usize == ancestor {
                return true;
            }
            id = parent as usize;
        }
        false
    }

    fn effect_key(&self, op: &OperationNode<'_>) -> Option<Key> {
        if self.places.slots().is_empty() {
            return None;
        }
        let (element, inside) = match op {
            OperationNode::SetProp(node) => (node.element, false),
            OperationNode::SetDynamicProps(node) => (node.element, false),
            OperationNode::SetMergedProps(node) => (node.element, false),
            OperationNode::SetText(node) => (node.element, !node.is_element),
            OperationNode::SetEvent(node) => (node.element, false),
            OperationNode::SetHtml(node) => (node.element, false),
            OperationNode::SetTemplateRef(node) => (node.element, false),
            OperationNode::Directive(node) => (node.element, false),
            _ => return None,
        };
        let place = self.place(element)?;
        Some((place.root, 4 * u64::from(place.rank) + u64::from(inside)))
    }

    /// Emits, as one render effect, the effect operations `selected` keeps,
    /// in their order.
    fn emit(
        &self,
        ctx: &mut GenerateContext,
        selected: impl Fn(&OperationNode<'_>) -> bool,
        element_template_map: &FxHashMap<usize, usize>,
    ) {
        let block = self.block;
        let operations = || {
            (block.effect.iter())
                .flat_map(|effect| effect.operations.iter())
                .filter(|op| selected(op))
        };
        let mut chosen = operations();
        let Some(first) = chosen.next() else {
            return;
        };
        let single = chosen.next().is_none();
        ctx.use_helper("renderEffect");
        let outer = core::mem::replace(&mut ctx.in_effect, true);
        if single && let Some(code) = generate_operation_inline(ctx, first) {
            ctx.in_effect = outer;
            ctx.push_indent();
            ctx.push("_renderEffect(() => ");
            ctx.push_spanned(&code);
            ctx.push(")\n");
            return;
        }
        ctx.push_line("_renderEffect(() => {");
        ctx.indent();
        for op in operations() {
            generate_operation(ctx, op, element_template_map);
        }
        ctx.deindent();
        ctx.push_line("})");
        ctx.in_effect = outer;
    }
}

/// The id, insertion parent and anchor of an operation that inserts a block.
fn insertion(op: &OperationNode<'_>) -> Option<(usize, Option<usize>, Option<InsertionAnchor>)> {
    Some(match op {
        OperationNode::If(node) => (node.id, node.parent, node.anchor),
        OperationNode::For(node) => (node.id, node.parent, node.anchor),
        OperationNode::Key(node) => (node.id, node.parent, node.anchor),
        OperationNode::CreateComponent(node) => (node.id, node.parent, node.anchor),
        OperationNode::SlotOutlet(node) => (node.id, node.parent, node.anchor),
        _ => return None,
    })
}

/// Most blocks reference few elements: their places stay on the stack.
const INLINE_PLACES: usize = 48;

#[expect(
    clippy::large_enum_variant,
    reason = "inline places spare a heap allocation per block; one lives on the stack at a time"
)]
enum Places {
    Empty,
    Inline([Option<Place>; INLINE_PLACES], usize),
    Heap(std::vec::Vec<Option<Place>>),
}

impl Places {
    fn with_len(len: usize) -> Self {
        if len <= INLINE_PLACES {
            Self::Inline([None; INLINE_PLACES], len)
        } else {
            Self::Heap(std::vec![None; len])
        }
    }

    fn slots(&self) -> &[Option<Place>] {
        match self {
            Self::Empty => &[],
            Self::Inline(slots, len) => slots.get(..*len).unwrap_or_default(),
            Self::Heap(slots) => slots,
        }
    }

    fn slots_mut(&mut self) -> &mut [Option<Place>] {
        match self {
            Self::Empty => &mut [],
            Self::Inline(slots, len) => slots.get_mut(..*len).unwrap_or_default(),
            Self::Heap(slots) => slots,
        }
    }
}
