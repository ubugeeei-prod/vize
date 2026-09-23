//! P5-4a — the block firewall, with exact cache-hit accounting.
//!
//! Every scenario reads the whole file through the queries, then pins the
//! complete per-query accounting (executed / reused, from salsa's own event
//! stream) and checks every served artifact against the clean path run from
//! scratch. The fixture has four blocks: `script setup`, the template, and
//! two styles. Every block runs both per-block queries (`s1_block` answers
//! `None` outside a template, `s2_page` outside templates and styles), so
//! "only the edited block re-executes" is exactly `executed: 1, reused: 3`
//! on each of them — whichever block was edited.
//!
//! Off under `seeded-stale-cache`: that mutation build breaks the firewall on
//! purpose, and `tests/equivalence.rs` is the test that must see it.

#![cfg_attr(
    not(feature = "seeded-stale-cache"),
    expect(
        clippy::expect_used,
        clippy::string_slice,
        reason = "tests assert by panicking"
    )
)]
#![cfg(not(feature = "seeded-stale-cache"))]

use vize_resident::{
    Accounting, QueryCounts, ResidentDatabase, SourceFile, StageConfig, compute_file_artifacts,
};
use vize_s0::String;
use vize_s0::config::VueVersion;

const BASE: &str = "<script setup lang=\"ts\">
const n = 1
</script>

<template>
  <div :title=\"n\">{{ n }}</div>
</template>

<style scoped>
.a { color: v-bind(n); }
</style>

<style>
.b { color: red; }
</style>
";

fn accounting(rows: &[(&str, u32, u32)]) -> Accounting {
    rows.iter()
        .map(|&(query, executed, reused)| (String::from(query), QueryCounts { executed, reused }))
        .collect()
}

/// `base` with the single occurrence of `find` replaced.
fn edit(base: &str, find: &str, replace: &str) -> String {
    assert_eq!(base.matches(find).count(), 1, "edit anchor {find:?}");
    let at = base.find(find).expect("anchor");
    let mut out = String::from(&base[..at]);
    out.push_str(replace);
    out.push_str(&base[at + find.len()..]);
    out
}

/// Read every artifact of `file` through the queries, check it against the
/// clean path, and return the accounting of that read.
fn read_and_check(
    db: &ResidentDatabase,
    file: SourceFile,
    text: &str,
    config: StageConfig,
) -> Accounting {
    let served = db.file_artifacts(file);
    let accounting = db.take_accounting();
    assert_eq!(served, compute_file_artifacts(text, config));
    assert_eq!(served.len(), 4, "the fixture keeps its four blocks");
    accounting
}

fn opened() -> (ResidentDatabase, SourceFile) {
    let db = ResidentDatabase::default();
    let file = db.open("Counter.vue", BASE);
    let first = read_and_check(&db, file, BASE, StageConfig::default());
    assert_eq!(
        first,
        accounting(&[("sfc_blocks", 1, 0), ("s1_block", 4, 0), ("s2_page", 4, 0)])
    );
    (db, file)
}

#[test]
fn a_repeat_read_in_the_same_revision_runs_nothing() {
    let (db, file) = opened();
    assert_eq!(
        read_and_check(&db, file, BASE, StageConfig::default()),
        Accounting::new()
    );
}

#[test]
fn a_template_edit_reexecutes_only_the_templates_queries() {
    let (mut db, file) = opened();
    let text = edit(BASE, "{{ n }}", "{{ n + 1 }}");
    db.edit(file, &text);
    assert_eq!(
        read_and_check(&db, file, &text, StageConfig::default()),
        accounting(&[("sfc_blocks", 1, 0), ("s1_block", 1, 3), ("s2_page", 1, 3)])
    );
}

#[test]
fn a_style_edit_reexecutes_only_that_styles_queries() {
    let (mut db, file) = opened();
    let text = edit(BASE, "color: red", "color: blue");
    db.edit(file, &text);
    assert_eq!(
        read_and_check(&db, file, &text, StageConfig::default()),
        accounting(&[("sfc_blocks", 1, 0), ("s1_block", 1, 3), ("s2_page", 1, 3)])
    );
}

#[test]
fn an_edit_above_every_block_reexecutes_no_stage_query() {
    let (mut db, file) = opened();
    let above = edit(BASE, "<script setup", "<!-- header -->\n\n<script setup");
    db.edit(file, &above);
    // Every block moved (the served `start`s changed, and still equal the
    // clean path); no block's content did.
    assert_eq!(
        read_and_check(&db, file, &above, StageConfig::default()),
        accounting(&[("sfc_blocks", 1, 0), ("s1_block", 0, 4), ("s2_page", 0, 4)])
    );
}

#[test]
fn a_script_edit_above_the_template_reexecutes_only_the_scripts_queries() {
    let (mut db, file) = opened();
    let script = edit(BASE, "const n = 1", "const n = 1\nconst m = 2");
    db.edit(file, &script);
    // The template and both styles moved; only the script's own per-block
    // queries (both answering `None`) re-ran.
    assert_eq!(
        read_and_check(&db, file, &script, StageConfig::default()),
        accounting(&[("sfc_blocks", 1, 0), ("s1_block", 1, 3), ("s2_page", 1, 3)])
    );
}

#[test]
fn a_config_change_reexecutes_only_the_queries_that_read_it() {
    let (mut db, file) = opened();
    let vue2 = StageConfig {
        vue_version: VueVersion::V2,
    };
    db.configure(vue2);
    // The block split and S1 never read the project config; every page does.
    assert_eq!(
        read_and_check(&db, file, BASE, vue2),
        accounting(&[("sfc_blocks", 0, 1), ("s1_block", 0, 4), ("s2_page", 4, 0)])
    );
}

#[test]
fn a_no_op_edit_backdates_at_the_firewall() {
    let (mut db, file) = opened();
    db.edit(file, BASE);
    assert_eq!(
        read_and_check(&db, file, BASE, StageConfig::default()),
        accounting(&[("sfc_blocks", 1, 0), ("s1_block", 0, 4), ("s2_page", 0, 4)])
    );
}
