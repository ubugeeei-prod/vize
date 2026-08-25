//! Davinci P1-7 differential lane — corpus-runnable entry (atelier wave B).
//!
//! Compiled only with the `davinci-differential` feature (see
//! `[[test]] required-features` in Cargo.toml), which arms the dual-run
//! comparators inside the migrated atelier sites: every retained-AST
//! consumption — the transform-lane rewrite walk, the codegen prefixer
//! walk, the vapor resolve walk, and the boolean shape checks — re-runs the
//! legacy re-parse path in an uncounted arena and panics on divergence
//! (never averaged, never preferred silently). Counters live in
//! `vize_atelier_core::retained::differential`.
//!
//! Run:
//!
//! ```text
//! cargo test -p vize_atelier_sfc --features davinci-differential \
//!     --test davinci_differential -- --nocapture
//! ```
//!
//! sweeps a committed expression battery plus the P0-2 fixture ladder —
//! each source compiled through the full SFC pipeline three times (default
//! DOM, SSR, vapor) — with pinned comparison counts (the ratchet: dispatch
//! or retention changes move these numbers loudly). The codegen-prefixer
//! lane pins at 0 on these routes (the SFC pipeline prefixes in the
//! transform, so codegen-side prefixing never fires); its coverage witness
//! is `vize_atelier_core`'s `retained::tests`, which drives that lane
//! directly. Setting
//! `VIZE_DAVINCI_DIFFERENTIAL_CORPUS=<dir>` additionally sweeps every
//! `.vue` file under `<dir>` (e.g. `tests/_fixtures/_git` for the hydrated
//! ecosystem corpus) through the same three-lane dual-run compile and
//! reports the totals. The canonical fixture root fails closed unless its
//! submodule inventory reconciles; other roots sweep in smoke scope with
//! `closure_evidence=false` (see `davinci_test_support::corpus`).

use std::fs;

use vize_atelier_core::retained::differential;
use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};

/// Complex-expression battery: every migrated consumption lane the P1-7
/// gates cover, plus fallback-class plants (TS-only syntax, comments,
/// v-for sub-expressions, multi-statement handlers) that must route to the
/// legacy parse, not the comparator.
const BATTERY_SOURCE: &str = r#"<script setup lang="ts">
import { ref } from "vue";
const isActive = ref(true);
const open = ref(false);
const items = ref([{ id: 1 }]);
const total = ref(2);
const alpha = ref(1);
const beta = ref(2);
const count = ref(0);
const value = ref("v");
const suffix = ref("s");
const entries = ref([{ id: 1, label: "a" }]);
const handle = (_event: unknown, _tag: string) => {};
</script>

<template>
  <div :class="{ active: isActive, 'is-open': open }" :style="{ zIndex: items.length + 1 }">
    {{ items.filter(item => item.id > 0).length / total }}
    {{ alpha + beta }}
  </div>
  <button @click="handle($event, 'x')" @keyup="count++" @focus="count++; total++">go</button>
  <span :title="(value as string) + suffix">{{ /* note */ alpha }}</span>
  <li v-for="{ id, label } in entries" :key="id">{{ label }}</li>
</template>
"#;

fn compile_source_all_lanes(source: &str) {
    let Ok(descriptor) = parse_sfc(source, SfcParseOptions::default()) else {
        return;
    };
    // Default (DOM) lane.
    let _ = compile_sfc(&descriptor, SfcCompileOptions::default());
    // SSR lane.
    let mut ssr = SfcCompileOptions::default();
    ssr.template.ssr = true;
    let _ = compile_sfc(&descriptor, ssr);
    // Vapor lane. Pug/jade templates are skipped here: parsing their source
    // as HTML yields a degenerate deeply nested tree that aborts the vapor
    // lane with a pre-existing unguarded recursion (reproduced on the P1-6
    // base tree; dom/ssr survive the same descriptor). Tracked outside
    // P1-7; remove the skip when the descent is stack-guarded.
    let template_is_pug = descriptor
        .template
        .as_ref()
        .and_then(|block| block.lang.as_deref())
        .is_some_and(|lang| lang.eq_ignore_ascii_case("pug") || lang.eq_ignore_ascii_case("jade"));
    if !template_is_pug {
        let vapor = SfcCompileOptions {
            vapor: true,
            ..SfcCompileOptions::default()
        };
        let _ = compile_sfc(&descriptor, vapor);
    }
}

/// One test function: the whole binary shares the process-global counters,
/// so a single entry keeps every pinned count deterministic. The body runs
/// on a dedicated wide-stack thread: test threads get 2 MiB, while the
/// dual-run over pathological corpus templates (deeply nested trees plus
/// the comparator's second parse) legitimately needs the headroom the CLI's
/// guarded entry normally provides.
#[test]
fn differential_lane_agrees_and_reports() {
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(differential_lane_body)
        .expect("spawn differential lane thread")
        .join()
        .expect("differential lane thread must not panic");
}

fn differential_lane_body() {
    let start = differential::stats();
    assert_eq!(
        start,
        differential::DifferentialStats {
            transform_rewrite_comparisons: 0,
            codegen_rewrite_comparisons: 0,
            vapor_resolve_comparisons: 0,
            shape_comparisons: 0,
        },
        "the lane binary must own its counters from zero"
    );
    assert_eq!(
        differential::transform_rewrite_legacy_stats(),
        differential::TransformRewriteLegacyStats {
            params: 0,
            unretained: 0,
            dialect_rejected: 0,
            ts_strip_rewrote: 0,
        },
        "the lane binary must own the P1-9 legacy counters from zero"
    );

    // -- committed battery, pinned counts ---------------------------------
    compile_source_all_lanes(BATTERY_SOURCE);
    let after_battery = differential::stats();
    assert_eq!(
        after_battery,
        differential::DifferentialStats {
            transform_rewrite_comparisons: 18,
            codegen_rewrite_comparisons: 0,
            vapor_resolve_comparisons: 7,
            shape_comparisons: 8,
        },
        "battery comparison counts moved: dispatch, retention, or lane routing changed — re-derive the pin deliberately"
    );
    // The P1-9 residual: the battery's fallback plants stay legacy — the
    // multi-statement `@focus` handler and the synthesized v-for source
    // sub-expression carry no whole-expression retained AST (the P1-5
    // completeness contract), and the TS-cast `:title` binding's retained
    // AST fails the dialect gate on its `TSAsExpression`. The dom and ssr
    // lanes route their shares here; vapor prefixes in its own resolver,
    // never through `rewrite_expression`.
    assert_eq!(
        differential::transform_rewrite_legacy_stats(),
        differential::TransformRewriteLegacyStats {
            params: 0,
            unretained: 4,
            dialect_rejected: 2,
            ts_strip_rewrote: 0,
        },
        "battery legacy-class counts moved: the P1-9 admission split changed — re-derive the pin deliberately"
    );

    // -- P0-2 fixture ladder, pinned counts -------------------------------
    for fixture in &davinci_harness::fixtures::LADDER {
        compile_source_all_lanes(fixture.source);
    }
    let after_ladder = differential::stats();
    assert_eq!(
        after_ladder,
        differential::DifferentialStats {
            transform_rewrite_comparisons: 1408,
            codegen_rewrite_comparisons: 0,
            vapor_resolve_comparisons: 64,
            shape_comparisons: 252,
        },
        "ladder comparison counts moved: dispatch, retention, or fixture content changed — re-derive the pin deliberately"
    );
    // Cumulative like the snapshot above it. Ladder-only residual: 16
    // params-position validations (slot params on `large.vue`) and 8 more
    // unretained rewrites (multi-statement handlers and v-for source
    // sub-expressions); every other transform rewrite across battery plus
    // ladder — 1408 of 1438 calls (97.9%) — was served by the AST-driven
    // splice.
    let after_ladder_legacy = differential::transform_rewrite_legacy_stats();
    assert_eq!(
        after_ladder_legacy,
        differential::TransformRewriteLegacyStats {
            params: 16,
            unretained: 12,
            dialect_rejected: 2,
            ts_strip_rewrote: 0,
        },
        "ladder legacy-class counts moved: the P1-9 admission split changed — re-derive the pin deliberately"
    );

    // -- optional corpus sweep --------------------------------------------
    if let Some(sweep) = davinci_test_support::corpus::resolve_env_sweep() {
        let files = &sweep.files;
        assert!(
            !files.is_empty(),
            "corpus sweep found no .vue files under {}",
            sweep.root.display()
        );
        let mut compiled = 0u64;
        for file in files {
            let Ok(source) = fs::read_to_string(file) else {
                continue;
            };
            compile_source_all_lanes(&source);
            compiled += 1;
        }
        let end = differential::stats();
        eprintln!(
            "davinci-differential corpus sweep: files={} compiled={} transform_rewrite_comparisons={} codegen_rewrite_comparisons={} vapor_resolve_comparisons={} shape_comparisons={}",
            files.len(),
            compiled,
            end.transform_rewrite_comparisons - after_ladder.transform_rewrite_comparisons,
            end.codegen_rewrite_comparisons - after_ladder.codegen_rewrite_comparisons,
            end.vapor_resolve_comparisons - after_ladder.vapor_resolve_comparisons,
            end.shape_comparisons - after_ladder.shape_comparisons,
        );
        // P1-9 admitted/legacy split for the transform prefixer over the
        // sweep: admitted == the transform_rewrite comparisons above.
        let sweep_admitted =
            end.transform_rewrite_comparisons - after_ladder.transform_rewrite_comparisons;
        let end_legacy = differential::transform_rewrite_legacy_stats();
        let sweep_legacy = differential::TransformRewriteLegacyStats {
            params: end_legacy.params - after_ladder_legacy.params,
            unretained: end_legacy.unretained - after_ladder_legacy.unretained,
            dialect_rejected: end_legacy.dialect_rejected - after_ladder_legacy.dialect_rejected,
            ts_strip_rewrote: end_legacy.ts_strip_rewrote - after_ladder_legacy.ts_strip_rewrote,
        };
        eprintln!(
            "davinci-differential corpus prefixer split: admitted={} legacy_total={} legacy_params={} legacy_unretained={} legacy_dialect_rejected={} legacy_ts_strip_rewrote={} admitted_pct={:.2}",
            sweep_admitted,
            sweep_legacy.total(),
            sweep_legacy.params,
            sweep_legacy.unretained,
            sweep_legacy.dialect_rejected,
            sweep_legacy.ts_strip_rewrote,
            100.0 * sweep_admitted as f64 / (sweep_admitted + sweep_legacy.total()) as f64,
        );
        assert!(
            end.transform_rewrite_comparisons > after_ladder.transform_rewrite_comparisons,
            "a corpus sweep that compares nothing proves nothing"
        );
    }

    let end = differential::stats();
    eprintln!(
        "davinci-differential totals: transform_rewrite_comparisons={} codegen_rewrite_comparisons={} vapor_resolve_comparisons={} shape_comparisons={}",
        end.transform_rewrite_comparisons,
        end.codegen_rewrite_comparisons,
        end.vapor_resolve_comparisons,
        end.shape_comparisons,
    );
}
