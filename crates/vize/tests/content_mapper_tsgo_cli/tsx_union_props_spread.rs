//! Under `exactOptionalPropertyTypes`, TSX may spread a union of props objects
//! into an SFC. TypeScript infers the JSX key witness from one union member, so
//! a witness that kept that member's required keys rejected the other members.
//! The witness only names authored keys; it now requires none of them. Missing,
//! conflicting and mistyped props are still reported, as vue-tsc reports them.

use std::path::PathBuf;

use super::{
    CompactString, TSGO_ENV, check_project, cstr, install_mapper_manifest, install_vue_package,
    options_array_native_root_13113::install_published_sdk, output_text, workspace_root,
};

macro_rules! reported {
    ($path:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/typechecker/tsx-union-props-spread/",
            $path
        ))
    };
}

const FILES: [(&str, &str); 4] = [
    ("tsconfig.json", reported!("tsconfig.json.txt")),
    ("src/UnionProps.vue", reported!("src/UnionProps.vue.txt")),
    (
        "src/OptionalProps.vue",
        reported!("src/OptionalProps.vue.txt"),
    ),
    ("src/use.tsx", reported!("src/use.tsx.txt")),
];

/// The two union spreads pass; the missing, conflicting and mistyped uses fail.
const EXPECTED: [&str; 3] = [
    "src/use.tsx(9,25) TS2322",
    "src/use.tsx(10,22) TS2322",
    "src/use.tsx(11,23) TS2322",
];

/// Each diagnostic's head line as `file(line,col) code`.
fn located(stdout: &str) -> Vec<CompactString> {
    stdout
        .lines()
        .filter_map(|line| line.split_once(": error "))
        .map(|(at, rest)| cstr!("{at} {}", rest.split(':').next().unwrap_or(rest)))
        .collect()
}

#[test]
fn standard_tsgo_accepts_union_props_spread_under_exact_optional_property_types() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping TSX union props spread: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} is not a file");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let receipt = tempfile::Builder::new()
        .prefix("content-mapper-tsx-union-props-spread-")
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
