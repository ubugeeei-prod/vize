//! #13239: a generic parent filling a slot of a generic child whose slot record
//! keys depend on the parent's type parameter (Nuxt UI's `UTable`
//! `DynamicCellSlots`) typed the slot props as `unknown`. The checker defers
//! the payload there, and the deferred payload's constraint collapsed. The
//! reporter's files pass; the variants cover a plain `Record<string, F>`, an
//! exact key union, Nuxt UI's whole `TableSlots` intersection and both dynamic
//! slot names. One control reads a member `TItem` does not have, which proves
//! the payload is typed, not `any`.

use std::path::PathBuf;

use super::{
    CompactString, TSGO_ENV, check_project, cstr, install_mapper_manifest, install_vue_package,
    options_array_native_root_13113::install_published_sdk, output_text, workspace_root,
};

macro_rules! reported {
    ($path:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/typechecker/generic-slot-props-13239/",
            $path
        ))
    };
}

const FILES: [(&str, &str); 9] = [
    ("tsconfig.json", reported!("tsconfig.json.txt")),
    ("src/components.d.ts", reported!("src/components.d.ts.txt")),
    ("src/my-table.vue", reported!("src/my-table.vue.txt")),
    ("src/plain-table.vue", reported!("src/plain-table.vue.txt")),
    ("src/exact-table.vue", reported!("src/exact-table.vue.txt")),
    ("src/ui-table.vue", reported!("src/ui-table.vue.txt")),
    (
        "src/generic-outer.vue",
        reported!("src/generic-outer.vue.txt"),
    ),
    ("src/plain-outer.vue", reported!("src/plain-outer.vue.txt")),
    (
        "src/generic-variants.vue",
        reported!("src/generic-variants.vue.txt"),
    ),
];

/// Only the control's `row.original.missing` is reported. Before the fix every
/// generic slot scope except the plain `Record<string, F>` one was TS2339 on
/// `row` of `unknown`.
const EXPECTED: [&str; 1] = ["src/generic-variants.vue(21,52) TS2339"];

/// Each diagnostic's head line as `file(line,col) code`.
fn located(stdout: &str) -> Vec<CompactString> {
    stdout
        .lines()
        .filter_map(|line| line.split_once(": error "))
        .map(|(at, rest)| cstr!("{at} {}", rest.split(':').next().unwrap_or(rest)))
        .collect()
}

#[test]
fn standard_tsgo_types_generic_child_slot_props_inside_a_generic_parent() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping generic slot props: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} is not a file");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let receipt = tempfile::Builder::new()
        .prefix("content-mapper-generic-slot-props-")
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
