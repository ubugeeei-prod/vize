//! #13179: under `exactOptionalPropertyTypes`, `withDefaults` accepts an
//! `undefined` default for a prop whose type includes `undefined`, as Vue's
//! `InferDefault` does since vuejs/core#13594. The helper's native default
//! types lacked `undefined`, so the reporter's default was TS2379. A prop that
//! does not include `undefined` still rejects it.

use std::path::PathBuf;

use super::{
    CompactString, TSGO_ENV, check_project, cstr, install_mapper_manifest, install_vue_package,
    options_array_native_root_13113::install_published_sdk, output_text, workspace_root,
};

macro_rules! reported {
    ($path:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/typechecker/with-defaults-undefined-13179/",
            $path
        ))
    };
}

const FILES: [(&str, &str); 3] = [
    ("tsconfig.json", reported!("tsconfig.json.txt")),
    ("src/Repro.vue", reported!("src/Repro.vue.txt")),
    ("src/Control.vue", reported!("src/Control.vue.txt")),
];

/// Vue 3.6's own `withDefaults` answers the same: `Repro.vue` passes, and
/// `Control.vue`, whose prop excludes `undefined`, is TS2379 at its defaults.
const EXPECTED: [&str; 1] = ["src/Control.vue(3,50) TS2379"];

/// Each diagnostic's head line as `file(line,col) code`.
fn located(stdout: &str) -> Vec<CompactString> {
    stdout
        .lines()
        .filter_map(|line| line.split_once(": error "))
        .map(|(at, rest)| cstr!("{at} {}", rest.split(':').next().unwrap_or(rest)))
        .collect()
}

#[test]
fn standard_tsgo_accepts_undefined_with_defaults_under_exact_optional_property_types() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping withDefaults undefined defaults: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} is not a file");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let receipt = tempfile::Builder::new()
        .prefix("content-mapper-with-defaults-undefined-")
        .tempdir_in(&cases_root)
        .unwrap();
    for published_sdk in [false, true] {
        let mode = if published_sdk { "sdk" } else { "embedded" };
        let project = receipt.path().join(mode);
        install_mapper_manifest(&project);
        install_vue_package(&project);
        if published_sdk {
            install_published_sdk(&project);
        }
        for (path, contents) in FILES {
            let path = project.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, contents).unwrap();
        }
        let output = check_project(&tsgo, &project, "tsconfig.json");
        assert_eq!(output.stderr, b"", "{mode}: {}", output_text(&output));
        let located = located(std::str::from_utf8(&output.stdout).unwrap());
        assert_eq!(
            located.iter().map(|at| at.as_str()).collect::<Vec<_>>(),
            EXPECTED,
            "{mode}: {}",
            output_text(&output)
        );
        assert_eq!(
            output.status.code(),
            Some(2),
            "{mode}: {}",
            output_text(&output)
        );
    }
}
