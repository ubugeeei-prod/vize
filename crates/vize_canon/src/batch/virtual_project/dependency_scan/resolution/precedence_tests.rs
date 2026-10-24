//! Authored path-selection boundaries shared by batch and editor projections.
#![expect(clippy::disallowed_macros, reason = "fixtures use std formatting")]

use super::{resolve_dependency, resolve_dependency_with_inputs};

#[test]
fn exact_alias_wins_over_longer_or_equal_scored_wildcards_in_either_order() {
    let root = tempfile::tempdir().unwrap();
    let exact = root.path().join("exact.ts");
    std::fs::write(&exact, "export const value = 'exact';").unwrap();
    for wildcard in ["@x*", "@*"] {
        let capture = if wildcard == "@*" { "x" } else { "" };
        std::fs::write(
            root.path().join(format!("wildcard{capture}.ts")),
            "export const value = 123;",
        )
        .unwrap();
        let mut aliases = vec![
            (wildcard.to_owned(), "wildcard*.ts".to_owned()),
            ("@x".to_owned(), "exact.ts".to_owned()),
        ];
        for _ in 0..2 {
            let (selected, inputs) =
                resolve_dependency_with_inputs("@x", root.path(), root.path(), &aliases);
            assert_eq!(selected, Some(exact.clone()));
            assert_eq!(
                inputs,
                vec![exact.clone()],
                "shadowed routes must not become dependencies"
            );
            aliases.reverse();
        }
    }
}

#[test]
fn an_exact_alias_with_missing_targets_does_not_fall_through_to_a_wildcard() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("wildcard.ts"), "export const value = 123;").unwrap();
    let aliases = vec![
        ("@x*".to_owned(), "wildcard*.ts".to_owned()),
        ("@x".to_owned(), "missing.ts".to_owned()),
    ];
    let (selected, inputs) =
        resolve_dependency_with_inputs("@x", root.path(), root.path(), &aliases);
    assert_eq!(selected, None);
    assert_eq!(inputs, vec![root.path().join("missing.ts")]);
}

#[test]
fn the_longest_matching_wildcard_is_selected_before_any_file_probes() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("genericx.ts"), "export const value = 123;").unwrap();
    let aliases = vec![
        ("@*".to_owned(), "generic*.ts".to_owned()),
        ("@x*".to_owned(), "missing*.ts".to_owned()),
    ];
    let (selected, inputs) =
        resolve_dependency_with_inputs("@x", root.path(), root.path(), &aliases);
    assert_eq!(selected, None);
    assert_eq!(inputs, vec![root.path().join("missing.ts")]);
}

#[test]
fn target_fallback_stays_inside_the_selected_exact_or_wildcard_pattern() {
    let root = tempfile::tempdir().unwrap();
    let expected = root.path().join("selected.ts");
    std::fs::write(&expected, "export const value = 'selected';").unwrap();
    std::fs::write(root.path().join("decoy.ts"), "export const value = 123;").unwrap();
    for pattern in ["@x", "@x*"] {
        let targets = if pattern.contains('*') {
            ["missing*.ts", "selected*.ts"]
        } else {
            ["missing.ts", "selected.ts"]
        };
        let aliases = vec![
            ("@*".to_owned(), "decoy*.ts".to_owned()),
            (pattern.to_owned(), targets[0].to_owned()),
            (pattern.to_owned(), targets[1].to_owned()),
        ];
        let (selected, inputs) =
            resolve_dependency_with_inputs("@x", root.path(), root.path(), &aliases);
        assert_eq!(selected, Some(expected.clone()));
        assert_eq!(
            inputs,
            vec![root.path().join("missing.ts"), expected.clone()]
        );
    }
}

#[test]
fn equal_wildcard_prefixes_retain_the_incoming_pattern_order() {
    let root = tempfile::tempdir().unwrap();
    let first = root.path().join("firsty.ts");
    let second = root.path().join("second.ts");
    for file in [&first, &second] {
        std::fs::write(file, "export {};").unwrap();
    }
    let mut aliases = vec![
        ("@x*".to_owned(), "first*.ts".to_owned()),
        ("@x*y".to_owned(), "second*.ts".to_owned()),
    ];
    assert_eq!(
        resolve_dependency("@xy", root.path(), root.path(), &aliases),
        Some(first)
    );
    aliases.reverse();
    assert_eq!(
        resolve_dependency("@xy", root.path(), root.path(), &aliases),
        Some(second)
    );
}
