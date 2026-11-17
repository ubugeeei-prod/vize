//! A component with a listener also gets an inference-only copy of its
//! props, which gives callbacks no contextual type and ignores only the
//! first line of each entry. Callbacks inside a multi-line array or object
//! value (vuestic-admin `VaInput :rules="[\n (v) => ...]"`) started below
//! that line, so every parameter reported TS7006 under the content mapper
//! and `vize check` although the canonical props call types it. The real
//! errors inside those callbacks still report, once.

use std::{fs, path::Path, path::PathBuf, process::Command};

use super::{
    CompactString, TSGO_ENV, check_project, cstr, install_mapper_manifest, install_vue_package,
    output_text, workspace_root,
};

macro_rules! fixture {
    ($path:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/typechecker/multiline-library-prop-callbacks/",
            $path
        ))
    };
}

const FILES: [(&str, &str); 5] = [
    ("tsconfig.json", fixture!("tsconfig.json.txt")),
    (
        "node_modules/rules-lib/package.json",
        fixture!("rules-lib/package.json.txt"),
    ),
    (
        "node_modules/rules-lib/index.d.ts",
        fixture!("rules-lib/index.d.ts.txt"),
    ),
    ("src/Valid.vue", fixture!("src/Valid.vue.txt")),
    ("src/Invalid.vue", fixture!("src/Invalid.vue.txt")),
];

/// The genuine errors, and nothing in `Valid.vue`.
const REPORTS: [&str; 2] = [
    "src/Invalid.vue(12,16) TS2339",
    "src/Invalid.vue(13,7) TS2322",
];

fn write(root: &Path, path: &str, contents: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

/// `file(line,col) TScode` for every diagnostic tsgo reports with the mapper.
fn mapper_lines(tsgo: &Path, root: &Path) -> Vec<CompactString> {
    let output = check_project(tsgo, root, "tsconfig.json");
    assert_eq!(output.stderr, b"", "{}", output_text(&output));
    std::str::from_utf8(&output.stdout)
        .unwrap()
        .lines()
        .filter_map(|line| {
            let (position, rest) = line.split_once(": error ")?;
            let (code, _) = rest.split_once(':')?;
            Some(cstr!("{position} {code}"))
        })
        .collect()
}

/// The same `file(line,col) TScode` rows from `vize check`, sorted.
fn cli_lines(tsgo: &Path, root: &Path) -> Vec<CompactString> {
    let cli = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .env("CORSA_PATH", tsgo)
        .args([
            "check",
            "--quiet",
            "--format",
            "json",
            "--tsconfig",
            "tsconfig.json",
            "src",
        ])
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&cli.stdout)
        .unwrap_or_else(|error| panic!("{error}\n{}", output_text(&cli)));
    let mut observed = report["files"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|file| {
            let name = file["file"].as_str().unwrap().replace('\\', "/");
            file["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(move |diagnostic| {
                    let text = diagnostic.as_str()?.strip_prefix("error:")?;
                    let (position, rest) = text.split_once(" [")?;
                    let (code, _) = rest.split_once(']')?;
                    let (line, column) = position.split_once(':')?;
                    Some(cstr!("{name}({line},{column}) {code}"))
                })
        })
        .collect::<Vec<_>>();
    observed.sort();
    observed
}

#[test]
fn multi_line_callbacks_in_library_props_keep_their_context() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} is not a file");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("multiline-library-prop-callbacks-")
        .tempdir_in(cases_root)
        .unwrap();
    let root = project.path();
    install_mapper_manifest(root);
    install_vue_package(root);
    for (path, contents) in FILES {
        write(root, path, contents);
    }
    let expected = REPORTS.map(CompactString::from).to_vec();
    assert_eq!(mapper_lines(&tsgo, root), expected);
    assert_eq!(cli_lines(&tsgo, root), expected);
}
