//! Path-pattern laws without changing candidate probing or alias precedence.
#![expect(clippy::disallowed_macros, reason = "fixtures use std formatting")]

use super::{resolve_dependency, resolve_dependency_with_inputs};
use std::path::Path;

#[test]
fn suffix_wildcards_resolve_only_the_captured_authored_child() {
    let root = tempfile::tempdir().unwrap();
    let child = root.path().join("src/components/Child.vue");
    std::fs::create_dir_all(child.parent().unwrap()).unwrap();
    std::fs::write(&child, "<template><div /></template>").unwrap();
    let aliases = vec![("@/*.vue".to_owned(), "src/*.vue".to_owned())];
    let (resolved, inputs) = resolve_dependency_with_inputs(
        "@/components/Child.vue",
        root.path(),
        root.path(),
        &aliases,
    );
    assert_eq!(resolved, Some(child.clone()));
    assert_eq!(inputs, vec![child]);
    for specifier in ["@/components/Child.ts", "other/Child.vue", "@/.vu"] {
        assert_eq!(
            resolve_dependency(specifier, root.path(), root.path(), &aliases),
            None,
            "suffix and prefix must both match: {specifier}"
        );
    }
}

#[test]
fn wildcard_targets_preserve_their_suffix() {
    let root = tempfile::tempdir().unwrap();
    let entry = root.path().join("src/feature/index.ts");
    std::fs::create_dir_all(entry.parent().unwrap()).unwrap();
    std::fs::write(&entry, "export {};").unwrap();
    let aliases = vec![("@parts/*".to_owned(), "src/*/index.ts".to_owned())];
    assert_eq!(
        resolve_dependency("@parts/feature", root.path(), root.path(), &aliases),
        Some(entry)
    );
}

#[test]
fn wildcard_capture_can_be_empty_or_non_ascii() {
    let root = tempfile::tempdir().unwrap();
    for capture in ["", "部品"] {
        let file = root.path().join(format!("{capture}.vue"));
        std::fs::write(&file, "<template><div /></template>").unwrap();
        assert_eq!(
            resolve_dependency(
                &format!("@/{capture}.vue"),
                root.path(),
                root.path(),
                &[("@/*.vue".to_owned(), "*.vue".to_owned())]
            ),
            Some(file)
        );
    }
}

#[test]
fn existing_literal_and_trailing_wildcard_routes_keep_target_order() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("Child.ts");
    std::fs::write(&file, "export {};").unwrap();
    for aliases in [
        vec![("@child".to_owned(), file.to_str().unwrap().to_owned())],
        vec![
            ("@/*".to_owned(), "missing/*".to_owned()),
            ("@/*".to_owned(), "*".to_owned()),
        ],
    ] {
        let specifier = if aliases.len() == 1 {
            "@child"
        } else {
            "@/Child"
        };
        assert_eq!(
            resolve_dependency(specifier, root.path(), root.path(), &aliases),
            Some(file.clone())
        );
    }
    assert_eq!(
        resolve_dependency("./Child.js", root.path(), root.path(), &[]),
        Some(file)
    );
}

#[test]
fn a_generic_suffix_does_not_displace_a_longer_existing_wildcard_prefix() {
    let root = tempfile::tempdir().unwrap();
    let generic = root.path().join("generic/x/Child.vue");
    let specific = root.path().join("specific/Child.vue");
    for file in [&generic, &specific] {
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, "<template><div /></template>").unwrap();
    }
    let mut aliases = vec![
        ("@/*.vue".to_owned(), "generic/*.vue".to_owned()),
        ("@/x/*".to_owned(), "specific/*".to_owned()),
    ];
    for _ in 0..2 {
        assert_eq!(
            resolve_dependency("@/x/Child.vue", root.path(), root.path(), &aliases),
            Some(specific.clone()),
            "the longer wildcard prefix must retain its existing authored route"
        );
        aliases.reverse();
    }
}

#[test]
fn multiple_wildcards_do_not_become_a_supported_alias_shape() {
    let dir = tempfile::tempdir().unwrap();
    let root: &Path = dir.path();
    let child = root.join("src/components/Child.vue");
    std::fs::create_dir_all(child.parent().unwrap()).unwrap();
    std::fs::write(&child, "<template><div /></template>").unwrap();
    for (pattern, target) in [("@/*/*.vue", "src/*.vue"), ("@/*", "src/*/*.vue")] {
        let (resolved, inputs) = resolve_dependency_with_inputs(
            "@/components/Child.vue",
            root,
            root,
            &[(pattern.to_owned(), target.to_owned())],
        );
        assert_eq!(resolved, None);
        assert!(inputs.is_empty(), "invalid patterns must not add probes");
    }
}
