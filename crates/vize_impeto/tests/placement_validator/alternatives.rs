//! Exact rejections for alternatives the canonical graph does not permit.

use vize_impeto::op::OpKind;
use vize_impeto::operand::{OperandRole, ValueKind};
use vize_impeto::placement::Placement::{Cache, Group, Hoist, Inline};
use vize_s0::Allocator;

use super::fixture::{Build, JS, LIT, fixture, messages};

#[test]
fn hoist_requires_an_insert_node() {
    let arena = Allocator::default();
    let mut build = fixture(&arena, (LIT, "hi"));
    build.record(6, &[Inline, Hoist], None);
    assert_eq!(
        messages(&build.program),
        ["S3V010 @50:70 hoist for op#6 requires an insert-node"]
    );
}

#[test]
fn hoist_requires_a_control_region() {
    let arena = Allocator::default();
    let mut build = fixture(&arena, (LIT, "hi"));
    build.record(0, &[Inline, Hoist], None);
    assert_eq!(
        messages(&build.program),
        ["S3V010 @0:100 hoist for op#0 is outside every control region"]
    );
}

#[test]
fn hoist_rejects_instance_attributes() {
    let arena = Allocator::default();
    let mut build = fixture(&arena, (LIT, "hi"));
    build.attribute(5, "KEY", "row");
    build.record(5, &[Inline, Hoist], None);
    assert_eq!(
        messages(&build.program),
        ["S3V010 @45:75 hoist for op#5 requires a literal node without instance attributes"]
    );
}

#[test]
fn hoist_rejects_a_dynamic_descendant() {
    let arena = Allocator::default();
    let mut build = fixture(&arena, (JS, "label"));
    build.record(5, &[Inline, Hoist], None);
    assert_eq!(
        messages(&build.program),
        ["S3V010 @45:75 hoist for op#5 covers non-static op#6"]
    );
}

#[test]
fn hoist_rejects_a_dynamic_binding_on_the_root() {
    let arena = Allocator::default();
    let mut build = fixture(&arena, (LIT, "hi"));
    build.binding(
        8,
        OpKind::SetProp,
        5,
        (46, 50),
        ("bind", "title", JS, "msg"),
    );
    build.record(5, &[Inline, Hoist], None);
    assert_eq!(
        messages(&build.program),
        ["S3V010 @45:75 hoist for op#5 covers non-static op#8"]
    );
}

#[test]
fn cache_requires_a_set_event() {
    let arena = Allocator::default();
    let mut build = fixture(&arena, (LIT, "hi"));
    build.record(2, &[Inline, Cache], None);
    assert_eq!(
        messages(&build.program),
        ["S3V010 @16:25 cache for op#2 requires a set-event"]
    );
}

#[test]
fn cache_rejects_a_static_event() {
    let arena = Allocator::default();
    let mut build = Build::new(&arena, (0, 30));
    build.element(0, "button", 0, (0, 30), false);
    build.op(1, OpKind::SetEvent, 0, (5, 15), false);
    build.operand(1, OperandRole::BindingKind, Some(0), LIT, "on");
    build.operand(1, OperandRole::Name, Some(0), LIT, "click");
    build.operand(1, OperandRole::Value, Some(0), JS, "save");
    build.record(1, &[Inline, Cache], None);
    assert_eq!(
        messages(&build.program),
        ["S3V010 @5:15 cache for op#1 is already static"]
    );
}

#[test]
fn cache_rejects_an_opaque_handler() {
    let arena = Allocator::default();
    let mut build = Build::new(&arena, (0, 30));
    build.element(0, "button", 0, (0, 30), false);
    let parts = ("on", "click", ValueKind::Opaque, "save(");
    build.binding(1, OpKind::SetEvent, 0, (5, 15), parts);
    build.record(1, &[Inline, Cache], None);
    assert_eq!(
        messages(&build.program),
        ["S3V010 @5:15 cache for op#1 requires one plain js handler under a static event name"]
    );
}

#[test]
fn cache_rejects_a_handler_under_for_scope() {
    let arena = Allocator::default();
    let mut build = Build::new(&arena, (0, 50));
    build.op(0, OpKind::For, 0, (0, 50), true);
    build.operand(0, OperandRole::ForSource, None, JS, "items");
    build.region(1, 0, 0, (5, 45));
    build.element(1, "button", 1, (5, 45), true);
    build.binding(
        2,
        OpKind::SetEvent,
        1,
        (10, 20),
        ("on", "click", JS, "save"),
    );
    build.record(2, &[Inline, Cache], None);
    assert_eq!(
        messages(&build.program),
        ["S3V010 @10:20 cache for op#2 may read for, slot, or component scope"]
    );
}

#[test]
fn group_requires_a_direct_reference_leaf_update() {
    let arena = Allocator::default();
    let mut build = fixture(&arena, (LIT, "hi"));
    build.record(5, &[Inline, Group], Some(2));
    assert_eq!(
        messages(&build.program),
        ["S3V010 @45:75 group for op#5 requires a dynamic direct-reference leaf update"]
    );
}

#[test]
fn group_leaders_must_resolve_and_read_a_direct_reference() {
    let arena = Allocator::default();
    let mut build = fixture(&arena, (LIT, "hi"));
    build.record(3, &[Inline, Group], Some(99));
    assert_eq!(
        messages(&build.program),
        ["S3V010 @30:40 group leader op#99 for op#3 does not resolve"]
    );

    let mut build = fixture(&arena, (LIT, "hi"));
    build.record(3, &[Inline, Group], Some(1));
    assert_eq!(
        messages(&build.program),
        ["S3V010 @30:40 group leader op#1 for op#3 is not a dynamic direct-reference leaf update"]
    );
}

#[test]
fn group_leaders_must_precede_their_members() {
    let arena = Allocator::default();
    let mut build = fixture(&arena, (LIT, "hi"));
    build.record(2, &[Inline, Group], Some(3));
    assert_eq!(
        messages(&build.program),
        ["S3V010 @16:25 group leader op#3 does not precede op#2"]
    );
}

#[test]
fn group_members_read_the_leader_reference() {
    let arena = Allocator::default();
    let mut build = Build::new(&arena, (0, 20));
    build.text(0, 0, (0, 10), (JS, "a"), false);
    build.text(1, 0, (10, 20), (JS, "b"), false);
    build.record(1, &[Inline, Group], Some(0));
    assert_eq!(
        messages(&build.program),
        ["S3V010 @10:20 group for op#1 reads `b` but leader op#0 reads `a`"]
    );
}

#[test]
fn group_members_share_the_leader_scope() {
    let arena = Allocator::default();
    let mut build = fixture(&arena, (JS, "msg"));
    build.record(6, &[Inline, Group], Some(3));
    assert_eq!(
        messages(&build.program),
        [
            "S3V010 @50:70 group for op#6 leaves the root, if-branch, or for-item scope of leader op#3"
        ]
    );
}

#[test]
fn component_content_never_groups() {
    let arena = Allocator::default();
    let mut build = Build::new(&arena, (0, 40));
    build.op(0, OpKind::CreateComponent, 0, (0, 40), true);
    build.operand(0, OperandRole::Tag, None, LIT, "Card");
    build.region(1, 0, 0, (10, 30));
    build.text(1, 1, (10, 20), (JS, "item"), true);
    build.text(2, 1, (20, 30), (JS, "item"), true);
    vize_impeto::placement::annotate(&mut build.program);
    assert_eq!(build.program.placements.as_slice(), []);

    build.record(2, &[Inline, Group], Some(1));
    assert_eq!(
        messages(&build.program),
        [
            "S3V010 @20:30 group for op#2 leaves the root, if-branch, or for-item scope of leader op#1"
        ]
    );
}

#[test]
fn group_leaders_head_their_run() {
    let arena = Allocator::default();
    let mut build = Build::new(&arena, (0, 30));
    build.text(0, 0, (0, 10), (JS, "count"), false);
    build.text(1, 0, (10, 20), (JS, "count"), false);
    build.text(2, 0, (20, 30), (JS, "count"), false);
    build.record(1, &[Inline, Group], Some(0));
    build.record(2, &[Inline, Group], Some(1));
    assert_eq!(
        messages(&build.program),
        ["S3V010 @20:30 group leader op#1 for op#2 is itself grouped"]
    );
}

#[test]
fn group_members_are_contiguous_in_the_keyed_effect_chain() {
    let arena = Allocator::default();
    let mut build = fixture(&arena, (LIT, "hi"));
    build.record(7, &[Inline, Group], Some(3));
    assert_eq!(
        messages(&build.program),
        ["S3V010 @80:90 group for op#7 is not contiguous with leader op#3"]
    );
}
