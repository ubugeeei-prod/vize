//! TS-17 for the legacy port (P2-9 series 7, `_legacy` only): committed
//! fixture in, the **legacy** pipeline out, full normalized folio
//! snapshot — the P2-4 harness shape. The snapshots show the landed
//! surface: `vue.filter` lines with their pessimal `legacy-filter`
//! payloads, filter-bearing bind values as the same escape, the `.sync`
//! expansion's appended listener, the scoped-slot spelling as an
//! appended `ui.slot-content`, and the rewritten v-on modifiers; the
//! asset registration, walk accounting, and diagnostics are the
//! structural supplements (assurance §4).

mod support;

use std::path::{Path, PathBuf};

use vize_davinci::assert_folio_snapshot;
use vize_davinci::folio::{Folio, FolioMode};
use vize_ricalco::LegacyVueLine;

use support::{assert_transformed_sound_legacy, with_transformed_legacy};

fn fixture(name: &str) -> vize_carton::String {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("legacy")
        .join(name);
    let text = std::fs::read_to_string(path).expect("committed fixture reads");
    vize_carton::String::from(text.as_str())
}

#[test]
fn the_filters_fixture_snapshots_the_post_pass_folio() {
    let source = fixture("filters.vue");
    with_transformed_legacy(
        &source,
        LegacyVueLine::V2,
        |lowered, folio, facts, budget| {
            // The oracle: the full normalized folio after the legacy
            // pipeline ran. Lone chains are `vue.filter`, the bind chain is
            // the opaque value on its `ui.bind`, the logical-or and
            // malformed-name expressions stay ordinary `js`, and the merged
            // run keeps the Compound representation.
            assert_folio_snapshot!(*folio);

            // Supplements: six barriers plus the fusable singleton plus the
            // legacy barrier — seven walks.
            assert_eq!(
                budget.print_to_string(FolioMode::Full).as_str(),
                "[budget-observer]\nwalks=7\npasses=7\nanalyses=0\npipelines=1\nfailures=0\n\n"
            );
            // The registration fact: first-seen page order, deduplicated
            // (`capitalize` appears at two sites, once in the list).
            let names: Vec<&str> = facts.legacy.assets.iter().map(|s| s.as_str()).collect();
            assert_eq!(names, ["capitalize", "f", "g", "formatId", "quote", "h"]);
            // Five filter sites: four interpolations and one bind value —
            // the quoted-pipe title bind is genuinely a chain (`'a|b'`
            // piped through `quote`); the merged run and the two bails are
            // not sites.
            assert_eq!(facts.legacy.sites.len(), 5);
            assert_eq!(lowered.filters.len(), 5);
            assert_eq!(lowered.diagnostics, vec![]);
        },
    );
    assert_transformed_sound_legacy(&source, LegacyVueLine::V2, "filters.vue");
}

#[test]
fn the_sugar_fixture_snapshots_the_post_pass_folio() {
    let source = fixture("sugar.vue");
    with_transformed_legacy(
        &source,
        LegacyVueLine::V2,
        |lowered, folio, facts, budget| {
            // The oracle: the `.sync` binds stand stripped with their
            // appended `update:` listeners (the same-name shorthand's
            // camelized value included), the dynamic-argument `.sync` keeps
            // its authored modifier, the v-on modifiers are rewritten
            // (`native` gone, `13` → `enter`, `99` kept), and the
            // scoped-slot spellings are appended `ui.slot-content` ops —
            // except the conflict bail, whose authored `#conflict` wins.
            assert_folio_snapshot!(*folio);

            assert_eq!(
                budget.print_to_string(FolioMode::Full).as_str(),
                "[budget-observer]\nwalks=7\npasses=7\nanalyses=0\npipelines=1\nfailures=0\n\n"
            );
            // No filters in this fixture; the desugars leave provenance.
            assert!(facts.legacy.assets.is_empty());
            assert_eq!(facts.legacy.sites.len(), 0);
            let rule_count = |rule: &str| {
                lowered
                    .provenance
                    .iter()
                    .filter(|r| r.rule.as_str() == rule)
                    .count()
            };
            assert_eq!(rule_count("normalize.legacy.sync"), 3);
            assert_eq!(rule_count("normalize.legacy.slot-scope"), 2);
            assert_eq!(rule_count("consume.legacy.slot-name"), 1);
            assert_eq!(rule_count("normalize.legacy.native"), 2);
            assert_eq!(rule_count("normalize.legacy.keycode"), 1);
            assert_eq!(lowered.diagnostics, vec![]);
        },
    );
    assert_transformed_sound_legacy(&source, LegacyVueLine::V2, "sugar.vue");
}
