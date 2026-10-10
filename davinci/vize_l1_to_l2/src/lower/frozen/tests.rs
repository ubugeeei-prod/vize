//! Whole-source membership laws for the sole explicit compiler lowering entry.

use super::{BOUNDS, EXTRA, FOREIGN, MISSING};
use crate::lower::{lower, lower_with_frozen_element_spans};
use alloc::{borrow::ToOwned, vec};
use vize_l0::{Allocator, SourceRoot, Span, cstr};
use vize_l2::op::Op;

#[test]
fn absent_and_explicit_empty_custody_have_distinct_contracts() {
    let arena = Allocator::new();
    let source = "<slot v-pre></slot>";
    let (tree, errors) = vize_l1::parse(&arena, source);
    let original = lower(&arena, &tree, &errors);
    assert!(matches!(&original.root.ops[0], Op::Slot(_)));
    let block = SourceRoot::new(source).unwrap().whole_block();
    assert_eq!(
        lower_with_frozen_element_spans(&arena, &tree, &errors, block, &[]).err(),
        Some(MISSING)
    );
    let (normal, errors) = vize_l1::parse(&arena, "<slot></slot>");
    let block = SourceRoot::new(normal.source).unwrap().whole_block();
    let explicit = lower_with_frozen_element_spans(&arena, &normal, &errors, block, &[]).unwrap();
    let default = lower(&arena, &normal, &errors);
    assert_eq!(cstr!("{explicit:?}"), cstr!("{default:?}"));
}

#[test]
fn complete_opening_membership_changes_only_the_explicit_owner() {
    let arena = Allocator::new();
    let source = "<slot v-pre></slot>";
    let (tree, errors) = vize_l1::parse(&arena, source);
    let block = SourceRoot::new(source).unwrap().whole_block();
    let spans = [Span::new(0, 12)];
    let explicit = lower_with_frozen_element_spans(&arena, &tree, &errors, block, &spans).unwrap();
    assert!(
        matches!(&explicit.root.ops[0], Op::Element(element) if element.tag == "slot" && element.children.ops.is_empty())
    );
    assert!(matches!(
        &lower(&arena, &tree, &errors).root.ops[0],
        Op::Slot(_)
    ));
}

#[test]
fn foreign_whole_block_and_nonzero_blocks_refuse_without_tree_mutation() {
    let arena = Allocator::new();
    let source = "<slot v-pre></slot>".to_owned();
    let copy = source.clone();
    let (tree, errors) = vize_l1::parse(&arena, &source);
    let before = cstr!("{tree:?}");
    let foreign = SourceRoot::new(&copy).unwrap().whole_block();
    assert_eq!(
        lower_with_frozen_element_spans(&arena, &tree, &errors, foreign, &[]).err(),
        Some(FOREIGN)
    );
    assert_eq!(cstr!("{tree:?}"), before);
    let root = "prefix<slot v-pre></slot>";
    let block = SourceRoot::new(root).unwrap().block(&root[6..], 6).unwrap();
    let (tree, errors) = vize_l1::parse(&arena, block.source());
    let before = cstr!("{tree:?}");
    assert_eq!(
        lower_with_frozen_element_spans(&arena, &tree, &errors, block, &[]).err(),
        Some(FOREIGN)
    );
    assert_eq!(cstr!("{tree:?}"), before);
}

#[test]
fn duplicate_overlapping_out_of_bounds_and_non_frozen_spans_refuse() {
    let arena = Allocator::new();
    let source = "<slot v-pre></slot><i></i>";
    let (tree, errors) = vize_l1::parse(&arena, source);
    let block = SourceRoot::new(source).unwrap().whole_block();
    let before = cstr!("{tree:?}");
    for spans in [
        vec![Span::new(0, 12), Span::new(0, 12)],
        vec![Span::new(0, 12), Span::new(11, 13)],
        vec![Span::new(0, 999)],
        vec![Span::new(12, 0)],
    ] {
        assert_eq!(
            lower_with_frozen_element_spans(&arena, &tree, &errors, block, &spans).err(),
            Some(BOUNDS)
        );
    }
    for spans in [
        vec![Span::new(0, 11)],
        vec![Span::new(0, 12), Span::new(19, 22)],
    ] {
        let error = lower_with_frozen_element_spans(&arena, &tree, &errors, block, &spans).err();
        assert_eq!(error, Some(if spans.len() == 1 { MISSING } else { EXTRA }));
    }
    assert_eq!(cstr!("{tree:?}"), before);
}

#[test]
fn non_pre_opening_custody_is_not_an_arbitrary_literal_permission() {
    let arena = Allocator::new();
    let source = "<Child></Child>";
    let (tree, errors) = vize_l1::parse(&arena, source);
    let block = SourceRoot::new(source).unwrap().whole_block();
    let spans = [Span::new(0, 7)];
    assert_eq!(
        lower_with_frozen_element_spans(&arena, &tree, &errors, block, &spans).err(),
        Some(EXTRA)
    );
    assert!(matches!(
        &lower(&arena, &tree, &errors).root.ops[0],
        Op::Component(_)
    ));
}

#[test]
fn explicit_utf8_boundary_and_extra_nonopening_ranges_refuse() {
    let arena = Allocator::new();
    let source = "<div v-pre>ä</div>";
    let (tree, errors) = vize_l1::parse(&arena, source);
    let block = SourceRoot::new(source).unwrap().whole_block();
    let spans = [Span::new(0, 12)];
    assert_eq!(
        lower_with_frozen_element_spans(&arena, &tree, &errors, block, &spans).err(),
        Some(BOUNDS)
    );
    let spans = [Span::new(0, 11), Span::new(11, 13)];
    assert_eq!(
        lower_with_frozen_element_spans(&arena, &tree, &errors, block, &spans).err(),
        Some(EXTRA)
    );
}
