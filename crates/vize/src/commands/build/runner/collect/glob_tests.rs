#![allow(clippy::disallowed_macros, clippy::disallowed_methods)]

use super::BuildGlob;
use std::path::{Path, PathBuf};

#[test]
fn matches_segment_wildcards_and_character_classes() {
    let single_segment = BuildGlob::new("src/*/App?.vue").unwrap();
    assert!(single_segment.matches(Path::new("src/admin/App1.vue")));
    assert!(!single_segment.matches(Path::new("src/admin/deep/App1.vue")));
    assert!(!single_segment.matches(Path::new("src/admin/App12.vue")));

    let class = BuildGlob::new("src/[AB]*[0-9].vue").unwrap();
    assert!(class.matches(Path::new("src/App2.vue")));
    assert!(class.matches(Path::new("src/Button7.vue")));
    assert!(!class.matches(Path::new("src/Card7.vue")));
}

#[test]
fn recursive_wildcards_match_zero_or_many_directories() {
    let recursive = BuildGlob::new("src/**/App.vue").unwrap();
    assert_eq!(recursive.max_depth(), None);
    assert!(recursive.matches(Path::new("src/App.vue")));
    assert!(recursive.matches(Path::new("src/a/b/App.vue")));
    assert!(!recursive.matches(Path::new("other/src/App.vue")));
}

#[test]
fn rejects_invalid_patterns_with_original_positions() {
    let unclosed = BuildGlob::new("./src/[AB.vue").unwrap_err();
    assert_eq!(
        unclosed.to_string(),
        "Pattern syntax error near position 6: invalid range pattern"
    );

    let embedded_recursive = BuildGlob::new("src/foo**/App.vue").unwrap_err();
    assert_eq!(
        embedded_recursive.to_string(),
        "Pattern syntax error near position 6: recursive wildcards must form a single path component"
    );
}

#[test]
fn preserves_deep_literal_prefixes_and_absolute_roots() {
    let nested = BuildGlob::new("workspace/packages/ui/src/App?.vue").unwrap();
    assert_eq!(nested.root(), Path::new("workspace/packages/ui/src/"));

    let absolute = BuildGlob::new("/workspace/src/**/*.vue").unwrap();
    assert_eq!(absolute.root(), Path::new("/workspace/src/"));
    assert!(absolute.matches(Path::new("/workspace/src/nested/App.vue")));
    assert!(!absolute.matches(Path::new("/other/workspace/src/App.vue")));

    assert_eq!(BuildGlob::new("/**/*.vue").unwrap().root(), Path::new("/"));
    let windows_root = BuildGlob::new(r"C:\**\*.vue").unwrap();
    let expected_windows_root = if cfg!(windows) { r"C:\" } else { "C:/" };
    assert_eq!(windows_root.root().to_string_lossy(), expected_windows_root);
}

#[test]
fn bracket_expressions_escape_metacharacters() {
    let escaped = BuildGlob::new("src/Component[[]old[]]-[*]-[?].vue").unwrap();
    assert!(escaped.matches(Path::new("src/Component[old]-*-?.vue")));
    assert!(!escaped.matches(Path::new("src/Componentxold]-*-?.vue")));
}

#[test]
fn accepts_windows_separators_portably() {
    let pattern = BuildGlob::new(r"C:\workspace\src\**\Card?.vue").unwrap();
    assert!(pattern.matches(Path::new(r"C:\workspace\src\nested\Card1.vue")));
    assert!(!pattern.matches(Path::new(r"C:\workspace\other\nested\Card1.vue")));
}

#[test]
fn uses_native_case_sensitivity() {
    let pattern = BuildGlob::new("src/card?.vue").unwrap();
    assert_eq!(pattern.matches(Path::new("src/CardA.vue")), cfg!(windows));
}

#[cfg(windows)]
#[test]
fn windows_matching_is_case_insensitive_and_accepts_forward_slashes() {
    let pattern = BuildGlob::new("C:/workspace/src/card?.vue").unwrap();
    assert!(pattern.matches(Path::new(r"C:\workspace\src\CardA.vue")));
}

#[test]
fn glob_root_without_a_literal_directory_is_current_directory() {
    let root_pattern = BuildGlob::new("*.vue").unwrap();
    assert_eq!(root_pattern.root(), PathBuf::from("."));
    assert_eq!(root_pattern.max_depth(), Some(1));

    let nested = BuildGlob::new("src/*/App?.vue").unwrap();
    assert_eq!(nested.max_depth(), Some(2));
}
