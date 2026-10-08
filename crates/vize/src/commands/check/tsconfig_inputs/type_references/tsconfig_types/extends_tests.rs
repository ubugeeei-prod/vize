#![expect(
    clippy::disallowed_macros,
    reason = "whole JSON corpus uses std strings"
)]
#![expect(
    clippy::disallowed_methods,
    reason = "whole JSON corpus uses std strings"
)]

use super::*;

const INPUT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/tsconfig-types-extends-3984/input.json"
));

fn write(root: &Path, path: &str, source: &str) {
    let target = root.join(path);
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    std::fs::write(target, source).unwrap();
}

#[test]
fn types_arrays_follow_complete_official_extends_cases_including_empty_presence() {
    let corpus: Value = serde_json::from_str(INPUT).unwrap();
    for (name, case) in corpus["cases"].as_object().unwrap() {
        let root = tempfile::tempdir().unwrap();
        for group in [&corpus["commonFiles"], &case["files"]] {
            for (path, source) in group.as_object().unwrap() {
                write(root.path(), path, source.as_str().unwrap());
            }
        }
        let config = root.path().join("tsconfig.json");
        let actual = load_tsconfig_type_packages(&config, &mut ConfigChain::default()).unwrap();
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            case["effectiveTypes"],
            "{name}"
        );
        assert_eq!(
            serde_json::to_value(collect_tsconfig_type_packages(Some(&config))).unwrap(),
            case["effectiveTypes"],
            "public type-package collection for {name}"
        );
    }
}

#[test]
fn failed_parents_keep_valid_siblings_and_local_overrides_and_recover_next_operation() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "first.json",
        r#"{"compilerOptions":{"types":["first"]}}"#,
    );
    write(root.path(), "bad.json", "{");
    write(root.path(), "absent.json", "{}");
    write(
        root.path(),
        "root.json",
        r#"{"extends":["./first.json","./bad.json","./missing.json","./absent.json"]}"#,
    );
    let config = root.path().join("root.json");
    assert_eq!(collect_tsconfig_type_packages(Some(&config)), ["first"]);
    write(
        root.path(),
        "root.json",
        r#"{"extends":["./first.json","./bad.json"],"compilerOptions":{"types":[]}}"#,
    );
    assert!(collect_tsconfig_type_packages(Some(&config)).is_empty());
    write(
        root.path(),
        "root.json",
        r#"{"extends":["./first.json","./bad.json"]}"#,
    );
    write(
        root.path(),
        "bad.json",
        r#"{"compilerOptions":{"types":["repaired"]}}"#,
    );
    assert_eq!(collect_tsconfig_type_packages(Some(&config)), ["repaired"]);
    assert!(collect_tsconfig_type_packages(None).is_empty());
}

#[test]
fn failed_reads_and_parse_errors_release_ancestry_and_do_not_memoize_absence() {
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join("later.json");
    let mut chain = ConfigChain::default();
    assert!(load_tsconfig_type_packages(&config, &mut chain).is_err());
    write(root.path(), "later.json", "{");
    assert!(load_tsconfig_type_packages(&config, &mut chain).is_err());
    write(
        root.path(),
        "later.json",
        r#"{"compilerOptions":{"types":[]}}"#,
    );
    assert_eq!(
        load_tsconfig_type_packages(&config, &mut chain).unwrap(),
        Some(Vec::new())
    );
    // Successful absence is an effective value within this operation. A new
    // operation must observe subsequent author edits instead of a global memo.
    let absent = root.path().join("absent.json");
    write(root.path(), "absent.json", "{}");
    assert_eq!(
        load_tsconfig_type_packages(&absent, &mut chain).unwrap(),
        None
    );
    write(
        root.path(),
        "absent.json",
        r#"{"compilerOptions":{"types":["next"]}}"#,
    );
    assert_eq!(
        load_tsconfig_type_packages(&absent, &mut chain).unwrap(),
        None
    );
    assert_eq!(collect_tsconfig_type_packages(Some(&absent)), ["next"]);
}

#[test]
fn cycle_cut_types_remain_uncached_for_a_different_entry_ancestry() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "a.json",
        r#"{"extends":"./b.json","compilerOptions":{"types":["first"]}}"#,
    );
    write(root.path(), "b.json", r#"{"extends":"./a.json"}"#);
    let mut chain = ConfigChain::default();
    assert_eq!(
        load_tsconfig_type_packages(&root.path().join("a.json"), &mut chain).unwrap(),
        Some(vec!["first".to_owned()])
    );
    write(
        root.path(),
        "a.json",
        r#"{"extends":"./b.json","compilerOptions":{"types":[]}}"#,
    );
    assert_eq!(
        load_tsconfig_type_packages(&root.path().join("b.json"), &mut chain).unwrap(),
        Some(Vec::new())
    );
    // Stock TypeScript rejects these cycles; this law retains the existing
    // legacy termination policy and supplies no native cycle-parity credit.
}
