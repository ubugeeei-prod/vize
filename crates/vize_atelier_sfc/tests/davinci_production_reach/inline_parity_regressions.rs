//! Inline production compiles must retain transform-time scope and helper order.

use super::shapes::{Shape, compile};
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_s0::profiler::global_profiler;

#[test]
fn inline_scope_and_helper_order_match_shipped_compiler() {
    let _guard = crate::PROFILER_TEST_LOCK.lock().unwrap();
    assert_inline_parity(
        "for-value-shadow.vue",
        r#"<script setup lang="ts">
const props = defineProps<{ version: string }>()
const visibleDeps = [['dep', '1.0.0']]
</script>
<template><ul><li v-for="[dep, version] in visibleDeps" :key="dep"><LinkBase :to="dep">{{ version }}</LinkBase></li></ul></template>"#,
    );
    assert_inline_parity(
        "model-before-for.vue",
        r#"<script setup lang="ts">
import { ref } from 'vue'
let activeTab = ref('a')
const tabs = ['a', 'b']
</script>
<template><select v-model="activeTab"><option v-for="tab in tabs" :key="tab">{{ tab }}</option></select></template>"#,
    );
    assert_inline_parity(
        "transition-slot-before-for.vue",
        r#"<script setup lang="ts">
import { ref } from 'vue'
let label = ref('ready')
const items = [1, 2]
</script>
<template><PageWithHeader><div v-if="items.length">{{ label }}<TransitionGroup><div v-for="item in items" :key="item">{{ item }}</div></TransitionGroup></div><template #footer><Transition><div v-show="items.length">{{ label }}</div></Transition></template></PageWithHeader></template>"#,
    );
    assert_inline_parity(
        "loop-callback-unref.vue",
        r#"<script setup lang="ts">
import { getKey } from './key'
const items = [{ id: 1 }]
</script>
<template><div><template v-for="item in items" :key="getKey(item)"><span>{{ getKey(item) }}</span></template></div></template>"#,
    );
    assert_inline_parity(
        "show-before-loop.vue",
        r#"<script setup lang="ts">
import { shown } from './shown'
const items = [1, 2]
</script>
<template><Transition><div v-show="shown"><span v-for="item in items" :key="item">{{ item }}</span></div></Transition></template>"#,
    );
    assert_inline_parity(
        "runtime-directive-before-loop.vue",
        r#"<script setup lang="ts">
import { options } from './options'
const items = [1, 2]
</script>
<template><div v-draggable="options"><TransitionGroup><span v-for="item in items" :key="item">{{ item }}</span></TransitionGroup></div></template>"#,
    );
}

fn assert_inline_parity(filename: &str, source: &str) {
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: filename.into(),
            ..Default::default()
        },
    )
    .expect("fixture parses");
    let profiler = global_profiler();
    profiler.clear();
    profiler.enable();
    let emitted = compile(&descriptor, filename, Shape::DomInline).expect("S2 compile");
    let counters = profiler.counter_summary();
    profiler.disable();
    profiler.clear();
    assert!(
        counters
            .entries
            .iter()
            .any(|entry| entry.name == "davinci.s2_dom.accepted" && entry.total == 1),
        "{filename} must reach S2: {counters:?}"
    );
    let legacy = vize_atelier_dom::differential::with_legacy_lane(|| {
        compile(&descriptor, filename, Shape::DomInline)
    })
    .expect("legacy compile");
    assert_eq!(emitted.code, legacy.code, "{filename}");
}
