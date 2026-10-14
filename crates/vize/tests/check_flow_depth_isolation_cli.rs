#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]

//! A long `v-if`/`v-else-if` chain must not exhaust TypeScript's control-flow
//! analysis. When a generated template body does, the checker reports TS2563
//! on it and stops flow typing for every file it checks afterwards, so an
//! unrelated `.ts` file loses contextual typing. Expectations are the
//! `vue-tsc` result recorded in the fixture README: no diagnostics.

#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

macro_rules! fixture {
    ($name:literal) => {
        include_str!(concat!(
            "../../../tests/_fixtures/differential/typechecker/flow-depth-isolation/",
            $name
        ))
    };
}

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

#[test]
fn a_long_else_if_chain_keeps_flow_typing_for_later_files() {
    let Some(corsa) = corsa_requirement::required_or_skip::<PathBuf>(None) else {
        return;
    };
    let parent = workspace_root().join("npm/cli/target/vize-tests/flow-depth-isolation");
    std::fs::create_dir_all(&parent).unwrap();
    let case = tempfile::tempdir_in(parent).unwrap();
    write(case.path(), "tsconfig.json", fixture!("tsconfig.json.txt"));
    write(case.path(), "src/Chain.vue", fixture!("Chain.vue.txt"));
    write(case.path(), "src/Child.vue", fixture!("Child.vue.txt"));
    write(case.path(), "src/later.ts", fixture!("later.ts.txt"));

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

    let reported: Vec<String> = json["files"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|file| {
            let name = file["file"].as_str().unwrap().replace('\\', "/");
            file["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .map(move |diagnostic| format!("{name} {}", diagnostic.as_str().unwrap()))
        })
        .collect();
    assert_eq!(reported, Vec::<String>::new(), "{stderr}");
    assert_eq!(json["errorCount"], 0, "{stdout}");
    assert!(output.status.success(), "{stdout}\n{stderr}");
}
