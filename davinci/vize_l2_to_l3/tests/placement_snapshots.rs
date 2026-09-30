//! TS-17 snapshots of the canonical placement alternatives (P3-10).
//!
//! Each snapshot is the full graph Dump plus the placement page after
//! `annotate`. Annotation is an overlay: the graph page and the exported
//! partition facts must be byte-identical before and after it.

use vize_l0::dump::{Dump, Mode as DumpMode};
use vize_l0::{Allocator, String};
use vize_l2_to_l3::lower;
use vize_l3::dump::Page as L3Page;
use vize_l3::placement::annotate;
use vize_l3::placement::dump::Page as PlacementPage;
use vize_l3::verify::verify;

macro_rules! assert_placement_snapshot {
    ($value:expr) => {{
        #[allow(clippy::disallowed_macros)]
        {
            ::insta::assert_snapshot!($value);
        }
    }};
}

fn snapshot(source: &str) -> String {
    placements(source, true)
}

/// The graph page (when `graph_page` is set) followed by the placement page.
fn placements(source: &str, graph_page: bool) -> String {
    let allocator = Allocator::default();
    let (tree, errors) = vize_l1::parse(&allocator, source);
    let s2 = vize_l1_to_l2::lower(&allocator, &tree, &errors);
    let mut lowered = lower(&allocator, &s2.root);
    let graph = L3Page::of(&lowered.program).print_to_string(DumpMode::Full);
    let partition = lowered.partition.ops.to_vec();

    annotate(&mut lowered.program);

    assert_eq!(verify(&lowered.program), []);
    assert_eq!(
        L3Page::of(&lowered.program).print_to_string(DumpMode::Full),
        graph
    );
    assert_eq!(lowered.partition.ops.to_vec(), partition);
    let mut output = if graph_page { graph } else { String::default() };
    output.push_str(
        PlacementPage::of(&lowered.program)
            .print_to_string(DumpMode::Full)
            .as_str(),
    );
    output
}

#[test]
fn identical_direct_reads_offer_a_group() {
    assert_placement_snapshot!(snapshot(r#"<div :title="msg">{{ msg }}</div>"#));
}

#[test]
fn static_branch_content_offers_a_hoist_and_handlers_a_cache() {
    assert_placement_snapshot!(snapshot(
        r#"<section><p v-if="ready"><b>ok</b> done</p><button @click="save">go</button></section>"#
    ));
}

#[test]
fn loop_items_group_but_never_cache() {
    assert_placement_snapshot!(snapshot(
        r#"<ul><li v-for="item in items" @click="pick(item)" :title="item.name">{{ item.name }}</li></ul>"#
    ));
}

/// Every committed TS-19 battery fixture, well-formed or recovered, gets a
/// verifier-accepted placement overlay; the page per fixture is pinned whole.
#[test]
fn surface_battery_placements_verify() {
    use davinci_test_support::surface_fixture::{MALFORMED, WELL_FORMED};
    let mut output = String::default();
    for fixture in WELL_FORMED.iter().chain(MALFORMED) {
        output.push_str("# ");
        output.push_str(fixture.name);
        output.push('\n');
        output.push_str(placements(fixture.source, false).as_str());
    }
    assert_placement_snapshot!(output);
}

#[test]
fn different_reads_and_root_statics_offer_nothing() {
    assert_placement_snapshot!(snapshot(
        r#"<main class="shell"><h1>Title</h1><p :lang="locale">{{ label }}</p></main>"#
    ));
}
