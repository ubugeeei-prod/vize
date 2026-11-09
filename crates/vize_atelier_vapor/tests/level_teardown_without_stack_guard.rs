//! P1-10: the Vapor IR tears down without `drop_ir_stack_safe`.
//!
//! Patterned templates lower to nested `If`/`For` operations, and the derived
//! drop glue followed that graph on the machine stack — so `ir_drop.rs` used
//! to drain every recursive edge into an explicit heap worklist before the
//! remaining shallow values were dropped normally. That module existed only
//! because IR nodes owned heap strings and heap vectors; P1-10 moved both into
//! the arena, so the whole graph is drop-free and the worklist is gone.
//!
//! This pins the claim mechanically (no IR node needs dropping) and
//! end-to-end (`stress-deep.vue` lowers, generates and tears down on a thread
//! whose stack is far too small for a per-level teardown).

use std::mem::needs_drop;

use davinci_harness::fixtures::{LADDER, template_block};
use vize_atelier_vapor::{
    BlockIRNode, CreateComponentIRNode, DirectiveIRNode, ForIRNode, IRDynamicInfo, IREffect,
    IRProp, IRSlot, IfIRNode, InsertNodeIRNode, NegativeBranch, OperationNode, PrependNodeIRNode,
    RootIRNode, SetEventIRNode, SetPropIRNode, SlotOutletIRNode, VaporCompilerOptions,
    compile_vapor,
};

/// See the DOM twin of this test for why 256 KiB is the right size.
const SMALL_STACK: usize = 256 * 1024;

#[test]
fn ir_nodes_have_no_drop_glue() {
    assert!(!needs_drop::<OperationNode<'_>>());
    assert!(!needs_drop::<BlockIRNode<'_>>());
    assert!(!needs_drop::<IREffect<'_>>());
    assert!(!needs_drop::<IRDynamicInfo<'_>>());
    assert!(!needs_drop::<IfIRNode<'_>>());
    assert!(!needs_drop::<NegativeBranch<'_>>());
    assert!(!needs_drop::<ForIRNode<'_>>());
    assert!(!needs_drop::<SetPropIRNode<'_>>());
    assert!(!needs_drop::<SetEventIRNode<'_>>());
    assert!(!needs_drop::<DirectiveIRNode<'_>>());
    assert!(!needs_drop::<CreateComponentIRNode<'_>>());
    assert!(!needs_drop::<SlotOutletIRNode<'_>>());
    assert!(!needs_drop::<IRProp<'_>>());
    assert!(!needs_drop::<IRSlot<'_>>());
    assert!(!needs_drop::<InsertNodeIRNode<'_>>());
    assert!(!needs_drop::<PrependNodeIRNode<'_>>());
}

/// `RootIRNode` keeps owned maps of template strings, so it is the one IR
/// value that still runs a destructor — and it is a single by-value root, not
/// a recursive edge. Pinned so the distinction stays deliberate.
#[test]
fn only_the_ir_root_still_owns_anything() {
    assert!(needs_drop::<RootIRNode<'_>>());
}

#[test]
fn stress_deep_lowers_and_tears_down_on_a_small_stack() {
    let fixture = LADDER
        .iter()
        .find(|fixture| fixture.name == "stress-deep")
        .expect("the ladder carries stress-deep.vue");
    let template = template_block(fixture.source)
        .expect("stress-deep.vue has a template block")
        .to_owned();

    let code = std::thread::Builder::new()
        .stack_size(SMALL_STACK)
        .spawn(move || {
            let allocator = vize_carton::Allocator::new();
            let compiled = compile_vapor(&allocator, &template, VaporCompilerOptions::default());
            // The arena goes here, inside the small stack: this is the frame
            // the retired `drop_ir_stack_safe` worklist used to protect.
            drop(allocator);
            compiled.code
        })
        .expect("spawning the teardown thread")
        .join()
        .expect("the small-stack lowering and teardown must not abort");

    assert!(
        !code.is_empty(),
        "stress-deep.vue must produce a vapor render function"
    );
}
