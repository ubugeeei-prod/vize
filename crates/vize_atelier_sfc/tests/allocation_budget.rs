//! Allocation-budget gate (zero-cost regression guard).
//!
//! The PR wall-clock benchmark cannot see abstraction-cost regressions cheaply
//! — clone churn, boxed closures, dyn dispatch, per-node re-parsing. This test
//! compiles a fixed set of canonical SFCs under a tracking allocator and fails
//! if the allocation count or requested bytes climb past committed ceilings, so
//! the upcoming dialect work (Options API / legacy / petite-vue / class
//! components) cannot quietly add per-byte/per-node allocations.
//!
//! Why a ceiling, not an exact value: allocation counts are deterministic for a
//! given input and toolchain but can drift a little across std/dependency
//! bumps. The budgets carry headroom so unrelated bumps stay green while a real
//! regression (typically tens of percent) trips the gate. Improvements are free
//! and may ratchet the ceilings down.
//!
//! This binary owns the process global allocator, so the counters reflect only
//! its own work. Keep `sfc_compile_stays_within_allocation_budget` the SINGLE
//! test in this file: the counters are process-global, so a second test running
//! concurrently would corrupt the measurement.
//!
//! Re-bless ritual: when a change legitimately shifts allocations, run
//! `cargo test -p vize_atelier_sfc --test allocation_budget -- --nocapture`,
//! read the printed `measured ...`, and update the matching budget below.

#![allow(clippy::disallowed_macros, clippy::disallowed_types)]

use std::alloc::System;

use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};
use vize_carton::profiler::{
    ProfilingAllocator, allocation_snapshot, reset_allocation_counters,
    set_allocation_tracking_enabled,
};

#[global_allocator]
static GLOBAL: ProfilingAllocator<System> = ProfilingAllocator::new();

struct Fixture {
    name: &'static str,
    source: &'static str,
    /// Ceiling for `alloc + alloc_zeroed + realloc` calls during compile.
    max_alloc_calls: u64,
    /// Ceiling for bytes requested by those calls during compile.
    max_requested_bytes: u64,
}

const SCRIPT_SETUP: &str = r#"<script setup lang="ts">
import { ref, computed } from 'vue'
const count = ref(0)
const doubled = computed(() => count.value * 2)
function inc() { count.value++ }
</script>
<template>
  <button @click="inc">{{ count }} / {{ doubled }}</button>
</template>
"#;

const OPTIONS_API: &str = r#"<script>
export default {
  props: { initial: Number },
  data() { return { count: 0 } },
  computed: { doubled() { return this.count * 2 } },
  methods: { inc() { this.count++ } },
}
</script>
<template><div>{{ count }} {{ doubled }} {{ initial }}</div></template>
"#;

const TEMPLATE_HEAVY: &str = r#"<script setup>
import { ref } from 'vue'
const items = ref([1, 2, 3])
const show = ref(true)
</script>
<template>
  <section>
    <h1>Title</h1>
    <ul>
      <li v-for="(item, i) in items" :key="i" :class="{ active: show }">
        <span>{{ item }}</span>
        <em v-if="show">{{ i }}</em>
      </li>
    </ul>
    <footer><p>a</p><p>b</p><p>c</p></footer>
  </section>
</template>
"#;

const FIXTURES: &[Fixture] = &[
    // Budgets carry ~25-30% headroom over the measured steady state
    // (script_setup 185 / 236_041, options_api 110 / 137_333,
    // template_heavy 262 / 327_764) so toolchain drift stays green while a real
    // regression trips the gate.
    //
    // The options_api steady state rose from 72 / 89_285 when the compiler began
    // resolving Options API template bindings: a non-`<script setup>` component
    // now parses its `<script>` a second time to extract `data`/`computed`/
    // `methods`/`props`/`inject` binding metadata for the template prefixer
    // (`$data.` / `$options.` / `$props.`), mirroring the dedicated binding pass
    // `@vue/compiler-sfc` runs. Budgets re-blessed to keep ~25% headroom.
    Fixture {
        name: "script_setup",
        source: SCRIPT_SETUP,
        max_alloc_calls: 235,
        max_requested_bytes: 300_000,
    },
    Fixture {
        name: "options_api",
        source: OPTIONS_API,
        max_alloc_calls: 140,
        max_requested_bytes: 174_000,
    },
    Fixture {
        name: "template_heavy",
        source: TEMPLATE_HEAVY,
        max_alloc_calls: 330,
        max_requested_bytes: 415_000,
    },
];

#[test]
fn sfc_compile_stays_within_allocation_budget() {
    let mut failures = Vec::new();

    // Warm up one-time initializers (lazy statics, thread-locals) untracked so
    // the per-fixture measurements are steady-state and order-independent.
    {
        let descriptor =
            parse_sfc(SCRIPT_SETUP, SfcParseOptions::default()).expect("warmup must parse");
        compile_sfc(&descriptor, SfcCompileOptions::default()).expect("warmup must compile");
    }

    for fixture in FIXTURES {
        // Build the descriptor and options OUTSIDE the measured window so only
        // the compile itself is attributed to the budget.
        let options = SfcCompileOptions::default();
        let descriptor =
            parse_sfc(fixture.source, SfcParseOptions::default()).expect("fixture must parse");

        reset_allocation_counters();
        set_allocation_tracking_enabled(true);
        let result = compile_sfc(&descriptor, options);
        let snapshot = allocation_snapshot();
        set_allocation_tracking_enabled(false);

        result.expect("fixture must compile");

        let calls = snapshot.allocation_calls();
        let bytes = snapshot.requested_bytes();
        println!(
            "{}: measured alloc_calls={calls} requested_bytes={bytes} (budget calls<={} bytes<={})",
            fixture.name, fixture.max_alloc_calls, fixture.max_requested_bytes
        );

        if calls > fixture.max_alloc_calls {
            failures.push(format!(
                "{}: alloc_calls {calls} exceeds budget {}",
                fixture.name, fixture.max_alloc_calls
            ));
        }
        if bytes > fixture.max_requested_bytes {
            failures.push(format!(
                "{}: requested_bytes {bytes} exceeds budget {}",
                fixture.name, fixture.max_requested_bytes
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "SFC compile allocation budget exceeded:\n{}\n\nIf this is an intentional, justified \
         change, re-bless the budgets per this file's header comment.",
        failures.join("\n")
    );
}
