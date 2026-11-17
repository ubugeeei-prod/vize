//! #13021: a generic library component declared before vue-tsc wrote the
//! `__VLS_PROPS_FALLBACK` term must let undeclared attributes fall through in
//! loose mode on a TypeScript host that never negotiated diagnostic-code
//! selectors. The generated checker itself has to agree, not a directive.

use std::{fs, path::PathBuf};

use super::{
    CompactString, TSGO_ENV, check_project, cstr, install_mapper_manifest, install_vue_package,
    output_text, workspace_root,
};

macro_rules! reported {
    ($path:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/typechecker/library-generic-props-fallback-13021/reported/",
            $path
        ))
    };
}

const SOURCES: [(&str, &str); 4] = [
    ("libs.ts", reported!("src/libs.ts.txt")),
    ("App.vue", reported!("src/App.vue.txt")),
    ("Wrong.vue", reported!("src/Wrong.vue.txt")),
    (
        "MissingRequired.vue",
        reported!("src/MissingRequired.vue.txt"),
    ),
];

/// `file(line,column): error TSxxxx` heads the host printed, in order.
fn mapped_heads(prefix: &str, config: &str) -> Option<Vec<CompactString>> {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return None;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} is not a file");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in(cases_root)
        .unwrap();
    install_mapper_manifest(project.path());
    install_vue_package(project.path());
    fs::create_dir(project.path().join("src")).unwrap();
    fs::write(project.path().join("tsconfig.json"), config).unwrap();
    for (name, source) in SOURCES {
        fs::write(project.path().join("src").join(name), source).unwrap();
    }
    let output = check_project(&tsgo, project.path(), "tsconfig.json");
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let transformed = !stdout.contains("TS18069");
    assert!(
        transformed,
        "the mapper must transform every file: {}",
        output_text(&output)
    );
    Some(
        stdout
            .lines()
            .filter_map(|line| line.split_once(": error TS"))
            .map(|(head, rest)| cstr!("{head} TS{}", rest.get(..4).unwrap_or(rest)))
            .collect(),
    )
}

#[test]
fn standard_tsgo_lets_undeclared_attributes_fall_through_old_library_generics() {
    let Some(heads) = mapped_heads(
        "library-generic-loose-13021-",
        reported!("tsconfig.json.txt"),
    ) else {
        return;
    };
    // `buffer` and `data-index` fall through on every declaration shape. The
    // declared value error stays at its attribute, and the missing required
    // `items` is reported identically for both library shapes.
    assert_eq!(
        heads,
        [
            "src/MissingRequired.vue(6,4) TS2345",
            "src/MissingRequired.vue(7,4) TS2345",
            "src/Wrong.vue(6,16) TS2322",
        ],
    );
}

#[test]
fn standard_tsgo_keeps_strict_excess_properties_on_old_library_generics() {
    let Some(heads) = mapped_heads(
        "library-generic-strict-13021-",
        reported!("tsconfig.strict.json.txt"),
    ) else {
        return;
    };
    // Lines 8 and 12 are the `NoCtx` control, whose strict tail is the
    // environment's native attribute set; only the library rows are pinned.
    let library_rows = heads
        .iter()
        .map(CompactString::as_str)
        .filter(|head| {
            head.starts_with("src/App.vue(")
                && !head.starts_with("src/App.vue(8,")
                && !head.starts_with("src/App.vue(12,")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        library_rows,
        [
            "src/App.vue(6,32) TS2353",
            "src/App.vue(7,34) TS2353",
            "src/App.vue(10,32) TS2353",
            "src/App.vue(11,34) TS2353",
        ],
        "{heads:#?}",
    );
}
