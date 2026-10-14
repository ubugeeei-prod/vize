#![cfg(test)]
#![expect(
    clippy::disallowed_types,
    reason = "whole CLI fixtures use std strings"
)]

use std::{fs, path::Path, process::Command};
use vize_atelier_core::parser::with_vue2_preserve_whitespace;
use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};

const SOURCE: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/whitespace-vue2-preserve-8259/App.vue.txt"
);
const CONFIG: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/whitespace-vue2-preserve-8259/vize.config.json.txt"
);
const VUE27_RENDER: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/whitespace-vue2-preserve-8259/vue27-preserve-render.txt"
);

/// The text runs Vue 2.7.16 `preserve` emits for the fixture, as they appear
/// in Vue 3 codegen (`_vm._s(_vm.x)` becomes `_toDisplayString(x)`).
const VUE27_TEXT_RUNS: [(&str, &str); 2] = [
    (
        r#"_vm._v("\n      " + _vm._s(_vm.n) + "件\n    ")"#,
        r#""\n      " + _toDisplayString(n) + "件\n    ""#,
    ),
    (
        r#"_vm._v("\n    " + _vm._s(_vm.b) + "\n    ")"#,
        r#""\n    " + _toDisplayString(b) + "\n    ""#,
    ),
];

#[test]
fn configured_vue2_preserve_matches_vue_2_7_text_runs() {
    for (vue2, vue3) in VUE27_TEXT_RUNS {
        assert!(VUE27_RENDER.contains(vue2), "reference lost {vue2}");
        let built = build(false);
        assert!(built.contains(vue3), "missing {vue3} in:\n{built}");
    }
    let built = build(false);
    assert!(
        !built.contains(r#"" " + _toDisplayString(b) + " ""#),
        "Vue 3 preserve output must not be produced"
    );
    assert_eq!(built, compiler_code(false, false));
}

#[test]
fn configured_vue2_preserve_reaches_ssr_and_vapor() {
    for (ssr, vapor) in [(true, false), (false, true)] {
        let built = build_with(ssr, vapor);
        assert_eq!(built, compiler_code(ssr, vapor), "ssr={ssr}, vapor={vapor}");
    }
}

fn build(ssr: bool) -> String {
    build_with(ssr, false)
}

fn build_with(ssr: bool, vapor: bool) -> String {
    let project = tempfile::tempdir().unwrap();
    fs::write(project.path().join("App.vue"), SOURCE).unwrap();
    fs::write(project.path().join("vize.config.json"), CONFIG).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command
        .current_dir(project.path())
        .env("RAYON_NUM_THREADS", "1")
        .args([
            "build",
            "--slow-threshold",
            "600000",
            "-o",
            "out",
            "-f",
            "js",
            "App.vue",
        ]);
    if ssr {
        command.arg("--ssr");
    }
    if vapor {
        command.arg("--vapor");
    }
    let output = command.output().unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    fs::read_to_string(Path::new(project.path()).join("out/App.js")).unwrap()
}

fn compiler_code(ssr: bool, vapor: bool) -> String {
    let descriptor = parse_sfc(
        SOURCE,
        SfcParseOptions {
            filename: "App.vue".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let mut options = SfcCompileOptions::default();
    options.parse.filename = "App.vue".into();
    options.script.id = Some("App.vue".into());
    options.template.id = Some("App.vue".into());
    options.template.ssr = ssr;
    options.style.id = "App.vue".into();
    options.vapor = vapor;
    let result = with_vue2_preserve_whitespace(|| compile_sfc(&descriptor, options)).unwrap();
    assert!(result.errors.is_empty());
    result.code.as_str().into()
}
