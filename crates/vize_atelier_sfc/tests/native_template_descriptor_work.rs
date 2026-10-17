//! Isolated old/default whole-SFC allocation and existing parse/walk counters.
//! This is the only test in this executable: the existing counters are global.
#![cfg(not(target_arch = "wasm32"))]
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "measurement evidence uses serde JSON and original source strings"
)]

use davinci_harness::alloc::{self, CountingAllocator};
use serde_json::{Value, json};
use vize_atelier_core::{
    CodegenOptions, TemplateSyntaxMode, expr_parse_probe, options::CustomElementMatcher,
    walk_probe::WalkCounts,
};
use vize_atelier_sfc::{
    SfcCompileExperimentalOptions, SfcCompileOptions, SfcCompileResult, SfcError, SfcParseOptions,
    SfcScriptOutputMode, compile_sfc_for_adapter_with_experimental_options, parse_sfc,
};

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator::mimalloc();

const FILENAME: &str = "/workspace/original-controls/NativeDescriptor雪🌸.vue";

fn whole(source: &str, shape: &str, retained: bool) -> Result<SfcCompileResult, SfcError> {
    let parse = SfcParseOptions {
        filename: FILENAME.into(),
        ..Default::default()
    };
    let descriptor = if retained {
        vize_croquis::sfc::parse_sfc(source, parse)
    } else {
        parse_sfc(source, parse)
    }?;
    let mut options = SfcCompileOptions::default();
    options.parse.filename = FILENAME.into();
    options.script.id = Some(FILENAME.into());
    options.script.inline_template = shape == "dom-inline";
    options.template.ssr = shape == "ssr";
    options.vapor = shape == "vapor";
    compile_sfc_for_adapter_with_experimental_options(
        &descriptor,
        options,
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
        if shape == "dom-inline" {
            SfcScriptOutputMode::InlineTemplate
        } else {
            SfcScriptOutputMode::SeparateTemplate
        },
        SfcCompileExperimentalOptions::default(),
    )
}

fn observe(source: &str, shape: &str, retained: bool, warm: bool) -> (Value, Value) {
    // "Cold" means an empty existing per-worker arena pool, not a claim to
    // reset process-global library initialization or operating-system caches.
    vize_l0::pool::clear();
    if warm {
        drop(whole(source, shape, retained));
    }
    let before_walks = WalkCounts::snapshot();
    let before_expressions = expr_parse_probe::expr_parse_count();
    let (result, allocations) = alloc::measure_returning(|| whole(source, shape, retained));
    let expressions = expr_parse_probe::expr_parse_count() - before_expressions;
    let walks = WalkCounts::snapshot().since(before_walks);
    let allocations = allocations.expect("actual installed existing counting allocator");
    // Serialization and result destruction stay outside the measured window.
    let output = serde_json::to_value(result).expect("complete public Result");
    (
        output,
        json!({
            "allocationCalls": allocations.calls,
            "peakBytesOverStart": allocations.peak_bytes_over_start,
            "legacyTemplateExpressionParses": expressions,
            "stageWalks": walks.walks,
            "stageVisits": walks.visits,
        }),
    )
}

#[test]
fn old_and_default_whole_sfc_paths_retain_results_and_counted_work() {
    alloc::mark_installed();
    let rich = format!(
        "<template><ul>{}</ul></template>",
        "<li class='item' title='雪🌸' data-kind='literal'>text</li>".repeat(64)
    );
    let cases = [
        ("admitted", "<template><div/></template>"),
        ("admitted-attributes", rich.as_str()),
        (
            "guard-script",
            "<script setup>const value=1;</script><template><p>{{value}}</p></template>",
        ),
        (
            "guard-style",
            "<template><p/></template><style scoped>.p{color:red}</style>",
        ),
        (
            "native-refused-v-pre",
            "<template><div v-pre>{{</div></template>",
        ),
    ];
    let shapes = ["dom-inline", "dom-module", "vapor", "ssr"];
    // Initialize shared library caches equally before the arena-cold/warm pairs.
    for (_, source) in cases {
        for shape in shapes {
            drop(whole(source, shape, true));
            drop(whole(source, shape, false));
        }
    }
    let mut rows = Vec::new();
    let mut failures = Vec::new();
    for (id, source) in cases {
        for shape in shapes {
            for warm in [false, true] {
                let (old, old_work) = observe(source, shape, true, warm);
                let (new, new_work) = observe(source, shape, false, warm);
                if old != new {
                    failures.push(format!("{id}/{shape}/{warm}: complete Result"));
                }
                for field in [
                    "legacyTemplateExpressionParses",
                    "stageWalks",
                    "stageVisits",
                ] {
                    if old_work[field] != new_work[field] {
                        failures.push(format!("{id}/{shape}/{warm}: {field}"));
                    }
                }
                rows.push(json!({"id": id, "source": source, "shape": shape,
                    "arena": if warm { "warm" } else { "cold" },
                    "old": { "result": old, "work": old_work },
                    "default": { "result": new, "work": new_work }}));
            }
        }
    }
    let report = json!({"schema": "vize.sfc.native-descriptor-controlled-proof",
        "githubSha": std::env::var("GITHUB_SHA").ok(),
        "githubRunId": std::env::var("GITHUB_RUN_ID").ok(),
        "filename": FILENAME,
        "scope": "Same executable, untouched retained parser versus actual default parser, with identical complete downstream compilation. Allocation counters are direct existing harness observations, not timing/cap changes or native-only compiler credit. Cold/warm refer only to the existing arena pool. Valid v-pre refusal adds a lexical native boundary attempt before retained fallback.",
        "rows": rows, "failures": failures});
    // Reuse the existing Rust-cohort evidence directory; no workflow change.
    if let Ok(profile) = std::env::var("NEXTEST_PROFILE") {
        assert!(matches!(profile.as_str(), "pr" | "full"));
        let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/nextest")
            .join(profile);
        std::fs::create_dir_all(&directory).expect("existing cohort evidence directory");
        std::fs::write(
            directory.join("native-template-descriptor-controlled-proof.json"),
            serde_json::to_vec_pretty(&report).expect("complete evidence JSON"),
        )
        .expect("preserve all original controlled measurements before assertions");
    }
    assert_eq!(
        report["rows"].as_array().expect("all measured pairs").len(),
        40
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
