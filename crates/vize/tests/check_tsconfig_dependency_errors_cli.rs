#![cfg(test)]
#![expect(clippy::disallowed_types, reason = "whole subprocess fixture packets")]
#![expect(clippy::disallowed_macros, reason = "whole subprocess fixture packets")]

use std::{fs, path::Path, process::Command};

const DENY_JS: &str = r#"{"extends":"./tsconfig.base.json","compilerOptions":{"allowJs":false}}"#;

fn run(root: &Path, args: &[&str], code: i32, expected_stderr: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .env("CORSA_PATH", root.join("must-not-start-corsa"))
        .env("NO_COLOR", "1")
        .env_remove("FORCE_COLOR")
        .env_remove("CLICOLOR_FORCE")
        .arg("check")
        .args(args)
        .arg("--no-config")
        .output()
        .unwrap();
    eprintln!("strict precollection tsconfig CLI packet: {output:?}");
    assert_eq!(
        (
            output.status.code(),
            output.stdout.as_slice(),
            output.stderr.as_slice()
        ),
        (Some(code), b"".as_slice(), expected_stderr.as_bytes()),
    );
}

fn error(config: &Path, malformed: bool) -> String {
    let reason = if malformed {
        format!(
            "Configuration error: cannot parse tsconfig {}: JSON parse error: key must be a string at line 1 column 2",
            config.with_file_name("tsconfig.base.json").display()
        )
    } else {
        format!(
            "Configuration error: cannot resolve tsconfig extends \"./tsconfig.base.json\" from {}",
            config.display()
        )
    };
    format!(
        "Error: Failed to resolve TypeScript configuration `{}`: {reason}\n",
        config.display()
    )
}

fn no_match(input: &str) -> String {
    format!(
        "No .vue, .ts, .tsx, .mts, .cts, .js, .jsx, .mjs, .cjs, .d.ts, .d.mts, or .d.cts files found matching inputs: [{input:?}]\n"
    )
}

/// A missing `extends` entry is reported and skipped like `tsc` (TS6053), so
/// it never refuses before collection; a malformed base still does.
#[test]
fn missing_extends_continues_but_malformed_bases_refuse_before_collection() {
    let case = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(case.path()).unwrap();
    let config = root.join("tsconfig.json");
    fs::write(
        root.join("package.json"),
        r#"{"private":true,"type":"module"}"#,
    )
    .unwrap();
    fs::write(root.join("input.js"), "export const value = 1;\n").unwrap();
    fs::write(&config, DENY_JS).unwrap();
    run(&root, &["input.js"], 0, &no_match("input.js"));
    fs::write(root.join("tsconfig.base.json"), "{invalid").unwrap();
    run(&root, &["input.js"], 2, &error(&config, true));
    fs::remove_file(root.join("tsconfig.base.json")).unwrap();
    run(&root, &["input.js"], 0, &no_match("input.js"));
    assert_eq!(
        fs::read_to_string(root.join("input.js")).unwrap(),
        "export const value = 1;\n"
    );
}

#[test]
fn nested_selected_owner_refuses_before_its_javascript_filter() {
    let case = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(case.path()).unwrap();
    let app = root.join("app");
    fs::create_dir(&app).unwrap();
    fs::write(
        root.join("package.json"),
        r#"{"private":true,"type":"module"}"#,
    )
    .unwrap();
    fs::write(
        root.join("tsconfig.json"),
        r#"{"files":[],"references":[{"path":"./app"}]}"#,
    )
    .unwrap();
    let config = app.join("tsconfig.json");
    fs::write(&config, DENY_JS).unwrap();
    fs::write(app.join("input.js"), "export const value = 1;\n").unwrap();
    run(&root, &["app/input.js"], 0, &no_match("app/input.js"));
    fs::write(app.join("tsconfig.base.json"), "{invalid").unwrap();
    run(&root, &["app/input.js"], 1, &error(&config, true));
    // Repair the same selected owner without changing its source or shell.
    fs::write(app.join("tsconfig.base.json"), "{}").unwrap();
    run(&root, &["app/input.js"], 0, &no_match("app/input.js"));
    assert_eq!(
        fs::read_to_string(app.join("input.js")).unwrap(),
        "export const value = 1;\n"
    );
    assert_eq!(fs::read_to_string(&config).unwrap(), DENY_JS);
}

#[test]
fn valid_denied_javascript_configuration_keeps_no_match_success_after_repair() {
    let case = tempfile::tempdir().unwrap();
    let root = fs::canonicalize(case.path()).unwrap();
    fs::write(
        root.join("package.json"),
        r#"{"private":true,"type":"module"}"#,
    )
    .unwrap();
    let config = root.join("tsconfig.json");
    fs::write(&config, DENY_JS).unwrap();
    fs::write(root.join("input.js"), "export const value = 1;\n").unwrap();
    run(&root, &["input.js"], 0, &no_match("input.js"));
    fs::write(root.join("tsconfig.base.json"), "{}").unwrap();
    run(&root, &["input.js"], 0, &no_match("input.js"));
    run(
        &root,
        &["input.js", "--tsconfig", "tsconfig.json"],
        0,
        &no_match("input.js"),
    );
    assert_eq!(
        fs::read_to_string(root.join("input.js")).unwrap(),
        "export const value = 1;\n"
    );
    assert_eq!(fs::read_to_string(&config).unwrap(), DENY_JS);
}
