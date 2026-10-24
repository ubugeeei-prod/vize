#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]

//! Boolean casting beside an object-literal `withDefaults`. Expectations are the
//! `vue-tsc` diagnostics recorded in the fixture README: an absent `boolean`
//! prop is `boolean` in the template unless its default is `undefined`.

use crate::support::corsa_requirement;

use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

macro_rules! fixture {
    ($name:literal) => {
        include_str!(concat!(
            "../../../tests/_fixtures/differential/typechecker/with-defaults-boolean-casting/",
            $name
        ))
    };
}

const CONFIG: &str = fixture!("tsconfig.json.txt");
const EXPECT: &str = fixture!("expect.ts.txt");

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap()
}

fn write(root: &Path, file: &str, source: &str) {
    let path = root.join(file);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, source).unwrap();
}

/// Run `vize check` over one component and return `line:column code` for
/// every diagnostic it reports on that component.
fn check(component: &str) -> Option<Vec<String>> {
    let corsa = corsa_requirement::required_or_skip::<PathBuf>(None)?;
    let parent = workspace_root().join("npm/cli/target/vize-tests/with-defaults-boolean-casting");
    std::fs::create_dir_all(&parent).unwrap();
    let case = tempfile::tempdir_in(parent).unwrap();
    write(case.path(), "tsconfig.json", CONFIG);
    write(case.path(), "src/expect.ts", EXPECT);
    write(case.path(), "src/A.vue", component);

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(case.path())
        .env("CORSA_PATH", &corsa)
        .args(["check", "--no-config", "--format", "json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let json: Value =
        serde_json::from_str(&stdout).unwrap_or_else(|error| panic!("{error}\n{stdout}\n{stderr}"));

    let mut reported = Vec::new();
    for file in json["files"].as_array().unwrap() {
        let diagnostics = file["diagnostics"].as_array().unwrap();
        if diagnostics.is_empty() {
            continue;
        }
        let name = file["file"].as_str().unwrap().replace('\\', "/");
        assert_eq!(name, "src/A.vue", "{stdout}\n{stderr}");
        for diagnostic in diagnostics {
            reported.push(position_and_code(diagnostic.as_str().unwrap()));
        }
    }
    assert_eq!(json["errorCount"], reported.len(), "{stdout}\n{stderr}");
    assert_eq!(output.status.success(), reported.is_empty(), "{stdout}");
    Some(reported)
}

/// `error:10:14 [TS2345] Argument …` → `10:14 TS2345`.
fn position_and_code(diagnostic: &str) -> String {
    let rest = diagnostic.strip_prefix("error:").unwrap_or(diagnostic);
    let (position, rest) = rest.split_once(" [").unwrap();
    let (code, _) = rest.split_once(']').unwrap();
    format!("{position} {code}")
}

fn assert_reported(component: &str, expected: &[&str]) {
    let Some(reported) = check(component) else {
        return;
    };
    assert_eq!(reported, expected);
}

#[test]
fn an_absent_boolean_prop_is_cast_to_false_beside_literal_defaults() {
    assert_reported(fixture!("AbsentBoolean.vue.txt"), &[]);
}

#[test]
fn an_absent_non_boolean_prop_stays_optional() {
    assert_reported(fixture!("AbsentNumber.vue.txt"), &["8:28 TS2345"]);
}

#[test]
fn a_boolean_prop_defaulted_to_undefined_stays_optional() {
    assert_reported(
        fixture!("UndefinedBooleanDefault.vue.txt"),
        &["8:26 TS2345"],
    );
}
