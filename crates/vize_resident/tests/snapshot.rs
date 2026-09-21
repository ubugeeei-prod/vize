//! TS-46 — snapshot adoption and cancellation (P5-5), plus the
//! fault-isolation scenario (extends TS-47).
//!
//! Every update's accounting is pinned exactly per joint (header, block,
//! region: adopted / computed / cancelled), and every tree's artifacts equal
//! the clean path. The fixture's template has three root regions.

#![cfg(not(feature = "seeded-stale-cache"))]

use std::cell::{Cell, RefCell};

use vize_resident::snapshot::cancel::{CancelToken, Cancelled};
use vize_resident::snapshot::isolate::{FileJob, FileOutcome, update_files_isolated};
use vize_resident::snapshot::region::{RegionLowering, RegionSyntax, lower_region};
use vize_resident::snapshot::{JointCounts, SnapshotStats, SnapshotTree, Stages};
use vize_resident::{BlockSource, StageConfig, SurfaceArtifact, compute_file_artifacts};
use vize_s0::String;
use vize_s0::config::VueVersion;
use vize_s1_to_s2::LegacyCaps;

const BASE: &str = "<script setup>
const a = 1
</script>

<template>
  <header :title=\"a\">{{ a }}</header>
  <main v-if=\"a\">one</main>
  <main v-else>two</main>
  <footer>{{ a + 1 }}</footer>
</template>

<style>
.a { color: red; }
</style>
";

fn counts(adopted: u32, computed: u32, cancelled: u32) -> JointCounts {
    JointCounts {
        adopted,
        computed,
        cancelled,
    }
}

fn stats(header: JointCounts, blocks: JointCounts, regions: JointCounts) -> SnapshotStats {
    SnapshotStats {
        header,
        blocks,
        regions,
    }
}

fn edit(base: &str, find: &str, replace: &str) -> String {
    assert_eq!(base.matches(find).count(), 1, "edit anchor {find:?}");
    let at = base.find(find).expect("anchor");
    let mut out = String::from(&base[..at]);
    out.push_str(replace);
    out.push_str(&base[at + find.len()..]);
    out
}

/// Update and check the tree against the clean path.
fn step(
    previous: Option<&SnapshotTree>,
    text: &str,
    config: StageConfig,
) -> (SnapshotTree, SnapshotStats) {
    let (tree, stats) = SnapshotTree::update(
        previous,
        text,
        config,
        &Stages::DEFAULT,
        CancelToken::root(),
    )
    .expect("uncancelled");
    assert_eq!(tree.artifacts(), compute_file_artifacts(text, config));
    (tree, stats)
}

#[test]
fn unchanged_joints_are_adopted_and_changed_ones_cancelled() {
    let config = StageConfig::default();
    let (open, first) = step(None, BASE, config);
    assert_eq!(
        first,
        stats(counts(0, 1, 0), counts(0, 3, 0), counts(0, 3, 0))
    );

    // Inside the last region: the if-chain and the header region are adopted
    // in place; the footer region is re-lowered; script and style adopted.
    let footer = edit(BASE, "{{ a + 1 }}", "{{ a + 2 }}");
    let (tree, stats_footer) = step(Some(&open), &footer, config);
    assert_eq!(
        stats_footer,
        stats(counts(1, 0, 0), counts(2, 1, 1), counts(2, 1, 1))
    );

    // Growing the first region moves the two after it: adopted, spans moved.
    let header = edit(&footer, "{{ a }}</header>", "{{ a }} and more</header>");
    let (tree, stats_header) = step(Some(&tree), &header, config);
    assert_eq!(
        stats_header,
        stats(counts(1, 0, 0), counts(2, 1, 1), counts(2, 1, 1))
    );

    // A script edit moves the template and the style: both adopted whole.
    let script = edit(&header, "const a = 1", "const a = 10");
    let (tree, stats_script) = step(Some(&tree), &script, config);
    assert_eq!(
        stats_script,
        stats(counts(1, 0, 0), counts(2, 1, 1), counts(0, 0, 0))
    );

    // A header change (a new attribute) restarts the file: nothing adopted.
    let scoped = edit(&script, "<style>", "<style scoped>");
    let (tree, stats_scoped) = step(Some(&tree), &scoped, config);
    assert_eq!(
        stats_scoped,
        stats(counts(0, 1, 1), counts(0, 3, 0), counts(0, 3, 0))
    );

    // So does a project configuration change.
    let vue2 = StageConfig {
        vue_version: VueVersion::V2,
    };
    let (_, stats_config) = step(Some(&tree), &scoped, vue2);
    assert_eq!(
        stats_config,
        stats(counts(0, 1, 1), counts(0, 3, 0), counts(0, 3, 0))
    );
}

#[test]
fn replaced_tasks_are_cancelled_and_adopted_ones_are_not() {
    let config = StageConfig::default();
    let (open, _) = step(None, BASE, config);
    let root = open.token().clone();
    let footer = edit(BASE, "{{ a + 1 }}", "{{ a + 2 }}");
    let (tree, _) = step(Some(&open), &footer, config);
    let cancelled = |tree: &SnapshotTree| -> Vec<bool> {
        tree.task_tokens()
            .iter()
            .map(CancelToken::is_cancelled)
            .collect()
    };
    // Tokens in document order: file, script, template, its three regions,
    // style. The footer edit replaced the old template task — cascading to
    // its three region tasks — and nothing else; the header was adopted, so
    // the old file task lives on.
    assert!(!root.is_cancelled());
    assert_eq!(
        cancelled(&open),
        [false, false, true, true, true, true, false]
    );
    // The new tree's tasks, adopted or computed, are all live.
    assert_eq!(cancelled(&tree), [false; 7]);
    // Cancelling the new file task cascades to every task of the new tree,
    // adopted subtrees included.
    tree.token().cancel();
    assert_eq!(cancelled(&tree), [true; 7]);
    let (fresh, _) = step(Some(&open), &footer, config);
    let scoped = edit(&footer, "<style>", "<style scoped>");
    let (_, _) = step(Some(&fresh), &scoped, config);
    // The header restart cancelled every task of the previous tree.
    assert_eq!(cancelled(&fresh), [true; 7]);
}

#[test]
fn cancellation_cascades_to_every_child_token() {
    let file = CancelToken::root();
    let block = file.child();
    let region = block.child();
    let sibling = file.child();
    block.cancel();
    assert_eq!(
        [
            file.is_cancelled(),
            block.is_cancelled(),
            region.is_cancelled(),
            sibling.is_cancelled()
        ],
        [false, true, true, false]
    );
    file.cancel();
    assert_eq!(sibling.check(), Err(Cancelled));
}

thread_local! {
    static CANCEL_ON_REGION: RefCell<Option<CancelToken>> = const { RefCell::new(None) };
    static REGIONS_LOWERED: Cell<u32> = const { Cell::new(0) };
}

/// A region stage that cancels the running update after its first region.
fn cancelling_region(block: &str, region: &RegionSyntax, caps: LegacyCaps) -> RegionLowering {
    REGIONS_LOWERED.with(|count| count.set(count.get() + 1));
    CANCEL_ON_REGION.with(|token| token.borrow().as_ref().map(CancelToken::cancel));
    lower_region(block, region, caps)
}

#[test]
fn a_cancelled_update_stops_between_units_and_keeps_the_previous_tree() {
    let config = StageConfig::default();
    let (open, _) = step(None, BASE, config);
    let before = open.artifacts();

    let already = CancelToken::root();
    already.cancel();
    let edited = edit(BASE, "{{ a + 1 }}", "{{ a + 2 }}");
    let result = SnapshotTree::update(Some(&open), &edited, config, &Stages::DEFAULT, already);
    assert_eq!(result.map(|_| ()), Err(Cancelled));

    // Cancelled while the template's regions are being lowered: exactly one
    // region ran, then the update stopped.
    let token = CancelToken::root();
    CANCEL_ON_REGION.with(|slot| *slot.borrow_mut() = Some(token.clone()));
    REGIONS_LOWERED.with(|count| count.set(0));
    let stages = Stages {
        region: cancelling_region,
        ..Stages::DEFAULT
    };
    let multi = edit(&edited, "<header", "<nav>x</nav>\n  <header");
    let result = SnapshotTree::update(Some(&open), &multi, config, &stages, token);
    assert_eq!(result.map(|_| ()), Err(Cancelled));
    assert_eq!(REGIONS_LOWERED.with(Cell::get), 1);
    assert_eq!(open.artifacts(), before);
}

/// An S1 stage that panics on one marked file.
fn panicking_surface(source: &BlockSource) -> Option<SurfaceArtifact> {
    if source.text.contains("PANIC") {
        panic!("injected stage failure");
    }
    vize_resident::artifact::surface_artifact(source)
}

#[test]
fn a_panicking_stage_degrades_only_its_file() {
    let config = StageConfig::default();
    let stages = Stages {
        surface: panicking_surface,
        ..Stages::DEFAULT
    };
    let bad = edit(BASE, "one", "PANIC");
    let (previous, _) = step(None, BASE, config);
    let texts = [BASE, bad.as_str(), BASE];
    let jobs = texts
        .iter()
        .enumerate()
        .map(|(index, text)| FileJob {
            previous: (index == 1).then_some(&previous),
            text,
            token: CancelToken::root(),
        })
        .collect();
    let outcomes = update_files_isolated(jobs, config, &stages, 2);
    let summary: Vec<String> = outcomes
        .iter()
        .map(|outcome| match outcome {
            FileOutcome::Ready(tree, _) => {
                assert_eq!(tree.artifacts(), compute_file_artifacts(BASE, config));
                String::from("ready")
            }
            FileOutcome::Cancelled => String::from("cancelled"),
            FileOutcome::Degraded(message) => message.clone(),
        })
        .collect();
    assert_eq!(
        summary,
        [
            "ready",
            "internal error in a stage task: injected stage failure",
            "ready"
        ]
    );
    // The degraded file keeps answering from its previous tree.
    assert_eq!(previous.artifacts(), compute_file_artifacts(BASE, config));
}
