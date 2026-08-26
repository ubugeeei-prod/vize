//! P1-10: tearing a deep template down no longer needs the stack guard.
//!
//! Before this task the template nodes owned heap strings, so the arena had to
//! run their destructors — and that teardown was itself a recursive walk of the
//! subtree (`Vec<TemplateChildNode>` -> `Box<ElementNode>` -> `Vec<…>`, one
//! machine-stack frame per nesting level). `ElementNode`, `IfBranchNode` and
//! `ForNode` therefore carried hand-written `Drop` impls that cleared their
//! children inside `ensure_sufficient_stack`, or a template deep enough would
//! abort the process on the way *out* of a compile that had just succeeded.
//!
//! The string diet removed the last owner, so the nodes have no drop glue at
//! all and the guard's reason is gone. This pins both halves of that claim:
//! the mechanical one (no node type needs dropping, so teardown cannot
//! recurse) and the end-to-end one (`stress-deep.vue` compiles and tears down
//! on a thread whose stack is far too small for a per-level teardown).

use std::mem::needs_drop;

use davinci_harness::fixtures::{LADDER, template_block};
use vize_atelier_core::{
    AttributeNode, CommentNode, DirectiveNode, ElementNode, ForNode, IfBranchNode, IfNode,
    InterpolationNode, PropNode, RootNode, SimpleExpressionNode, TemplateChildNode, TextNode,
};
use vize_atelier_dom::{DomCompilerOptions, compile_template_with_options};
use vize_s0::Allocator;

/// Stack for the teardown thread.
///
/// `stress-deep.vue` nests 60+ levels; the retired drop glue burned a frame
/// per level on top of whatever the compile passes had already used, which is
/// exactly what the guard existed to survive. 256 KiB is comfortable for the
/// guarded passes (they grow the stack on demand through
/// `vize_s0::ensure_sufficient_stack`) and leaves no room for an unguarded
/// recursive teardown to hide in.
const SMALL_STACK: usize = 256 * 1024;

#[test]
fn template_nodes_have_no_drop_glue() {
    assert!(!needs_drop::<TemplateChildNode<'_>>());
    assert!(!needs_drop::<ElementNode<'_>>());
    assert!(!needs_drop::<AttributeNode<'_>>());
    assert!(!needs_drop::<DirectiveNode<'_>>());
    assert!(!needs_drop::<PropNode<'_>>());
    assert!(!needs_drop::<TextNode<'_>>());
    assert!(!needs_drop::<CommentNode<'_>>());
    assert!(!needs_drop::<InterpolationNode<'_>>());
    assert!(!needs_drop::<SimpleExpressionNode<'_>>());
    assert!(!needs_drop::<IfNode<'_>>());
    assert!(!needs_drop::<IfBranchNode<'_>>());
    assert!(!needs_drop::<ForNode<'_>>());
    assert!(!needs_drop::<RootNode<'_>>());
}

#[test]
fn stress_deep_compiles_and_tears_down_on_a_small_stack() {
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
            let allocator = Allocator::new();
            let (root, errors, compiled) =
                compile_template_with_options(&allocator, &template, DomCompilerOptions::default());
            assert!(errors.is_empty(), "stress-deep.vue must compile cleanly");
            // Drop the tree and the arena here, inside the small stack: this
            // is the frame the retired `Drop` impls used to recurse on.
            drop(root);
            drop(allocator);
            compiled.code
        })
        .expect("spawning the teardown thread")
        .join()
        .expect("the small-stack compile and teardown must not abort");

    assert!(
        !code.is_empty(),
        "stress-deep.vue must produce a render function"
    );
}
