#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]

#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

use serde_json::{Value, json};
use std::{
    path::Path,
    process::{Command, Output},
};

const INPUT: &str = include_str!(
    "../../../tests/_fixtures/differential/typechecker/outside-import-types/input.json"
);

fn write(root: &Path, file: &str, source: &str) {
    let path = root.join(file);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, source).unwrap();
}

fn fixture(root: &Path) -> Value {
    let input: Value = serde_json::from_str(INPUT).unwrap();
    for (file, source) in input["files"].as_object().unwrap() {
        write(root, file, source.as_str().unwrap());
    }
    input
}

fn cli_diagnostics(app: &Path, corsa: &Path, extra: &[&str]) -> Vec<(String, Value)> {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(app)
        .env("CORSA_PATH", corsa)
        .args(["check", "--no-config", "--format", "json"])
        .args(extra)
        .output()
        .unwrap();
    capture(app, corsa, "cli", extra, &output);
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        !output.status.success(),
        "the authored errors must fail: {stdout}\n{stderr}"
    );
    let result: Value = serde_json::from_str(&stdout).expect(&stdout);
    let mut diagnostics = result["files"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|file| !file["diagnostics"].as_array().unwrap().is_empty())
        .map(|file| {
            (
                file["file"].as_str().unwrap().replace('\\', "/"),
                file["diagnostics"].clone(),
            )
        })
        .collect::<Vec<_>>();
    diagnostics.sort_by(|left, right| left.0.cmp(&right.0));
    diagnostics
}

fn oracle(app: &Path, corsa: &Path, expected: &str) {
    let version = Command::new(corsa).arg("--version").output().unwrap();
    assert_eq!(
        String::from_utf8(version.stdout).unwrap().trim(),
        "Version 7.0.2"
    );
    let output = Command::new(corsa)
        .current_dir(app)
        .args(["-p", "tsconfig.json", "--pretty", "false"])
        .output()
        .unwrap();
    capture(app, corsa, "oracle", &[], &output);
    assert!(!output.status.success());
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
    assert_eq!(
        String::from_utf8(output.stdout)
            .unwrap()
            .replace("\r\n", "\n"),
        expected
    );
}

fn capture(app: &Path, corsa: &Path, phase: &str, extra: &[&str], output: &Output) {
    let Some(root) = std::env::var_os("VIZE_TYPECHECK_REGRESSION_CAPTURE") else {
        return;
    };
    let thread = std::thread::current();
    let case = thread.name().unwrap_or("unknown");
    let root = std::path::PathBuf::from(root).join(case);
    std::fs::create_dir_all(&root).unwrap();
    let phase = if extra.is_empty() {
        phase.to_owned()
    } else {
        format!("{phase}-sharded")
    };
    std::fs::write(root.join(format!("{phase}.stdout.txt")), &output.stdout).unwrap();
    std::fs::write(root.join(format!("{phase}.stderr.txt")), &output.stderr).unwrap();
    std::fs::write(
        root.join(format!("{phase}.json")),
        json!({
            "exitCode": output.status.code(), "cwd": app,
            "cliBinary": env!("CARGO_BIN_EXE_vize"),
            "nativeBinary": corsa,
            "arguments": extra,
        })
        .to_string(),
    )
    .unwrap();
    if phase == "oracle" {
        copy_inputs(app.parent().unwrap(), &root.join("inputs"));
    }
}

fn copy_inputs(source: &Path, target: &Path) {
    std::fs::create_dir_all(target).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        if entry.file_name() == ".vize" {
            continue;
        }
        let output = target.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_inputs(&entry.path(), &output);
        } else {
            std::fs::copy(entry.path(), output).unwrap();
        }
    }
}

#[test]
fn outside_import_preserves_original_type_libraries_and_real_cli_diagnostics() {
    let Some(corsa) = corsa_requirement::required_or_skip::<std::path::PathBuf>(None) else {
        return;
    };
    let case = tempfile::tempdir().unwrap();
    let input = fixture(case.path());
    let app = case.path().join("app");
    oracle(
        &app,
        &corsa,
        input["expected"]["nativeStdout"].as_str().unwrap(),
    );
    let expected = vec![("src/b.ts".into(), input["expected"]["diagnostics"].clone())];
    assert_eq!(cli_diagnostics(&app, &corsa, &[]), expected);
    assert_eq!(cli_diagnostics(&app, &corsa, &["--servers", "2"]), expected);
}

#[test]
fn outside_import_keeps_direct_scoped_type_packages_at_the_config_anchor() {
    let Some(corsa) = corsa_requirement::required_or_skip::<std::path::PathBuf>(None) else {
        return;
    };
    let case = tempfile::tempdir().unwrap();
    let input = fixture(case.path());
    let mut config: Value =
        serde_json::from_str(input["files"]["app/tsconfig.json"].as_str().unwrap()).unwrap();
    config["compilerOptions"]["types"] = json!(["foo", "@fixture/client"]);
    write(case.path(), "app/tsconfig.json", &config.to_string());
    write(
        case.path(),
        "app/node_modules/@fixture/client/package.json",
        "{\"name\":\"@fixture/client\",\"types\":\"index.d.ts\"}\n",
    );
    write(
        case.path(),
        "app/node_modules/@fixture/client/index.d.ts",
        "declare const appGlobal: 1;\n",
    );
    write(
        case.path(),
        "app/src/globals.ts",
        "export const foo: 1 = fooGlobal;\nexport const app: 1 = appGlobal;\n",
    );
    let app = case.path().join("app");
    oracle(
        &app,
        &corsa,
        input["expected"]["nativeStdout"].as_str().unwrap(),
    );
    assert_eq!(
        cli_diagnostics(&app, &corsa, &[]),
        vec![("src/b.ts".into(), input["expected"]["diagnostics"].clone())]
    );
}

#[test]
fn type_config_anchor_does_not_expose_app_modules_to_sibling_sources() {
    let Some(corsa) = corsa_requirement::required_or_skip::<std::path::PathBuf>(None) else {
        return;
    };
    let case = tempfile::tempdir().unwrap();
    let input = fixture(case.path());
    write(
        case.path(),
        "app/node_modules/only-in-app/package.json",
        "{\"name\":\"only-in-app\",\"types\":\"index.d.ts\"}\n",
    );
    write(
        case.path(),
        "app/node_modules/only-in-app/index.d.ts",
        "export {};\n",
    );
    write(
        case.path(),
        "shared/data.ts",
        "import {} from \"only-in-app\";\nexport const SHARED = 1;\n",
    );
    let app = case.path().join("app");
    let expected = concat!(
        "src/b.ts(3,14): error TS2322: Type 'number' is not assignable to type 'string'.\n",
        "../shared/data.ts(1,16): error TS2307: Cannot find module 'only-in-app' or its corresponding type declarations.\n"
    );
    oracle(&app, &corsa, expected);
    assert_eq!(
        cli_diagnostics(&app, &corsa, &[]),
        vec![
            (
                "../shared/data.ts".into(),
                json!([
                    "error:1:16 [TS2307] Cannot find module 'only-in-app' or its corresponding type declarations."
                ])
            ),
            ("src/b.ts".into(), input["expected"]["diagnostics"].clone())
        ]
    );
}
