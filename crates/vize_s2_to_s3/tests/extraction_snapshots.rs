//! TS-17 snapshots of P3-10 extraction decisions.
//!
//! Each snapshot pins the committed placement page and the extraction page
//! (tier, budget, plan metrics before and after, one row per decision). Every
//! run must also leave the graph page byte-identical, keep the exported
//! partition current, and verify.

use vize_davinci::folio::{Folio, FolioMode};
use vize_s0::{Allocator, String};
use vize_s2_to_s3::{lower, optimize};
use vize_s3::extract::{OptTier, S3ExtractionFolio};
use vize_s3::folio::S3Folio;
use vize_s3::placement::S3PlacementFolio;
use vize_s3::verify::verify;

macro_rules! assert_extraction_snapshot {
    ($value:expr) => {{
        #[allow(clippy::disallowed_macros)]
        {
            ::insta::assert_snapshot!($value);
        }
    }};
}

fn decisions(source: &str, tier: OptTier) -> String {
    let allocator = Allocator::default();
    let (tree, errors) = vize_s1::parse(&allocator, source);
    let s2 = vize_s1_to_s2::lower(&allocator, &tree, &errors);
    let mut lowered = lower(&allocator, &s2.root);
    let graph = S3Folio::of(&lowered.program).print_to_string(FolioMode::Full);

    let extraction = optimize(&mut lowered, tier);

    assert_eq!(verify(&lowered.program), []);
    assert_eq!(lowered.partition.stale(&lowered.program), None);
    assert_eq!(
        S3Folio::of(&lowered.program).print_to_string(FolioMode::Full),
        graph
    );
    let mut output = S3PlacementFolio::of(&lowered.program).print_to_string(FolioMode::Full);
    output.push_str(
        S3ExtractionFolio::of(&extraction)
            .print_to_string(FolioMode::Full)
            .as_str(),
    );
    output
}

const BRANCH_AND_HANDLER: &str =
    r#"<section><p v-if="ready"><b>ok</b> done</p><button @click="save">go</button></section>"#;

#[test]
fn o1_groups_identical_direct_reads() {
    assert_extraction_snapshot!(decisions(
        r#"<div :title="msg">{{ msg }}</div>"#,
        OptTier::O1
    ));
}

#[test]
fn o1_hoists_static_branch_content_and_misses_the_cache_on_size() {
    assert_extraction_snapshot!(decisions(BRANCH_AND_HANDLER, OptTier::O1));
}

#[test]
fn o0_measures_nothing_and_keeps_every_op_inline() {
    assert_extraction_snapshot!(decisions(BRANCH_AND_HANDLER, OptTier::O0));
}

#[test]
fn o1_groups_loop_item_reads() {
    assert_extraction_snapshot!(decisions(
        r#"<ul><li v-for="item in items" @click="pick(item)" :title="item.name">{{ item.name }}</li></ul>"#,
        OptTier::O1
    ));
}

#[test]
fn a_lone_static_branch_node_is_missed_on_size() {
    assert_extraction_snapshot!(decisions(
        r#"<div><template v-if="ok"><br></template></div>"#,
        OptTier::O1
    ));
}

const ELEVEN_TITLES: &str = r#"<nav><i :title="n"></i><i :title="n"></i><i :title="n"></i><i :title="n"></i><i :title="n"></i><i :title="n"></i><i :title="n"></i><i :title="n"></i><i :title="n"></i><i :title="n"></i><i :title="n"></i></nav>"#;

#[test]
fn o1_spends_its_budget_and_later_group_members_miss() {
    assert_extraction_snapshot!(decisions(ELEVEN_TITLES, OptTier::O1));
}

#[test]
fn o2_has_budget_for_the_whole_run() {
    assert_extraction_snapshot!(decisions(ELEVEN_TITLES, OptTier::O2));
}

/// Every committed TS-19 battery fixture at the largest tier: the plan
/// verifies, the partition stays current, and every decision is pinned.
#[test]
fn surface_battery_extracts_at_o3() {
    use davinci_test_support::surface_fixture::{MALFORMED, WELL_FORMED};
    let mut output = String::default();
    for fixture in WELL_FORMED.iter().chain(MALFORMED) {
        output.push_str("# ");
        output.push_str(fixture.name);
        output.push('\n');
        output.push_str(decisions(fixture.source, OptTier::O3).as_str());
    }
    assert_extraction_snapshot!(output);
}
