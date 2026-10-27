#![cfg(test)]
#![expect(
    clippy::disallowed_types,
    reason = "complete fixture packets use std strings"
)]
#![expect(clippy::disallowed_macros, reason = "fixture packet formatting")]
use std::{fs, path::Path, process::Command, time::Instant};
use vize_croquis::{
    Analyzer, AnalyzerOptions, EffectGraphScript, build_effect_graph_from_sfc_scripts,
};
use vize_croquis_cf::{CrossFileAnalyzer, CrossFileOptions};

const KEYS: &str = "import type { InjectionKey, Ref } from \"vue\";\nexport const PlainKey: InjectionKey<Ref<string | null>> = Symbol(\"plain\");\nexport const CastKey: InjectionKey<Ref<string>> = Symbol(\"cast\");\n";
const ORIGINAL: &str = "import { computed, provide, ref } from \"vue\";\nimport type { Ref } from \"vue\";\nimport MyChild from \"./components/MyChild.vue\";\nimport { CastKey, PlainKey } from \"./composables/keys\";\n\nconst source = ref<string | null>(\"a\");\nconst value = computed(() => source.value);\nprovide(PlainKey, value);\n// the same computed ref, narrowed with a type assertion\nprovide(CastKey, value as Ref<string>);\n";
const CHILD: &str = "import { inject } from \"vue\";\nimport { CastKey, PlainKey } from \"../composables/keys\";\n\nconst plain = inject(PlainKey)!;\nconst cast = inject(CastKey)!;\n";
const SETUP: &str = "import { computed, provide, ref, reactive, readonly } from 'vue';\nimport type { Ref } from 'vue';\nimport MyChild from './components/MyChild.vue';\nimport { CastKey, PlainKey } from './composables/keys';\nconst source = ref('a');\nconst value = computed(() => source.value);\nconst state = reactive({ count: 1 });\nconst plainValue = 'a';\nconst snapshot = source.value;\nfunction getValue() { return 'a'; }\nfunction useService() { return { value: source }; }\nprovide(PlainKey, value);\n";

#[test]
fn observe_complete_provide_assertion_packets() {
    let owned_artifacts = tempfile::tempdir().unwrap();
    let artifacts = std::env::var_os("VIZE_PROVIDE_ASSERTION_ARTIFACTS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| owned_artifacts.path().to_owned());
    let artifacts = artifacts.as_path();
    fs::create_dir_all(artifacts).unwrap();
    let cases = [
        ("original", "value as Ref<string>", false),
        ("computed-plain", "value", false),
        ("computed-as", "value as Ref<string>", false),
        ("computed-parenthesized", "(value)", false),
        (
            "computed-nested",
            "((value as unknown) as Ref<string>)",
            false,
        ),
        ("computed-satisfies", "value satisfies Ref<string>", false),
        ("computed-nonnull", "value!", false),
        ("computed-angle", "<Ref<string>>value", false),
        ("ref-as", "source as Ref<string>", false),
        ("reactive-as", "state as typeof state", false),
        ("ref-generic-call", "ref<string>('a') as Ref<string>", false),
        (
            "computed-call",
            "(computed(() => source.value)) as Ref<string>",
            false,
        ),
        ("readonly-call", "readonly(source) as Ref<string>", false),
        ("literal-plain", "'a'", false),
        ("literal-fake-ref", "'a' as unknown as Ref<string>", false),
        (
            "binding-fake-ref",
            "plainValue as unknown as Ref<string>",
            false,
        ),
        (
            "object-fake-ref",
            "({ value: 'a' }) as unknown as Ref<string>",
            false,
        ),
        (
            "snapshot-member",
            "source.value as unknown as Ref<string>",
            false,
        ),
        (
            "snapshot-binding",
            "snapshot as unknown as Ref<string>",
            false,
        ),
        (
            "unknown-call",
            "getValue() as unknown as Ref<string>",
            false,
        ),
        (
            "mixed-conditional",
            "(true ? source : plainValue) as unknown as Ref<string>",
            false,
        ),
        ("destructure-ref-plain", "source", true),
        ("destructure-ref-as", "source as Ref<string>", true),
        ("destructure-fake-plain", "({ value: 'a' })", true),
        (
            "destructure-fake-as",
            "({ value: 'a' }) as unknown as Ref<string>",
            true,
        ),
        (
            "service-object-as",
            "({ source, update: () => {} }) as unknown",
            false,
        ),
        ("service-composable-as", "useService() as unknown", false),
    ];
    let mut manifest = Vec::new();
    let mut observations = Vec::new();
    for (id, expression, destructure) in cases {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let script = if id == "original" {
            ORIGINAL.to_owned()
        } else {
            format!("{SETUP}provide(CastKey, {expression});\n")
        };
        let child = if destructure {
            CHILD.replace(
                "const cast = inject(CastKey)!;",
                "const { value: cast } = inject(CastKey)!;",
            )
        } else {
            CHILD.to_owned()
        };
        let keys = if id == "original" {
            KEYS.to_owned()
        } else {
            KEYS.replace("InjectionKey<Ref<string | null>>", "InjectionKey<unknown>")
                .replace("InjectionKey<Ref<string>>", "InjectionKey<unknown>")
        };
        write(root, "src/composables/keys.ts", &keys);
        write(
            root,
            "src/App.vue",
            &format!(
                "<script setup lang=\"ts\">\n{script}</script>\n\n<template>\n  <MyChild />\n</template>\n"
            ),
        );
        write(
            root,
            "src/components/MyChild.vue",
            &format!(
                "<script setup lang=\"ts\">\n{child}</script>\n\n<template>\n  <p>{{{{ plain }}}} {{{{ cast }}}}</p>\n</template>\n"
            ),
        );
        let target = artifacts.join(id);
        fs::create_dir_all(&target).unwrap();
        for source in [
            "src/composables/keys.ts",
            "src/App.vue",
            "src/components/MyChild.vue",
        ] {
            write(
                &target.join("fixture"),
                source,
                &fs::read_to_string(root.join(source)).unwrap(),
            );
        }
        let mut native = CrossFileAnalyzer::new(
            CrossFileOptions::default()
                .with_provide_inject(true)
                .with_reactivity_tracking(true),
        );
        native.add_file("src/composables/keys.ts", &keys);
        add_script(&mut native, "src/App.vue", &script, &["MyChild"]);
        add_script(&mut native, "src/components/MyChild.vue", &child, &[]);
        native.rebuild_component_edges();
        let mut result = native.analyze();
        fs::write(target.join("native.raw.txt"), format!("{result:#?}\n")).unwrap();
        let analysis_time_ms = result.stats.analysis_time_ms;
        result.stats.analysis_time_ms = 0.0;
        let canonical = format!("{result:#?}\n");
        fs::write(target.join("native.complete.txt"), &canonical).unwrap();
        fs::write(
            target.join("native.registry.txt"),
            format!("{:#?}\n", native.registry()),
        )
        .unwrap();
        fs::write(
            target.join("native.graph.txt"),
            format!("{:#?}\n", native.graph()),
        )
        .unwrap();
        let mut processes = Vec::new();
        let mut reports = Vec::new();
        let mut statuses = Vec::new();
        for (name, args) in [
            ("doctor", vec!["doctor", "--format", "json", "."]),
            (
                "lint",
                vec![
                    "lint",
                    "-f",
                    "json",
                    "--no-config",
                    "--cross-file",
                    "src/**/*.vue",
                ],
            ),
        ] {
            let start = Instant::now();
            let output = Command::new(env!("CARGO_BIN_EXE_vize"))
                .current_dir(root)
                .args(&args)
                .output()
                .unwrap();
            let elapsed_micros = start.elapsed().as_micros();
            fs::write(target.join(format!("{name}.stdout")), &output.stdout).unwrap();
            fs::write(target.join(format!("{name}.stderr")), &output.stderr).unwrap();
            let packet: serde_json::Value =
                serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
                    panic!(
                        "{id}/{name}: {error}: {}",
                        String::from_utf8_lossy(&output.stdout)
                    )
                });
            fs::write(
                target.join(format!("{name}.json")),
                serde_json::to_vec_pretty(&packet).unwrap(),
            )
            .unwrap();
            reports.push(packet);
            statuses.push(output.status.code());
            processes.push(serde_json::json!({
                "command": name, "arguments": args, "exitCode": output.status.code(),
                "elapsedMicros": elapsed_micros, "signal": output.status.code().is_none()
            }));
        }
        let packet = serde_json::json!({
            "id":id, "expression":expression, "destructure":destructure,
            "nativeAnalysisTimeMs": analysis_time_ms, "processes":processes,
            "nativeTimingNormalization":"Only stats.analysis_time_ms set to 0 in native.complete.txt; untouched native.raw.txt retained"
        });
        fs::write(
            target.join("processes.json"),
            serde_json::to_vec_pretty(&packet).unwrap(),
        )
        .unwrap();
        observations.push(serde_json::json!({"id":id,"native":canonical,"doctor":reports[0],"lint":reports[1],"statuses":statuses}));
        manifest.push(packet);
    }
    fs::write(
        artifacts.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
    assert_eq!(manifest.len(), 27);
    fs::write(
        artifacts.join("complete-observations.json"),
        serde_json::to_vec_pretty(&observations).unwrap(),
    )
    .unwrap();
    let expected: serde_json::Value =
        serde_json::from_str(include_str!("provide_type_assertion_expected.json")).unwrap();
    assert_eq!(
        serde_json::to_value(observations).unwrap(),
        expected,
        "all native, doctor, lint and status fields"
    );
}

fn add_script(analyzer: &mut CrossFileAnalyzer, path: &str, script: &str, used: &[&str]) {
    let mut local = Analyzer::with_options(AnalyzerOptions::full());
    local.analyze_script_setup(script);
    for component in used {
        local
            .croquis_mut()
            .note_used_component(vize_l0::CompactString::new(*component));
    }
    let summary =
        build_effect_graph_from_sfc_scripts(None, Some(EffectGraphScript::new(script, Some("ts"))))
            .summary();
    analyzer.add_file_with_analysis_and_effect_summary(path, script, local.finish(), summary);
}
fn write(root: &Path, file: &str, source: &str) {
    let file = root.join(file);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(file, source).unwrap();
}
