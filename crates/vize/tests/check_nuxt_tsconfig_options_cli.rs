#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;
#[path = "support/nuxt_cli.rs"]
mod nuxt_cli;

use std::{fs, path::Path, process::Command};

use nuxt_cli::resolve_test_corsa_path;

#[test]
fn generated_nuxt_paths_preserve_explicit_unknown_template_checks() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let case = tempfile::tempdir().unwrap();
    let root = case.path().canonicalize().unwrap();
    write(
        &root,
        "src/Child.vue",
        include_str!(
            "../../../tests/_fixtures/differential/typechecker/unknown-template-options/src/Child.vue.txt"
        ),
    );
    write(
        &root,
        "src/Parent.vue",
        include_str!(
            "../../../tests/_fixtures/differential/typechecker/unknown-template-options/src/Parent.vue.txt"
        ),
    );
    write(
        &root,
        "tsconfig.json",
        include_str!(
            "../../../tests/_fixtures/differential/typechecker/unknown-template-options/tsconfig.json.txt"
        ),
    );
    link_fixture_vue(&root);
    fs::write(root.join("nuxt.config.ts"), "export default {};\n").unwrap();
    fs::create_dir_all(root.join(".nuxt/types")).unwrap();
    fs::write(
        root.join(".nuxt/tsconfig.json"),
        r#"{ "compilerOptions": { "paths": { "~/*": ["../src/*"] } } }"#,
    )
    .unwrap();
    fs::write(root.join(".nuxt/types/imports.d.ts"), "export {};\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&root)
        .env("CORSA_PATH", &corsa_path)
        .args(["check", "--no-config", "--format", "json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(1),
        "Nuxt wrapper must retain explicit Vue checks\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let value: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(value["errorCount"], serde_json::json!(3), "{stdout}");
    let diagnostics = value["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|file| file["file"] == "src/Parent.vue")
        .and_then(|file| file["diagnostics"].as_array())
        .unwrap_or_else(|| panic!("missing Parent.vue diagnostics: {stdout}"));
    assert_eq!(diagnostics.len(), 3, "{stdout}");
    for (code, expected) in [("[TS2353]", 1), ("[TS2339]", 2)] {
        let actual = diagnostics
            .iter()
            .filter_map(serde_json::Value::as_str)
            .filter(|diagnostic| diagnostic.contains(code))
            .count();
        assert_eq!(actual, expected, "unexpected {code} diagnostics: {stdout}");
    }
}

fn link_fixture_vue(root: &Path) {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap();
    let vue = workspace
        .join("tests/node_modules/vue-ssr-css-vars-oracle")
        .canonicalize()
        .unwrap();
    fs::create_dir_all(root.join("node_modules")).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(vue, root.join("node_modules/vue")).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(vue, root.join("node_modules/vue")).unwrap();
}

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

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
        .current_dir(project)
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
