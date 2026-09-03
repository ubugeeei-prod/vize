//! Davinci P2-11 DOM corpus-runnable entry under binding metadata
//! (installment 86): the same committed battery and
//! `VIZE_DAVINCI_DIFFERENTIAL_CORPUS=<dir>` widening as
//! `davinci_dom_corpus`, with both lanes compiled in module mode under
//! `prefix_identifiers: true` and the SFC's own (non-inline) binding
//! metadata — the dev-server shape of a `<script setup>` component.

#![allow(clippy::disallowed_macros, clippy::disallowed_types)]

mod davinci_dom_corpus_support;

use davinci_dom_corpus_support::{
    Lane, Report, assert_clean_corpus, assert_empty, compare_sfc_template_lane, compare_sweep_lane,
};

const BATTERY: &[(&str, &str)] = &[
    (
        "template_only",
        r#"<template><div class="x">{{ msg }}</div></template>"#,
    ),
    (
        "script_setup",
        r#"<script setup>const msg = "hi"</script><template><p>{{ msg }}</p></template>"#,
    ),
    (
        "script_setup_bindings",
        r#"<script setup>
import { ref } from "vue"
import Child from "./Child.vue"
const props = defineProps({ title: String })
const count = ref(0)
let draft = ""
function bump() { count.value++ }
const vFocus = { mounted(el) { el.focus() } }
</script>
<template>
  <Child :title="title" @bump="bump" v-focus>{{ count }} {{ draft }} {{ other }}</Child>
  <input v-model="draft" @keyup.enter="bump">
</template>"#,
    ),
    (
        "options_api",
        r#"<script>
export default { data() { return { n: 1 } }, methods: { go() {} } }
</script>
<template><button @click="go">{{ n }}</button></template>"#,
    ),
    (
        "slot_template",
        r#"<template><Foo><template #default="{ item }"><span>{{ item }}{{ other }}</span></template></Foo></template>"#,
    ),
    (
        "handlers_and_loops",
        r#"<template><ul><li v-for="item in items" :key="item.id" @click="select(item, extra)">{{ item.label }}</li></ul></template>"#,
    ),
];

#[test]
fn binding_metadata_dom_emit_agrees_on_sfc_templates() {
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(binding_metadata_dom_emit_agrees_on_sfc_templates_body)
        .expect("spawn P2-11 binding DOM corpus thread")
        .join()
        .expect("P2-11 binding DOM corpus thread must not panic");
}

fn binding_metadata_dom_emit_agrees_on_sfc_templates_body() {
    let mut report = Report::default();
    for (name, source) in BATTERY {
        compare_sfc_template_lane(name, source, &mut report, Lane::Bindings);
    }
    assert_eq!(report.templates, BATTERY.len() as u64);
    assert_eq!(report.old_error_skips, 0, "battery old-lane error skips");
    assert_empty("battery S2 refusals", &report.s2_refusals);
    assert_empty("battery divergences", &report.divergences);

    let Some(sweep) = davinci_test_support::corpus::resolve_env_sweep() else {
        eprintln!("VIZE_DAVINCI_DIFFERENTIAL_CORPUS unset: committed battery only");
        return;
    };
    assert!(
        !sweep.files.is_empty(),
        "corpus sweep found no .vue files under {}",
        sweep.root.display()
    );

    let corpus = compare_sweep_lane(&sweep, Lane::Bindings);
    eprintln!(
        "davinci binding DOM corpus sweep: scope={} files={} unreadable={} parsed={} templates={} compared={} old_error_skips={} s2_refusals={} divergences={}",
        sweep.scope_label(),
        corpus.files,
        corpus.unreadable_count,
        corpus.parsed,
        corpus.templates,
        corpus.compared,
        corpus.old_error_skips,
        corpus.s2_refusal_count,
        corpus.divergence_count,
    );
    eprintln!(
        "davinci binding DOM corpus refusal reasons: {:?}",
        corpus.s2_refusal_reasons
    );
    eprintln!(
        "davinci binding DOM corpus old-lane error reasons: {:?}",
        corpus.old_error_reasons
    );
    eprintln!(
        "davinci binding DOM corpus refusal samples: {:?}",
        corpus.s2_refusal_samples
    );
    assert!(
        corpus.compared > 0,
        "a corpus sweep that compares nothing proves nothing"
    );
    assert_clean_corpus(&corpus);
}
