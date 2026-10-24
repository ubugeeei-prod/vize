#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]

//! `withDefaults(defineProps<T>(), <expression>)` where the defaults are not an
//! object literal. Expectations are the `vue-tsc` diagnostics recorded in the
//! fixture README: a prop is non-`undefined` in the template exactly when the
//! *type* of the defaults argument carries a defined value for it.

use crate::support::corsa_requirement;

use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

macro_rules! fixture {
    ($name:literal) => {
        include_str!(concat!(
            "../../../tests/_fixtures/differential/typechecker/opaque-with-defaults/",
            $name
        ))
    };
}

const CONFIG: &str = fixture!("tsconfig.json.txt");
const DEFAULTS: &str = fixture!("defaults.ts.txt");

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
    let parent = workspace_root().join("npm/cli/target/vize-tests/opaque-with-defaults");
    std::fs::create_dir_all(&parent).unwrap();
    let case = tempfile::tempdir_in(parent).unwrap();
    write(case.path(), "tsconfig.json", CONFIG);
    write(case.path(), "src/defaults.ts", DEFAULTS);
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
fn defaults_returned_by_a_call_define_every_prop_they_carry() {
    assert_reported(fixture!("CallDefaults.vue.txt"), &[]);
}

#[test]
fn props_missing_from_the_returned_defaults_stay_optional() {
    assert_reported(fixture!("PartialCall.vue.txt"), &["11:14 TS2345"]);
}

#[test]
fn spread_call_defaults_combine_with_literal_keys() {
    assert_reported(fixture!("SpreadCall.vue.txt"), &["10:14 TS2345"]);
}

#[test]
fn defaults_typed_undefined_keep_the_prop_optional() {
    assert_reported(fixture!("UndefinedCall.vue.txt"), &["10:14 TS2345"]);
}

#[test]
fn conditional_defaults_define_only_the_props_both_branches_carry() {
    assert_reported(fixture!("ConditionalDefaults.vue.txt"), &["10:14 TS2345"]);
}

#[test]
fn boolean_props_stay_defined_beside_opaque_defaults() {
    assert_reported(fixture!("BooleanCall.vue.txt"), &[]);
}
