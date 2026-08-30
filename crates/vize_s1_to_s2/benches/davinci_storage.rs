//! Allocation regression probes for S1-to-S2 lowering and S2 DOM emit.
//!
//! Setup stays outside each measured stage. The `v-for` case therefore
//! accounts lowering's textual split and alias collection, while the DOM emit
//! cases account only S2 DOM emission, including modifier classification and
//! the P2-11 late-surface matrix. Exact `allocs` budgets make the probes
//! deterministic and machine-independent. Exact peak-byte budgets are
//! platform-specific; wall time remains report-only until the reference runner
//! records it.

use criterion::{Criterion, criterion_group};
use davinci_harness::stage::bench_stage_with_metrics;
use vize_davinci::pass::NoObserver;
use vize_s0::{Allocator, cstr};
use vize_s1::parse;
use vize_s1_to_s2::{emit_dom, lower};

const VFOR_THREE_ALIASES: &str = r#"<li v-for="(item, key, index) in items">{{ item }}</li>"#;
// v-on-storage-synthetic:start
const VON_TWO_PER_BUCKET: &str =
    r#"<button @keyup.capture.once.stop.prevent.enter.escape="handler"></button>"#;
// v-on-storage-synthetic:end
const P2_11_DOM_SURFACE: &str = r#"
<div :foo-bar.camel="camelValue" v-bind.prop="nativeBag" v-on.once.capture="nativeHandlers">
  <component :is="view" v-model:[field].trim="draft[field]" @update:modelValue="sync">
    <template #header="{ row }" v-if="showHeader">
      <slot :name="row.slot" :item="row" @[row.event].once="row.handler">
        fallback {{ row.label }}
      </slot>
    </template>
    <template v-for="item in items" #default>
      <input v-model.trim="item.value" :key="item.id" v-show="item.visible">
      <span v-text="item.label"></span>
      <div v-html="item.html"></div>
      <p v-cloak>{{ item.note }}</p>
    </template>
  </component>
</div>
"#;

fn davinci_storage(criterion: &mut Criterion) {
    let vfor_id = cstr!("s1_to_s2_lower_vfor_three_aliases");
    bench_stage_with_metrics(
        criterion,
        &vfor_id,
        "synthetic:v-for-three-aliases",
        |window| {
            let allocator = Allocator::new();
            let (tree, errors) = parse(&allocator, VFOR_THREE_ALIASES);
            window.measure(|| {
                let lowered = lower(&allocator, &tree, &errors);
                (
                    lowered.op_count,
                    lowered.diagnostics.len(),
                    lowered.scopes.len(),
                )
            })
        },
    );

    let von_id = cstr!("s1_to_s2_emit_von_two_per_bucket");
    bench_stage_with_metrics(
        criterion,
        &von_id,
        "synthetic:v-on-two-option-event-key-modifiers",
        |window| {
            let allocator = Allocator::new();
            let (tree, errors) = parse(&allocator, VON_TWO_PER_BUCKET);
            let mut lowered = lower(&allocator, &tree, &errors);
            let facts = vize_s1_to_s2::pass::run_transform(&mut lowered, &mut NoObserver);
            window.measure(|| {
                let emitted = emit_dom(&lowered, &facts).expect("fixture must emit");
                (emitted.preamble.len(), emitted.code.len())
            })
        },
    );

    let p2_11_id = cstr!("s1_to_s2_emit_p2_11_dom_surface");
    bench_stage_with_metrics(
        criterion,
        &p2_11_id,
        "synthetic:p2-11-dom-surface",
        |window| {
            let allocator = Allocator::new();
            let (tree, errors) = parse(&allocator, P2_11_DOM_SURFACE);
            let mut lowered = lower(&allocator, &tree, &errors);
            let facts = vize_s1_to_s2::pass::run_transform(&mut lowered, &mut NoObserver);
            window.measure(|| {
                let emitted = emit_dom(&lowered, &facts).expect("fixture must emit");
                (emitted.preamble.len(), emitted.code.len())
            })
        },
    );
}

criterion_group!(davinci_storage_group, davinci_storage);
davinci_harness::main!(davinci_storage_group);
