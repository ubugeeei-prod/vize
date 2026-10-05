//! Complete original #7949 CLI vectors, against the pinned genuine Vue/native types.
#![cfg(test)]
#![expect(
    clippy::disallowed_macros,
    reason = "fixture assertions use std strings"
)]
#![expect(clippy::disallowed_types, reason = "fixture IO uses std strings")]
#![expect(clippy::disallowed_methods, reason = "fixture IO uses std strings")]

#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;
#[path = "support/vue_helper_fixture.rs"]
mod fixture;

use serde_json::{Value, json};
use std::{path::Path, process::Command};

const ATTRIBUTE_ERROR: &str =
    "error:2:12 [TS2322] Type '42' is not assignable to type 'Booleanish | undefined'.";
const MODEL_ERROR: &str = "error:6:19 [TS2322] Type 'string' is not assignable to type 'number'.";

fn check(
    root: &Path,
    corsa: &Path,
    vue: &Path,
    case: &str,
    mode: &str,
    model_error: bool,
    attribute_error: bool,
) {
    let app = root.join("apps/web");
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command.current_dir(&app).env("CORSA_PATH", corsa).args([
        "check",
        "--no-config",
        "--format",
        "json",
    ]);
    // A runner-provided Vue override would make root-unresolvable controls fake.
    for name in [
        "VIZE_VUE_PACKAGE",
        "VIZE_VUE_NAMESPACE_PACKAGE",
        "VIZE_VUE_RUNTIME_DOM_PACKAGE",
        "VIZE_RUNTIME_NODE_MODULES",
    ] {
        command.env_remove(name);
    }
    if mode != "default" {
        command.args(["App.vue", "Counter.vue", "Toggle.vue"]);
    }
    if mode == "sharded" {
        command.args(["--servers", "2"]);
    }
    let output = command.output().unwrap();
    if let Some(target) = fixture::capture(root, vue, corsa, case) {
        std::fs::write(target.join(format!("{mode}.stdout.txt")), &output.stdout).unwrap();
        std::fs::write(target.join(format!("{mode}.stderr.txt")), &output.stderr).unwrap();
        std::fs::write(target.join(format!("{mode}.runtime.json")), json!({"cliBinary":env!("CARGO_BIN_EXE_vize"),"exitCode":output.status.code(),"arguments":command.get_args().map(|arg|arg.to_string_lossy()).collect::<Vec<_>>()}).to_string()).unwrap();
        let project = vize_canon::VirtualProject::new(root).unwrap();
        let mirror = project.virtual_root();
        assert!(!mirror.join("__vize_helpers.d.ts").exists());
        for name in [
            "apps/web/__vize_helpers.d.ts",
            "apps/web/tsconfig.json",
            "apps/web/App.vue.ts",
            "apps/web/Counter.vue.ts",
            "apps/web/Toggle.vue.ts",
            "shared/router/Link.vue.ts",
            "shared/router/index.ts",
        ] {
            let output = target.join(mode).join("generated").join(name);
            std::fs::create_dir_all(output.parent().unwrap()).unwrap();
            std::fs::copy(mirror.join(name), output).unwrap();
        }
    }
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        output.status.code(),
        Some(i32::from(model_error || attribute_error)),
        "{output:?}"
    );
    assert_eq!(
        result["errorCount"],
        usize::from(model_error) + usize::from(attribute_error)
    );
    assert_eq!(result["warningCount"], 0);
    assert_eq!(result["fileCount"], 5);
    let mut expected = vec![
        json!({"file":"App.vue","diagnostics":[]}),
        json!({"file":"Counter.vue","diagnostics":if model_error {vec![MODEL_ERROR]} else {vec![]}}),
        json!({"file":"Toggle.vue","diagnostics":if attribute_error {vec![ATTRIBUTE_ERROR]} else {vec![]}}),
        json!({"file":root.join("shared/router/Link.vue"),"diagnostics":[]}),
        json!({"file":root.join("shared/router/index.ts"),"diagnostics":[]}),
    ];
    expected.sort_by(|left, right| left["file"].as_str().cmp(&right["file"].as_str()));
    assert_eq!(result["files"], json!(expected), "{output:?}");
}

fn native_oracle(
    root: &Path,
    corsa: &Path,
    vue: &Path,
    case: &str,
    model_error: bool,
    attribute_error: bool,
) {
    let app = root.join("apps/web");
    let model = if model_error {
        "count.value = 'x';"
    } else {
        "count.value++;"
    };
    let disabled = if attribute_error { "42" } else { "true" };
    let source = format!(
        "import {{ defineModel }} from 'vue';\nconst count = defineModel<number>({{ required: true }});\n{model}\nconst button: import('vue').NativeElements['button'] = {{ disabled: {disabled} }};\n"
    );
    fixture::write(root, "apps/web/oracle.ts", &source);
    fixture::write(
        root,
        "apps/web/oracle.tsconfig.json",
        "{\"compilerOptions\":{\"strict\":true,\"target\":\"ESNext\",\"module\":\"ESNext\",\"moduleResolution\":\"Bundler\",\"noEmit\":true,\"skipLibCheck\":true},\"files\":[\"oracle.ts\"]}\n",
    );
    let output = Command::new(corsa)
        .current_dir(&app)
        .args(["-p", "oracle.tsconfig.json", "--pretty", "false"])
        .output()
        .unwrap();
    if let Some(target) = fixture::capture(root, vue, corsa, case) {
        std::fs::write(target.join("native.stdout.txt"), &output.stdout).unwrap();
        std::fs::write(target.join("native.stderr.txt"), &output.stderr).unwrap();
        std::fs::copy(app.join("oracle.ts"), target.join("oracle.ts")).unwrap();
        std::fs::copy(
            app.join("oracle.tsconfig.json"),
            target.join("oracle.tsconfig.json"),
        )
        .unwrap();
        std::fs::write(target.join("native.runtime.json"), json!({"exitCode":output.status.code(),"arguments":["-p","oracle.tsconfig.json","--pretty","false"]}).to_string()).unwrap();
    }
    let mut expected = String::new();
    if model_error {
        expected.push_str(
            "oracle.ts(3,1): error TS2322: Type 'string' is not assignable to type 'number'.\n",
        );
    }
    if attribute_error {
        expected.push_str("oracle.ts(4,58): error TS2322: Type 'number' is not assignable to type 'Booleanish | undefined'.\n");
    }
    assert_eq!(
        output.status.code(),
        Some(if model_error || attribute_error { 1 } else { 0 })
    );
    assert_eq!(
        String::from_utf8(output.stdout)
            .unwrap()
            .replace("\r\n", "\n"),
        expected
    );
    assert_eq!(output.stderr, Vec::new());
    // Keep the native oracle separate from the original CLI program membership.
    std::fs::remove_file(app.join("oracle.ts")).unwrap();
    std::fs::remove_file(app.join("oracle.tsconfig.json")).unwrap();
}

#[test]
fn original_outside_alias_sfc_keeps_models_callable_and_native_attributes_checked() {
    let Some(corsa) = corsa_requirement::required_or_skip::<std::path::PathBuf>(None) else {
        return;
    };
    let version = Command::new(&corsa).arg("--version").output().unwrap();
    assert_eq!(version.status.code(), Some(0));
    assert_eq!(version.stdout, b"Version 7.0.2\n");
    assert_eq!(version.stderr, Vec::new());
    for root_vue in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let vue = fixture::fixture(&root, root_vue);
        let case = if root_vue {
            "original-root-vue-present"
        } else {
            "original-root-vue-absent"
        };
        native_oracle(&root, &corsa, &vue, case, false, true);
        for mode in ["explicit", "default", "sharded"] {
            check(&root, &corsa, &vue, case, mode, false, true);
        }
    }
}

#[test]
fn outside_alias_keeps_valid_attributes_and_invalid_model_assignments_typed() {
    let Some(corsa) = corsa_requirement::required_or_skip::<std::path::PathBuf>(None) else {
        return;
    };
    for invalid_model in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let vue = fixture::fixture(&root, false);
        let case = if invalid_model {
            "invalid-model"
        } else {
            "valid-disabled"
        };
        fixture::write(
            &root,
            "apps/web/Toggle.vue",
            &fixture::TOGGLE.replace("\"42\"", "\"true\""),
        );
        if invalid_model {
            fixture::write(
                &root,
                "apps/web/Counter.vue",
                &fixture::COUNTER.replace("count++", "count = 'x'"),
            );
        }
        native_oracle(&root, &corsa, &vue, case, invalid_model, false);
        for mode in ["explicit", "default"] {
            check(&root, &corsa, &vue, case, mode, invalid_model, false);
        }
    }
}
