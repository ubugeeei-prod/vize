//! Vue's public instance includes its re-exported `ComponentCustomProperties`.
//! The fallback fixture must keep declared globals typed: an `any` fallback
//! would hide both an incomplete runtime stub and an invalid argument.

use super::{
    create_cli_project, remove_path_if_exists, resolve_test_corsa_path, write_test_vue_stub,
};
use std::process::Command;

#[test]
fn check_template_dollar_globals_keep_stub_augmentations_typed() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };
    let project_root = create_cli_project(
        "typed-template-dollar-stub",
        &[
            (
                "src/@types/vue-globals.d.ts",
                r#"import "vue";
declare module "vue" {
  interface ComponentCustomProperties {
    $t: (key: string) => string;
  }
}
"#,
            ),
            (
                "src/App.vue",
                "<template>\n  <p>{{ $t(123) }}</p>\n</template>\n",
            ),
        ],
    );
    // Replace only this project's links before writing its minimal packages.
    remove_path_if_exists(&project_root.join("node_modules/vue")).unwrap();
    remove_path_if_exists(&project_root.join("node_modules/@vue")).unwrap();
    write_test_vue_stub(&project_root.join("node_modules")).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "src/App.vue",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("invalid JSON: {error}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });
    assert_eq!(
        serde_json::json!({
            "status": output.status.code(),
            "errorCount": json["errorCount"],
            "files": json["files"],
        }),
        serde_json::json!({
            "status": 1,
            "errorCount": 1,
            "files": [{
                "file": "src/App.vue",
                "diagnostics": ["error:2:12 [TS2345] Argument of type 'number' is not assignable to parameter of type 'string'. (source: <p>{{ $t(123) }}</p>)"],
            }],
        }),
        "stdout:\n{stdout}\nstderr:\n{stderr}",
    );
    let _ = std::fs::remove_dir_all(project_root);
}
