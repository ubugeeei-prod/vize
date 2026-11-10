//! Which listener of a component usage declares its inference-only
//! emit-props call.

use vize_carton::FxHashSet;

/// Usages whose inference-only emit-props call is already declared, keyed by
/// the guard block it was written in (`None`: the listeners' own level).
pub(crate) type EmitInferenceDeclarations = std::cell::RefCell<FxHashSet<(usize, Option<u64>)>>;

/// Whether this listener has to declare its usage's emit-props call.
///
/// A deferred callback that reads a stable `v-for`/`v-slot` alias is checked
/// inside its branch guard block. A call declared there cannot be seen by a
/// sibling listener written after that block closes or inside another block,
/// so each of those declares its own. A call declared at the listeners' own
/// level stays visible to every later sibling, blocks included.
pub(super) fn claim(declared: &EmitInferenceDeclarations, usage_idx: usize) -> bool {
    let block = super::super::open_chained_vif_guard_block();
    let mut declared = declared.borrow_mut();
    !declared.contains(&(usage_idx, None)) && declared.insert((usage_idx, block))
}
