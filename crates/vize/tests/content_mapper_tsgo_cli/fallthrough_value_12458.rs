//! #12458: a fallthrough attribute value error reaches the Content Mapper host
//! with the code, message and span `vize check` reports.

use std::{fs, path::PathBuf, process::Command};

use super::shared_helpers_performance::install_shared_helper_package;
use super::{
    TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};
use vize_l0::{String as CompactString, cstr};

macro_rules! fixture {
    ($path:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/typechecker/fallthrough-value-12458/",
            $path
        ))
    };
}

/// One owner each: the original fallthrough value, a declared prop, a
/// literal fallthrough value and an ARIA attr.
const EXPECTED: [(&str, u32, u32, &str); 4] = [
    (
        "App.vue",
        5,
        21,
        "Type '{}' is not assignable to type 'Numberish | undefined'.",
    ),
    (
        "App.vue",
        6,
        11,
        "Type 'number' is not assignable to type 'string'.",
    ),
    (
        "App.vue",
        6,
        22,
        "Type 'true' is not assignable to type 'Numberish | undefined'.",
    ),
    (
        "App.vue",
        6,
        39,
        "Type '1' is not assignable to type 'Booleanish | undefined'.",
    ),
];

#[test]
fn standard_tsgo_reports_fallthrough_values_like_vize_check() {
    check_pair(
        "fallthrough-value-12458-",
        fixture!("tsconfig.json.txt"),
        &[
            ("Field.vue", fixture!("Field.vue.txt")),
            ("App.vue", fixture!("App.vue.txt")),
        ],
        &EXPECTED,
    );
}

/// A fallthrough root's `Partial` literal and the per-prop assignment render
/// the same wrong-value message; the host must still see it once.
#[test]
fn relaxed_fallthrough_roots_report_wrong_values_once() {
    check_pair(
        "fallthrough-relaxed-root-12458-",
        fixture!("relaxed/tsconfig.json.txt"),
        &[
            ("Inner.vue", fixture!("relaxed/Inner.vue.txt")),
            ("Typed.vue", fixture!("relaxed/Typed.vue.txt")),
        ],
        &[(
            "Typed.vue",
            5,
            11,
            "Type 'number' is not assignable to type 'string'.",
        )],
    );
}

#[test]
#[ignore = "#12491: only the ignored per-prop assignment rejects an explicit undefined at a relaxed root"]
fn relaxed_fallthrough_roots_report_undefined_required_props() {
    check_pair(
        "fallthrough-relaxed-undefined-12458-",
        fixture!("relaxed/tsconfig.json.txt"),
        &[
            ("Inner.vue", fixture!("relaxed/Inner.vue.txt")),
            ("Wrapper.vue", fixture!("relaxed/Wrapper.vue.txt")),
        ],
        &[(
            "Wrapper.vue",
            5,
            11,
            "Type 'undefined' is not assignable to type 'string'.",
        )],
    );
}

/// A generic child infers its props from the literal, and its per-prop
/// contracts are `unknown` or `any`: the literal stays the only owner.
#[test]
fn generic_children_keep_the_literal_as_owner() {
    check_pair(
        "fallthrough-generic-12458-",
        fixture!("generic/tsconfig.json.txt"),
        &[
            ("Generic.vue", fixture!("generic/Generic.vue.txt")),
            ("GenericLib.ts", fixture!("generic/GenericLib.ts.txt")),
            ("App.vue", fixture!("generic/App.vue.txt")),
        ],
        &[
            (
                "App.vue",
                6,
                29,
                "Type 'string' is not assignable to type 'number'.",
            ),
            (
                "App.vue",
                7,
                32,
                "Type 'string' is not assignable to type 'number'.",
            ),
        ],
    );
}

/// The exact TS2322 vector through tsc and the Vize mapper, with and without
/// the shared helper package, and through `vize check` whenever a Corsa
/// executable resolves.
fn check_pair(
    prefix: &str,
    config: &str,
    files: &[(&str, &str)],
    expected: &[(&str, u32, u32, &str)],
) {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} is not a file");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    fs::create_dir_all(&cases_root).unwrap();
    let mut wanted = expected
        .iter()
        .map(|(file, line, column, message)| {
            cstr!("src/{file}({line},{column}): error TS2322: {message}")
        })
        .collect::<Vec<_>>();
    wanted.sort_unstable();
    let mut project = None;
    for shared_helpers in [false, true] {
        let root = tempfile::Builder::new()
            .prefix(prefix)
            .tempdir_in(&cases_root)
            .unwrap();
        install_mapper_manifest(root.path());
        install_vue_package(root.path());
        if shared_helpers {
            install_shared_helper_package(root.path());
        }
        fs::create_dir(root.path().join("src")).unwrap();
        fs::write(root.path().join("tsconfig.json"), config).unwrap();
        for (name, source) in files {
            fs::write(root.path().join("src").join(name), source).unwrap();
        }
        let output = check_project(&tsgo, root.path(), "tsconfig.json");
        let stdout = std::str::from_utf8(&output.stdout).unwrap();
        let mut mapped = stdout
            .lines()
            .filter(|line| line.contains(": error TS"))
            .collect::<Vec<_>>();
        mapped.sort_unstable();
        assert_eq!(
            mapped,
            wanted,
            "shared helpers: {shared_helpers}\n{}",
            output_text(&output)
        );
        project = Some(root);
    }
    let project = project.unwrap();

    let Some(corsa) = crate::support::corsa_path::resolve(workspace_root()) else {
        eprintln!("skipping the vize check side: no Corsa executable resolved");
        return;
    };
    let batch = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .env("CORSA_PATH", corsa)
        .args([
            "check",
            "--tsconfig",
            "tsconfig.json",
            "src",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&batch.stdout)
        .unwrap_or_else(|error| panic!("{error}\n{}", output_text(&batch)));
    let mut checked = report["files"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|file| {
            let path = file["file"].as_str().unwrap();
            file["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .map(move |diagnostic| {
                    let text = diagnostic.as_str().unwrap();
                    let head = text.split_once(" (source: ").map_or(text, |(head, _)| head);
                    cstr!("{path} {head}")
                })
        })
        .collect::<Vec<CompactString>>();
    checked.sort_unstable();
    let mut wanted = expected
        .iter()
        .map(|(file, line, column, message)| {
            cstr!("src/{file} error:{line}:{column} [TS2322] {message}")
        })
        .collect::<Vec<_>>();
    wanted.sort_unstable();
    assert_eq!(checked, wanted, "{}", output_text(&batch));
}
