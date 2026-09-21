//! Exact rejections for malformed records and committed choices.

use vize_impeto::placement::Placement::{Cache, Group, Hoist, Inline};
use vize_impeto::placement::annotate;
use vize_s0::Allocator;

use super::fixture::{Build, JS, LIT, fixture, messages};

#[test]
fn records_must_resolve() {
    let arena = Allocator::default();
    let mut build = fixture(&arena, (LIT, "hi"));
    build.record(42, &[Inline, Cache], None);
    assert_eq!(
        messages(&build.program),
        ["S3V010 @0:0 placement for op#42 does not resolve"]
    );
}

#[test]
fn records_follow_op_order_once_per_op() {
    let arena = Allocator::default();
    let mut build = fixture(&arena, (LIT, "hi"));
    annotate(&mut build.program);
    build.program.placements.swap(0, 1);
    assert_eq!(
        messages(&build.program),
        ["S3V010 @5:15 placement for op#1 is not in op order"]
    );

    let mut build = fixture(&arena, (LIT, "hi"));
    build.record(1, &[Inline, Cache], None);
    build.record(1, &[Inline, Cache], None);
    assert_eq!(
        messages(&build.program),
        ["S3V010 @5:15 placement for op#1 is not in op order"]
    );
}

#[test]
fn records_list_inline_and_another_alternative() {
    let arena = Allocator::default();
    let mut build = fixture(&arena, (LIT, "hi"));
    build.record(1, &[Inline], None);
    build.record(2, &[Cache], None);
    assert_eq!(
        messages(&build.program),
        [
            "S3V010 @5:15 placement for op#1 must list inline and another alternative",
            "S3V010 @16:25 placement for op#2 must list inline and another alternative",
        ]
    );
}

#[test]
fn choices_come_from_the_alternatives() {
    let arena = Allocator::default();
    let mut build = fixture(&arena, (LIT, "hi"));
    build.record(1, &[Inline, Cache], None);
    build.choose(1, Hoist);
    assert_eq!(
        messages(&build.program),
        ["S3V010 @5:15 placement for op#1 chooses hoist outside its alternatives"]
    );
}

#[test]
fn leaders_accompany_exactly_the_group_alternative() {
    let arena = Allocator::default();
    let mut build = fixture(&arena, (LIT, "hi"));
    build.record(1, &[Inline, Cache], Some(2));
    build.record(3, &[Inline, Group], None);
    assert_eq!(
        messages(&build.program),
        [
            "S3V010 @5:15 placement for op#1 names leader op#2 without the group alternative",
            "S3V010 @30:40 placement for op#3 lists group without a leader",
        ]
    );
}

#[test]
fn a_chosen_group_is_contiguous_with_its_committed_unit() {
    let arena = Allocator::default();
    let mut build = Build::new(&arena, (0, 30));
    build.text(0, 0, (0, 10), (JS, "count"), false);
    build.text(1, 0, (10, 20), (JS, "count"), false);
    build.text(2, 0, (20, 30), (JS, "count"), false);
    annotate(&mut build.program);
    build.choose(2, Group);
    assert_eq!(
        messages(&build.program),
        ["S3V010 @20:30 chosen group for op#2 is not contiguous with committed leader op#0"]
    );
}

#[test]
fn a_chosen_hoist_is_never_nested_in_another() {
    let arena = Allocator::default();
    let mut build = fixture(&arena, (LIT, "hi"));
    build.element(8, "b", 3, (60, 70), true);
    build.record(5, &[Inline, Hoist], None);
    build.record(8, &[Inline, Hoist], None);
    build.choose(8, Hoist);
    assert_eq!(messages(&build.program), [""; 0]);

    build.choose(5, Hoist);
    assert_eq!(
        messages(&build.program),
        ["S3V010 @60:70 chosen hoist for op#8 is nested inside hoisted op#5"]
    );
}

#[test]
fn every_defect_is_reported_once_in_record_order() {
    let arena = Allocator::default();
    let mut build = fixture(&arena, (LIT, "hi"));
    build.record(3, &[Inline, Group], Some(1));
    build.record(6, &[Inline, Hoist], None);
    build.choose(6, Hoist);
    assert_eq!(
        messages(&build.program),
        [
            "S3V010 @30:40 group leader op#1 for op#3 is not a dynamic direct-reference leaf update",
            "S3V010 @50:70 hoist for op#6 requires an insert-node",
        ]
    );
}
