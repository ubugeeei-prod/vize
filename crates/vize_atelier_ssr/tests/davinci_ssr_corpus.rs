//! Davinci P3-8 SSR corpus gate (TS-11 for the SSR lane).
//!
//! Every SFC template compiles on the selected SSR lane and with the legacy
//! AST walker pinned, under the options the production SFC compile derives
//! (script binding metadata, `is_ts`, the scoped-style id, the component
//! name). Code, preamble, source map, and diagnostics must be byte-identical;
//! a broken plan invariant (`rejected`) fails the gate; and the sweep must
//! actually emit from the S4 plan, so a gate that only ever compares the
//! legacy walker against itself proves nothing.
//!
//! `VIZE_DAVINCI_DIFFERENTIAL_CORPUS=<dir>` widens the committed battery to
//! every `.vue` file under `<dir>` (see `davinci_test_support::corpus`).
//!
//! Two sweeps share the input. The emitter sweep derives options by hand and
//! measures what the plan emitter can own; the production sweep
//! (`davinci_ssr_corpus/production.rs`) drives the SFC adapter entry point
//! unchanged and reports the production reach.

#![allow(clippy::disallowed_macros, clippy::disallowed_types)]

use std::collections::BTreeMap;
use std::fs;

use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};
use vize_atelier_ssr::differential::{SsrLaneComparison, compare_ssr_lanes};
use vize_atelier_ssr::{SsrCompilerExperimentalOptions, SsrCompilerOptions};

#[path = "davinci_ssr_corpus/production.rs"]
mod production;

const BATTERY: &[(&str, &str)] = &[
    (
        "Title.vue",
        "<template><title>{{ pageTitle }} — A &amp; B</title></template>",
    ),
    (
        "Template.vue",
        r#"<template><div class="x">{{ msg }}</div></template>"#,
    ),
    (
        "ScriptSetup.vue",
        r#"<script setup lang="ts">
import { ref } from "vue"
const props = defineProps<{ title: string }>()
const count = ref(0)
const items = ref([{ id: 1, label: "a" }])
</script>
<template>
  <section :class="{ busy: count > 0 }">
    <h1 :title="props.title">{{ title }}</h1>
    <ul>
      <li v-for="item in items" :key="item.id">{{ item.label }}</li>
    </ul>
    <p v-if="count">{{ count }}</p>
    <p v-else>none</p>
  </section>
</template>"#,
    ),
    (
        "OptionsApi.vue",
        r#"<script>
export default { data: () => ({ open: false, name: "x" }), methods: { toggle() {} } }
</script>
<template>
  <div v-show="open" @click="toggle">
    <input v-model="name">
    <span v-text="name"></span>
  </div>
</template>"#,
    ),
    (
        "Scoped.vue",
        r#"<template><main><p class="a" :style="{ color }">{{ text }}</p></main></template>
<style scoped>.a { color: red }</style>"#,
    ),
    (
        "Fragment.vue",
        r#"<template>
  <header>top</header>
  <template v-for="(row, i) in rows" :key="i"><b>{{ row }}</b><i>{{ i }}</i></template>
</template>"#,
    ),
];

#[derive(Default)]
struct Report {
    files: u64,
    parsed: u64,
    templates: u64,
    compared: u64,
    legacy_error_skips: u64,
    lanes: BTreeMap<&'static str, u64>,
    rejected: Vec<String>,
    divergences: Vec<String>,
}

impl Report {
    fn emitted(&self) -> u64 {
        self.lanes.get("s4").copied().unwrap_or_default()
    }
}

fn component_name(path: &str) -> vize_s0::String {
    std::path::Path::new(path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("anonymous")
        .into()
}

fn compare_sfc(name: &str, source: &str, report: &mut Report) {
    report.files += 1;
    let Ok(descriptor) = parse_sfc(source, SfcParseOptions::default()) else {
        return;
    };
    report.parsed += 1;
    let Some(template) = descriptor.template.as_ref() else {
        return;
    };
    report.templates += 1;
    let is_ts_lang = |lang: Option<&str>| matches!(lang, Some("ts" | "tsx"));
    let is_ts = descriptor
        .script
        .as_ref()
        .is_some_and(|block| is_ts_lang(block.lang.as_deref()))
        || descriptor
            .script_setup
            .as_ref()
            .is_some_and(|block| is_ts_lang(block.lang.as_deref()));
    let Ok(compiled) = compile_sfc(&descriptor, SfcCompileOptions::default()) else {
        report.legacy_error_skips += 1;
        return;
    };
    let scoped = descriptor.styles.iter().any(|style| style.scoped);
    let options = SsrCompilerOptions {
        scope_id: scoped.then(|| "data-v-5ca1ab1e".into()),
        is_ts,
        binding_metadata: compiled.bindings,
        ..SsrCompilerOptions::default()
    };
    let experimental = SsrCompilerExperimentalOptions {
        component_name: Some(component_name(name)),
        ..SsrCompilerExperimentalOptions::default()
    };
    let comparison = compare_ssr_lanes(&template.content, &options, &experimental);
    record(name, &comparison, report);
}

fn record(name: &str, comparison: &SsrLaneComparison, report: &mut Report) {
    if comparison
        .legacy
        .errors
        .iter()
        .any(|error| !error.is_recoverable())
    {
        report.legacy_error_skips += 1;
        return;
    }
    report.compared += 1;
    *report.lanes.entry(comparison.lane).or_default() += 1;
    if comparison.lane == "rejected" {
        report
            .rejected
            .push(format!("{name}: {:?}", comparison.selected.errors));
    }
    let (selected, legacy) = (&comparison.selected, &comparison.legacy);
    let diverged = selected.result.code != legacy.result.code
        || selected.result.preamble != legacy.result.preamble
        || selected.result.map != legacy.result.map
        || format!("{:?}", selected.errors) != format!("{:?}", legacy.errors);
    if diverged && report.divergences.len() < 20 {
        report.divergences.push(format!(
            "{name} [{}]\n--- selected\n{}{}\n--- legacy\n{}{}",
            comparison.lane,
            selected.result.preamble,
            selected.result.code,
            legacy.result.preamble,
            legacy.result.code,
        ));
    } else if diverged {
        report
            .divergences
            .push(format!("{name} [{}]", comparison.lane));
    }
}

fn assert_clean(label: &str, report: &Report) {
    assert!(
        report.rejected.is_empty(),
        "{label}: broken S4 plan invariants: {:#?}",
        report.rejected
    );
    assert!(
        report.divergences.is_empty(),
        "{label}: {} SSR divergences:\n{}",
        report.divergences.len(),
        report.divergences.join("\n")
    );
    assert!(
        report.emitted() > 0,
        "{label}: a gate that never emits from the S4 plan proves nothing"
    );
}

#[test]
fn ssr_lanes_agree_on_sfc_templates() {
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(ssr_lanes_agree_on_sfc_templates_body)
        .expect("spawn P3-8 SSR corpus thread")
        .join()
        .expect("P3-8 SSR corpus thread must not panic");
}

fn ssr_lanes_agree_on_sfc_templates_body() {
    let mut battery = Report::default();
    for (name, source) in BATTERY {
        compare_sfc(name, source, &mut battery);
    }
    assert_eq!(battery.templates, BATTERY.len() as u64);
    assert_eq!(battery.compared, BATTERY.len() as u64);
    assert_clean("battery", &battery);
    let mut battery_production = production::ProductionReport::default();
    for (name, source) in BATTERY {
        production::compare(name, source, &mut battery_production);
    }
    assert_eq!(battery_production.compared, BATTERY.len() as u64);
    production::assert_clean("battery", &battery_production);

    let Some(sweep) = davinci_test_support::corpus::resolve_env_sweep() else {
        eprintln!("VIZE_DAVINCI_DIFFERENTIAL_CORPUS unset: committed battery only");
        return;
    };
    let mut corpus = Report::default();
    let mut reach = production::ProductionReport::default();
    for file in &sweep.files {
        let Ok(source) = fs::read_to_string(file) else {
            continue;
        };
        let name = file.to_string_lossy();
        compare_sfc(name.as_ref(), &source, &mut corpus);
        production::compare(name.as_ref(), &source, &mut reach);
    }
    eprintln!(
        "davinci SSR production sweep: scope={} templates={} compared={} s4={} error_skips={} rejected={} divergences={}",
        sweep.scope_label(),
        reach.templates,
        reach.compared,
        reach.reach(),
        reach.error_skips,
        reach.rejected.len(),
        reach.divergences.len(),
    );
    eprintln!("davinci SSR production lanes: {:?}", reach.lanes);
    production::assert_clean("production", &reach);
    eprintln!(
        "davinci SSR corpus sweep: scope={} files={} parsed={} templates={} compared={} s4={} legacy_error_skips={} rejected={} divergences={}",
        sweep.scope_label(),
        corpus.files,
        corpus.parsed,
        corpus.templates,
        corpus.compared,
        corpus.emitted(),
        corpus.legacy_error_skips,
        corpus.rejected.len(),
        corpus.divergences.len(),
    );
    eprintln!("davinci SSR corpus lanes: {:?}", corpus.lanes);
    assert!(
        corpus.compared > 0,
        "a corpus sweep that compares nothing proves nothing"
    );
    assert_clean("corpus", &corpus);
}
