#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;
#[path = "support/nuxt_cli.rs"]
mod nuxt_cli;

use std::{fs, process::Command};

use nuxt_cli::resolve_test_corsa_path;

#[test]
fn self_contained_wrapper_preserves_authored_option_diagnostics() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let case = tempfile::tempdir().unwrap();
    let cases = [
        (
            "root-dirs",
            r#""rootDirs": "not-an-array", "paths": { "~/*": ["src/*"] }"#,
            "Compiler option 'rootDirs' requires a value of type Array.",
        ),
        (
            "base-url",
            r#""baseUrl": 42, "paths": { "~/*": ["src/*"] }"#,
            "Compiler option 'baseUrl' requires a value of type string.",
        ),
        (
            "paths",
            r#""paths": "not-an-object""#,
            "Compiler option 'paths' requires a value of type object.",
        ),
        (
            "ignore-deprecations",
            r#""ignoreDeprecations": 42, "paths": { "~/*": ["src/*"] }"#,
            "Compiler option 'ignoreDeprecations' requires a value of type string.",
        ),
    ];
    for (name, compiler_options, expected) in cases {
        assert_authored_option_diagnostic(
            &case.path().join(name),
            &corsa_path,
            compiler_options,
            expected,
        );
    }
}

fn assert_authored_option_diagnostic(
    project: &std::path::Path,
    corsa_path: &std::path::Path,
    compiler_options: &str,
    expected: &str,
) {
    fs::create_dir_all(project.join("src")).unwrap();
    fs::write(project.join("nuxt.config.ts"), "export default {}\n").unwrap();
    fs::write(
        project.join("tsconfig.json"),
        format!(
            r##"{{
  "compilerOptions": {{
    "strict": true,
    "module": "ESNext",
    "moduleResolution": "bundler",
    {compiler_options}
  }}
}}"##
        ),
    )
    .unwrap();
    fs::write(
        project.join("src/App.vue"),
        "<script setup lang=\"ts\">const value = 1; void value;</script>\n",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "src/App.vue",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
            "--no-config",
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let value: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|error| panic!("{error}\nstdout:\n{stdout}\nstderr:\n{stderr}"));
    let config = value["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|file| file["file"] == "tsconfig.json")
        .unwrap_or_else(|| {
            panic!("authored config must own its diagnostic\nstdout:\n{stdout}\nstderr:\n{stderr}")
        });
    assert_eq!(
        config["diagnostics"],
        serde_json::json!([format!("error:1:1 [TS5024] {expected}")])
    );
}
