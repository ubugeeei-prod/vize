//! #13111 through `vize check`: the content-mapper fixture's registered
//! components are known and typed by their registered values.
#![cfg(test)]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
use crate::check_canon_recent_issues_cli::{
    link_workspace_vue, resolve_test_corsa_path, unique_case_dir,
};
use crate::support::corsa_requirement;

use std::{path::Path, process::Command};

use vize_l0::cstr;

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/content_mapper_registered_components_13111"
);

const PRECISION: [&str; 4] = [
    "src/MixedWrong.vue error:14:13 [TS2322] Type 'number' is not assignable to type 'string'. (source: <Aliased :label=\"count\" />; binding: 'label')",
    "src/Wrong.vue error:13:13 [TS2322] Type 'number' is not assignable to type 'string'. (source: <Aliased :label=\"1\" />; binding: 'label')",
    "src/Wrong.vue error:14:17 [TS2322] Type 'number' is not assignable to type 'string'. (source: <AliasedMenu :as=\"2\" />; binding: 'as')",
    "src/Wrong.vue error:15:18 [TS2322] Type 'number' is not assignable to type 'string'. (source: <MemberButton :size=\"3\" />; binding: 'size')",
];
const UNKNOWN: [&str; 2] = [
    "src/MixedWrong.vue error:15:4 [TS2339] Property 'NotRegistered' does not exist on type '{}'. (source: <NotRegistered />)",
    "src/Wrong.vue error:16:4 [TS2339] Property 'NotRegistered' does not exist on type '{}'. (source: <NotRegistered />)",
];

fn copy_dir(source: &Path, target: &Path) {
    std::fs::create_dir_all(target).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let destination = target.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &destination);
        } else {
            std::fs::copy(entry.path(), destination).unwrap();
        }
    }
}

fn check_rows(project_root: &Path, corsa_path: &Path, tsconfig: &str) -> Vec<String> {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root)
        .env("CORSA_PATH", corsa_path)
        .args(["check", "--tsconfig", tsconfig, "--format", "json"])
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(stdout).unwrap();
    let mut rows: Vec<String> = json["files"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|file| {
            let name = file["file"].as_str().unwrap().replace('\\', "/");
            file["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(move |row| cstr!("{name} {row}").to_string())
        })
        .collect();
    rows.sort_unstable();
    rows
}

#[test]
fn check_types_registered_component_keys_by_their_values() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = unique_case_dir("registered-components-13111");
    let _ = std::fs::remove_dir_all(&project_root);
    copy_dir(Path::new(FIXTURE), &project_root);
    link_workspace_vue(&project_root).unwrap();

    let mut expected: Vec<_> = PRECISION.iter().chain(&UNKNOWN).copied().collect();
    expected.sort_unstable();
    assert_eq!(
        check_rows(&project_root, &corsa_path, "tsconfig.json"),
        expected
    );
    assert_eq!(
        check_rows(&project_root, &corsa_path, "tsconfig.disabled.json"),
        PRECISION
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
