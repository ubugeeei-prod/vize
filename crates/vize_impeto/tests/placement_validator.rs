//! TS-27 placement validator (`S3V010`): the verifier accepts exactly the
//! well-formed placement alternatives and committed choices.

mod placement_validator {
    mod alternatives;
    mod choices;
    pub mod fixture;
}

use placement_validator::fixture::{Build, JS, LIT, fixture};
use vize_impeto::op::{OpId, OpKind};
use vize_impeto::placement::{Placement, PlacementRecord, PlacementSet, annotate};
use vize_impeto::verify::verify;
use vize_s0::Allocator;

use Placement::{Cache, Group, Hoist};

fn record(op: u32, alternative: Placement, leader: Option<u32>) -> PlacementRecord {
    PlacementRecord::new(
        OpId::new(op),
        PlacementSet::INLINE.with(alternative),
        leader.map(OpId::new),
    )
}

#[test]
fn annotate_records_exactly_the_documented_alternatives() {
    let arena = Allocator::default();
    let mut build = fixture(&arena, (LIT, "hi"));
    annotate(&mut build.program);

    assert_eq!(
        build.program.placements.as_slice(),
        [
            record(1, Cache, None),
            record(3, Group, Some(2)),
            record(5, Hoist, None),
        ]
    );
    assert_eq!(verify(&build.program), []);
}

#[test]
fn annotate_replaces_previous_choices_with_inline() {
    let arena = Allocator::default();
    let mut build = fixture(&arena, (LIT, "hi"));
    build.record(0, &[Placement::Inline, Hoist], None);
    annotate(&mut build.program);
    build.choose(5, Hoist);
    annotate(&mut build.program);

    assert_eq!(
        build.program.placements.as_slice(),
        [
            record(1, Cache, None),
            record(3, Group, Some(2)),
            record(5, Hoist, None),
        ]
    );
}

#[test]
fn annotate_leaves_the_canonical_graph_untouched() {
    let arena = Allocator::default();
    let mut build = fixture(&arena, (LIT, "hi"));
    let before = (
        build.program.ops.to_vec(),
        build.program.regions.to_vec(),
        build.program.edges.to_vec(),
        build.program.effects.to_vec(),
        build.program.operands.len(),
    );
    annotate(&mut build.program);

    let after = (
        build.program.ops.to_vec(),
        build.program.regions.to_vec(),
        build.program.edges.to_vec(),
        build.program.effects.to_vec(),
        build.program.operands.len(),
    );
    assert_eq!(after, before);
}

#[test]
fn every_committed_alternative_is_accepted() {
    let arena = Allocator::default();
    let mut build = fixture(&arena, (LIT, "hi"));
    annotate(&mut build.program);
    build.choose(1, Cache);
    build.choose(3, Group);
    build.choose(5, Hoist);

    assert_eq!(verify(&build.program), []);
}

#[test]
fn a_dynamic_child_blocks_the_hoist_and_nothing_else() {
    let arena = Allocator::default();
    let mut build = fixture(&arena, (JS, "label"));
    annotate(&mut build.program);

    assert_eq!(
        build.program.placements.as_slice(),
        [record(1, Cache, None), record(3, Group, Some(2))]
    );
}

#[test]
fn a_run_of_identical_reads_joins_its_first_op() {
    let arena = Allocator::default();
    let mut build = Build::new(&arena, (0, 30));
    build.text(0, 0, (0, 10), (JS, "count"), false);
    build.text(1, 0, (10, 20), (JS, "count"), false);
    build.text(2, 0, (20, 30), (JS, "count"), false);
    annotate(&mut build.program);

    assert_eq!(
        build.program.placements.as_slice(),
        [record(1, Group, Some(0)), record(2, Group, Some(0))]
    );
    build.choose(1, Group);
    build.choose(2, Group);
    assert_eq!(verify(&build.program), []);
}

#[test]
fn only_direct_references_group() {
    let arena = Allocator::default();
    let mut build = Build::new(&arena, (0, 60));
    build.text(0, 0, (0, 10), (JS, "count + 1"), false);
    build.text(1, 0, (10, 20), (JS, "count + 1"), false);
    build.text(2, 0, (20, 30), (JS, "a.b"), false);
    build.text(3, 0, (30, 40), (JS, "a.b"), false);
    build.text(4, 0, (40, 50), (JS, "a.b()"), false);
    build.text(5, 0, (50, 60), (JS, "a.b()"), false);
    annotate(&mut build.program);

    assert_eq!(
        build.program.placements.as_slice(),
        [record(3, Group, Some(2))]
    );
}

#[test]
fn instance_attributes_never_hoist() {
    let arena = Allocator::default();
    let mut build = fixture(&arena, (LIT, "hi"));
    build.attribute(5, "ref", "panel");
    annotate(&mut build.program);

    assert_eq!(
        build.program.placements.as_slice(),
        [record(1, Cache, None), record(3, Group, Some(2))]
    );
    assert_eq!(verify(&build.program), []);
}

#[test]
fn handlers_under_a_for_scope_never_cache() {
    let arena = Allocator::default();
    let mut build = Build::new(&arena, (0, 50));
    build.op(0, OpKind::For, 0, (0, 50), true);
    build.operand(
        0,
        vize_impeto::operand::OperandRole::ForSource,
        None,
        JS,
        "items",
    );
    build.region(1, 0, 0, (5, 45));
    build.element(1, "button", 1, (5, 45), true);
    build.binding(
        2,
        OpKind::SetEvent,
        1,
        (10, 20),
        ("on", "click", JS, "save"),
    );
    annotate(&mut build.program);

    assert_eq!(build.program.placements.as_slice(), []);
    assert_eq!(verify(&build.program), []);
}
