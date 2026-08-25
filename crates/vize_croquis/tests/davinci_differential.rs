//! Davinci P1-6 differential lane — corpus-runnable entry.
//!
//! Compiled only with the `davinci-differential` feature (see
//! `[[test]] required-features` in Cargo.toml), which also arms the dual-run
//! comparators inside the migrated croquis helpers: every retained-AST
//! identifier walk and component-reference check re-runs the legacy re-parse
//! path and panics on divergence, and every v-for destructure shape is
//! witness-counted (wave A forks no v-for path — see
//! `vize_croquis::drawer::differential`).
//!
//! Run:
//!
//! ```text
//! cargo test -p vize_croquis --features davinci-differential \
//!     --test davinci_differential -- --nocapture
//! ```
//!
//! sweeps a committed expression battery plus the P0-2 fixture ladder with
//! pinned comparison counts (the ratchet: dispatch or retention changes move
//! these numbers loudly). Setting `VIZE_DAVINCI_DIFFERENTIAL_CORPUS=<dir>`
//! additionally sweeps every `.vue` file under `<dir>` (e.g.
//! `tests/_fixtures/_git` for the hydrated ecosystem corpus) through the
//! same dual-run analysis and reports the totals. The canonical fixture
//! root fails closed unless its submodule inventory reconciles; other roots
//! sweep in smoke scope with `closure_evidence=false` (see
//! `davinci_test_support::corpus`).

use std::fs;

use vize_armature::Parser;
use vize_atelier_sfc::croquis::{SfcCroquisOptions, analyze_sfc_descriptor};
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_carton::Allocator;
use vize_croquis::drawer::differential;

/// Complex-expression battery: every retained slow-dispatch shape the P1-6
/// migration covers, plus component references and v-for destructures.
const BATTERY_SOURCE: &str = r#"<script setup lang="ts">
import { ref } from "vue";
const isActive = ref(true);
const open = ref(false);
const items = ref([{ id: 1 }]);
const total = ref(2);
const value = ref("v");
const suffix = ref("s");
const prefix = ref("p");
const count = ref(0);
const offset = ref(1);
const alpha = ref(1);
const message = ref("m");
const yes = ref("y");
const no = ref("n");
const targetComponent = ref("SomeComponent");
const shapes = ref({ current: "OtherComponent" });
const entries = ref([{ id: 1, label: "a" }]);
const matrix = ref([[1]]);
const nested = ref({ value: 1 });
const cond = ref(true);
const 名前 = ref("n");
const handle = (_event: unknown) => {};
</script>

<template>
  <div :class="{ active: isActive, 'is-open': open }">
    {{ items.filter(item => item.id > 0).length / total }}
  </div>
  <span :title="(value as string) + suffix">{{ /* note */ ({ a: alpha }).a }}</span>
  <p v-custom="{ deep: nested.value }">{{ `${prefix}-${suffix}` }}</p>
  <p>{{ message.match(/^m+$/) ? yes : no }}</p>
  <p>{{ 名前 + suffix }}</p>
  <component :is="targetComponent" @click="handle($event)" />
  <component :is="shapes.current" />
  <component :is="cond ? targetComponent : shapes.current" />
  <li v-for="{ id, label } in entries" :key="id">{{ label }}</li>
  <li v-for="(row, index) of matrix" :key="index">{{ row.join('/') }}</li>
</template>
"#;

fn analyze_source(source: &str, options: SfcCroquisOptions) {
    let descriptor = match parse_sfc(source, SfcParseOptions::default()) {
        Ok(descriptor) => descriptor,
        Err(_) => return,
    };
    let allocator = Allocator::new();
    let template_ast = descriptor.template.as_ref().map(|block| {
        let (root, _errors) = Parser::new(&allocator, block.content.as_ref()).parse();
        root
    });
    let croquis = analyze_sfc_descriptor(&descriptor, template_ast.as_ref(), options);
    core::hint::black_box(&croquis);
}

/// One test function: the whole binary shares the process-global counters,
/// so a single entry keeps every pinned count deterministic.
#[test]
fn differential_lane_agrees_and_reports() {
    let start = differential::stats();
    assert_eq!(
        start,
        differential::DifferentialStats {
            identifier_comparisons: 0,
            component_reference_comparisons: 0,
            v_for_shapes: 0,
        },
        "the lane binary must own its counters from zero"
    );

    // -- committed battery, pinned counts ---------------------------------
    // 8 comment-free retained slow-dispatch expressions (the comment-carrying
    // interpolation is planted as fallback-class coverage and must not be
    // compared), 3 `<component :is>` references checked in both drawer passes,
    // 2 v-for shapes.
    analyze_source(BATTERY_SOURCE, SfcCroquisOptions::full());
    let after_battery = differential::stats();
    assert_eq!(
        after_battery,
        differential::DifferentialStats {
            identifier_comparisons: 8,
            component_reference_comparisons: 6,
            v_for_shapes: 2,
        },
        "battery comparison counts moved: dispatch, retention, or drawer routing changed — re-derive the pin deliberately"
    );

    // -- P0-2 fixture ladder, pinned counts -------------------------------
    for fixture in &davinci_harness::fixtures::LADDER {
        analyze_source(fixture.source, SfcCroquisOptions::full());
        analyze_source(fixture.source, SfcCroquisOptions::for_compile());
    }
    let after_ladder = differential::stats();
    assert_eq!(
        after_ladder,
        differential::DifferentialStats {
            identifier_comparisons: 24,
            component_reference_comparisons: 10,
            v_for_shapes: 4,
        },
        "ladder comparison counts moved: dispatch, retention, or fixture content changed — re-derive the pin deliberately"
    );

    // -- optional corpus sweep --------------------------------------------
    if let Some(sweep) = davinci_test_support::corpus::resolve_env_sweep() {
        let files = &sweep.files;
        assert!(
            !files.is_empty(),
            "corpus sweep found no .vue files under {}",
            sweep.root.display()
        );
        let mut analyzed = 0u64;
        for file in files {
            let Ok(source) = fs::read_to_string(file) else {
                continue;
            };
            analyze_source(&source, SfcCroquisOptions::full());
            analyzed += 1;
        }
        let end = differential::stats();
        eprintln!(
            "davinci-differential corpus sweep: files={} analyzed={} identifier_comparisons={} component_reference_comparisons={} v_for_shapes={}",
            files.len(),
            analyzed,
            end.identifier_comparisons - after_ladder.identifier_comparisons,
            end.component_reference_comparisons - after_ladder.component_reference_comparisons,
            end.v_for_shapes - after_ladder.v_for_shapes,
        );
        assert!(
            end.identifier_comparisons > after_ladder.identifier_comparisons,
            "a corpus sweep that compares nothing proves nothing"
        );
    }

    let end = differential::stats();
    eprintln!(
        "davinci-differential totals: identifier_comparisons={} component_reference_comparisons={} v_for_shapes={}",
        end.identifier_comparisons, end.component_reference_comparisons, end.v_for_shapes,
    );
}
