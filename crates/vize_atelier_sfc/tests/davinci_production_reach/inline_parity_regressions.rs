//! Inline production compiles must retain transform-time scope and helper order.

use super::shapes::{Shape, compile};
use sha2::{Digest, Sha256};
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_l0::profiler::global_profiler;

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

#[test]
fn typed_slot_handlers_keep_the_current_scope_and_outside_cache() {
    let _guard = crate::PROFILER_TEST_LOCK.lock().unwrap();
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../tests/_fixtures/differential/compiler/typed-slot-handler-scope-8142/cases.json"
    ))
    .expect("whole independent typed scope controls");
    let cases = fixture["cases"]
        .as_array()
        .expect("complete six authored controls");
    assert_eq!(cases.len(), 6);
    for case in cases {
        let filename = case["filename"].as_str().expect("authored filename");
        let source = case["source"].as_str().expect("whole authored SFC");
        assert_inline_parity(filename, source);
    }
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
    let emitted = compile(&descriptor, filename, Shape::DomInline);
    let counters = profiler.counter_summary();
    profiler.disable();
    profiler.clear();
    let legacy = vize_atelier_dom::differential::with_legacy_lane(|| {
        compile(&descriptor, filename, Shape::DomInline)
    });
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/differential/typed-slot-handler-scope");
    std::fs::create_dir_all(&root).expect("whole scope compiler packet directory");
    let executable = std::env::current_exe().expect("actual source test producer");
    let binary = std::fs::read(&executable).expect("actual source test producer bytes");
    let binary_sha256 = Sha256::digest(binary)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let packet = serde_json::json!({
        "filename": filename, "source": source, "shape": "dom_inline",
        "producer": { "executable": executable, "binarySha256": binary_sha256,
            "githubSha": std::env::var("GITHUB_SHA").ok(),
            "argv": std::env::args().collect::<Vec<_>>() },
        "counters": format!("{counters:?}"),
        "selectedResult": emitted, "legacyResult": legacy,
    });
    std::fs::write(
        root.join(format!("{filename}.json")),
        serde_json::to_vec_pretty(&packet).expect("complete original compiler outputs"),
    )
    .expect("retain whole actual packet before parity assertion");
    assert!(
        counters
            .entries
            .iter()
            .any(|entry| entry.name == "davinci.s2_dom.accepted" && entry.total == 1),
        "{filename} must reach L2: {counters:?}"
    );
    let emitted = emitted.expect("L2 compile");
    let legacy = legacy.expect("legacy compile");
    assert_eq!(emitted.code, legacy.code, "{filename}");
}
