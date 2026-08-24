//! Allocation regression probes for compact S1-to-S2 storage.
//!
//! Setup stays outside each measured stage. The `v-for` case therefore
//! accounts lowering's textual split and alias collection, while the `v-on`
//! case accounts only S2 DOM emission, including modifier classification.
//! Exact `allocs` budgets make both probes deterministic and
//! machine-independent. Exact peak-byte budgets are platform-specific; wall
//! time remains report-only until the reference runner records it.

use criterion::{Criterion, criterion_group};
use davinci_harness::stage::bench_stage_with_metrics;
use vize_davinci::pass::NoObserver;
use vize_ricalco::{emit_dom, lower};
use vize_s0::{Allocator, cstr};
use vize_s1::parse;

const VFOR_THREE_ALIASES: &str = r#"<li v-for="(item, key, index) in items">{{ item }}</li>"#;
// v-on-storage-synthetic:start
const VON_TWO_PER_BUCKET: &str =
    r#"<button @keyup.capture.once.stop.prevent.enter.escape="handler"></button>"#;
// v-on-storage-synthetic:end

fn davinci_storage(criterion: &mut Criterion) {
    let vfor_id = cstr!("ricalco_lower_vfor_three_aliases");
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

    let von_id = cstr!("ricalco_emit_von_two_per_bucket");
    bench_stage_with_metrics(
        criterion,
        &von_id,
        "synthetic:v-on-two-option-event-key-modifiers",
        |window| {
            let allocator = Allocator::new();
            let (tree, errors) = parse(&allocator, VON_TWO_PER_BUCKET);
            let mut lowered = lower(&allocator, &tree, &errors);
            let facts = vize_ricalco::pass::run_transform(&mut lowered, &mut NoObserver);
            window.measure(|| {
                let emitted = emit_dom(&lowered, &facts).expect("fixture must emit");
                (emitted.preamble.len(), emitted.code.len())
            })
        },
    );
}

criterion_group!(davinci_storage_group, davinci_storage);
davinci_harness::main!(davinci_storage_group);
