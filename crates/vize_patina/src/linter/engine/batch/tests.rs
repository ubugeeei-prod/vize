use super::*;
use crate::{LintPreset, Severity};
use serde_json::{Value, json};
use std::path::PathBuf;

fn runtime() -> Option<PathBuf> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../node_modules/@typescript")
        .join(&*vize_l0::cstr!(
            "typescript-{}",
            vize_carton::corsa_resolver::platform_suffix()
        ))
        .join("lib")
        .join(&*vize_l0::cstr!("tsc{}", std::env::consts::EXE_SUFFIX));
    let available = path.is_file();
    assert!(
        available || std::env::var("VIZE_TEST_REQUIRE_TSGO").as_deref() != Ok("1"),
        "batch regressions require the real checker"
    );
    available.then_some(path)
}

fn configured(runtime: &std::path::Path) -> Linter {
    Linter::with_preset(LintPreset::Opinionated)
        .with_corsa_path(Some(runtime.to_owned()))
        .with_type_aware_lint(true)
        .with_enabled_rules(Some(vec![
            "type/no-floating-promises".into(),
            "type/strict-boolean-expressions".into(),
        ]))
}

fn complete(result: &LintResult) -> Value {
    let diagnostics: Vec<_> = result.diagnostics.iter().map(|diagnostic| {
        let labels: Vec<_> = diagnostic.labels.iter().map(|label| json!({"message":label.message.as_str(),"start":label.start,"end":label.end})).collect();
        json!({"rule_name":diagnostic.rule_name,"severity":diagnostic.severity,"message":diagnostic.message.as_str(),"start":diagnostic.start,"end":diagnostic.end,"help":diagnostic.help.as_ref().map(|help|help.as_str()),"labels":labels,"fix":diagnostic.fix})
    }).collect();
    json!({"filename":result.filename.as_str(),"error_count":result.error_count,"warning_count":result.warning_count,"diagnostics":diagnostics})
}

fn floating(source: &str) -> Value {
    let start = source.rfind("save();").unwrap();
    json!({"rule_name":"type/no-floating-promises","severity":"warning","message":"Floating Promise must be awaited, returned, or explicitly ignored with `void`","start":start,"end":start+6,"help":"Add `await`, return the Promise, or prefix it with `void` when the fire-and-forget behavior is intentional.","labels":[],"fix":null})
}

fn boolean(source: &str) -> Value {
    let start = source.find("if (flag)").unwrap() + 4;
    json!({"rule_name":"type/strict-boolean-expressions","severity":"warning","message":"Unexpected object value in conditional. An object is always truthy.","start":start,"end":start+4,"help":null,"labels":[],"fix":null})
}

fn expected(filename: &str, diagnostics: Vec<Value>) -> Value {
    json!({"filename":filename,"error_count":0,"warning_count":diagnostics.len(),"diagnostics":diagnostics})
}

#[test]
fn batch_preserves_complete_pinned_diagnostics_through_edits_and_pug_views() {
    let Some(runtime) = runtime() else { return };
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("package.json"), "{}").unwrap();
    std::fs::write(root.path().join("Child.vue"), CHILD).unwrap();
    let filename = root.path().join("Parent.vue");
    let filename = filename.to_str().unwrap();
    let other = root.path().join("Other.vue");
    let other = other.to_str().unwrap();
    let linters = [configured(&runtime), configured(&runtime)];
    for original in [PARENT, PARENT_PUG, PARENT_JADE] {
        for newline in ["\n", "\r\n"] {
            let source = original.replace('\n', newline);
            let fixed = source
                .replace("save();", "void save();")
                .replace("if (flag)", "if (Boolean(flag))");
            for (current, diagnostics) in [
                (&source, vec![floating(&source), boolean(&source)]),
                (&fixed, vec![]),
                (&source, vec![floating(&source), boolean(&source)]),
            ] {
                let inputs = vec![
                    (
                        linters.first().unwrap(),
                        linters.first().unwrap().prepare_sfc(current, filename),
                    ),
                    (
                        linters.last().unwrap(),
                        linters.last().unwrap().prepare_sfc(OTHER, other),
                    ),
                ];
                let actual: Vec<_> = Linter::lint_prepared_batch(inputs)
                    .iter()
                    .map(complete)
                    .collect();
                assert_eq!(
                    actual,
                    vec![
                        expected(filename, diagnostics),
                        expected(other, vec![floating(OTHER)])
                    ]
                );
            }
        }
    }
    Linter::finish_type_aware_lint(&linters, &[]);
}

#[test]
fn batch_keeps_script_jsx_options_suppression_and_input_order() {
    let Some(runtime) = runtime() else { return };
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("package.json"), "{}").unwrap();
    let linters = [
        configured(&runtime),
        configured(&runtime).with_rule_severity_overrides(vec![(
            "type/no-floating-promises".into(),
            Severity::Error,
        )]),
    ];
    let source = "export {};\n// 😀\nasync function save(): Promise<void> {}\nsave();\n";
    let suppressed = source.replace(
        "save();",
        "// eslint-disable-next-line type/no-floating-promises\nsave();",
    );
    let script = root.path().join("plain.ts");
    let jsx = root.path().join("render.tsx");
    let clean = root.path().join("suppressed.ts");
    let script = script.to_str().unwrap();
    let jsx = jsx.to_str().unwrap();
    let clean = clean.to_str().unwrap();
    let ordinary = linters.first().unwrap();
    let strict = linters.last().unwrap();
    let mut error = floating(source);
    error
        .as_object_mut()
        .unwrap()
        .insert("severity".into(), json!("error"));
    let error_result =
        json!({"filename":jsx,"error_count":1,"warning_count":0,"diagnostics":[error]});
    let inputs = vec![
        (
            strict,
            strict.prepare_jsx(source, jsx, vize_atelier_jsx::JsxLang::Tsx),
        ),
        (ordinary, ordinary.prepare_script(&suppressed, clean)),
        (ordinary, ordinary.prepare_script(source, script)),
    ];
    let actual: Vec<_> = Linter::lint_prepared_batch(inputs)
        .iter()
        .map(complete)
        .collect();
    assert_eq!(
        actual,
        vec![
            error_result,
            expected(clean, vec![]),
            expected(script, vec![floating(source)])
        ]
    );
    Linter::finish_type_aware_lint(&linters, &[]);
}

#[test]
fn batch_does_not_start_a_checker_for_files_without_queries() {
    let absent = std::path::Path::new("/nonexistent-patina-runtime");
    let linter = configured(absent);
    let source = "export {}; const value = 1;";
    let inputs = vec![
        (&linter, linter.prepare_script(source, "/tmp/plain.ts")),
        (
            &linter,
            linter.prepare_sfc("<template><div /></template>", "/tmp/Empty.vue"),
        ),
    ];
    let actual: Vec<_> = Linter::lint_prepared_batch(inputs)
        .iter()
        .map(complete)
        .collect();
    assert_eq!(
        actual,
        vec![
            expected("/tmp/plain.ts", vec![]),
            expected("/tmp/Empty.vue", vec![])
        ]
    );
}

#[test]
fn batch_retains_sfc_directive_and_malformed_source_contracts() {
    let Some(runtime) = runtime() else { return };
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("package.json"), "{}").unwrap();
    std::fs::write(root.path().join("Child.vue"), CHILD).unwrap();
    let filename = root.path().join("Parent.vue");
    let filename = filename.to_str().unwrap();
    let linters = [configured(&runtime)];
    let linter = linters.first().unwrap();
    for marker in [
        "// eslint-disable-next-line type/no-floating-promises",
        "// @vize:ignore",
    ] {
        let source = PARENT.replace("save();", &vize_l0::cstr!("{marker}\nsave();"));
        let diagnostics = if marker.starts_with("// eslint-") {
            vec![boolean(&source)]
        } else {
            vec![floating(&source), boolean(&source)]
        };
        let actual =
            Linter::lint_prepared_batch(vec![(linter, linter.prepare_sfc(&source, filename))]);
        assert_eq!(
            actual.iter().map(complete).collect::<Vec<_>>(),
            vec![expected(filename, diagnostics)]
        );
    }
    for source in [
        "<script setup lang=\"ts\">const n = 1</script><script setup>const n = 2</script><template><div /></template>",
        "<template lang=\"pug\">\ninclude missing.pug\n</template>",
    ] {
        let before = complete(&linter.lint_sfc(source, filename));
        let actual =
            Linter::lint_prepared_batch(vec![(linter, linter.prepare_sfc(source, filename))]);
        assert_eq!(
            actual.iter().map(complete).collect::<Vec<_>>(),
            vec![before]
        );
    }
    Linter::finish_type_aware_lint(&linters, &[]);
}

const PARENT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/retained-projection-10440/Parent.vue.txt"
));
const PARENT_PUG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/retained-projection-10440/Parent.pug.vue.txt"
));
const PARENT_JADE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/retained-projection-10440/Parent.jade.vue.txt"
));
const CHILD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/retained-projection-10440/Child.vue.txt"
));
const OTHER: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/retained-projection-10440/Other.vue.txt"
));
