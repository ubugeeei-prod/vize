use super::{import_targets_path, is_builtin_attr};
use std::path::Path;

#[test]
fn test_is_builtin_attr() {
    assert!(is_builtin_attr("key"));
    assert!(is_builtin_attr("ref"));
    assert!(is_builtin_attr("v-model"));
    assert!(!is_builtin_attr("myProp"));
    assert!(!is_builtin_attr("customAttr"));
}

/// `./Button.vue` imported from `pages/` must not be treated as targeting
/// `admin/Button.vue`: relative specifiers require exact canonical equality,
/// so the prop-validation alias mapping never crosses directories.
#[test]
fn relative_import_targets_only_sibling() {
    let from_dir = Some(Path::new("pages"));
    assert!(import_targets_path(
        "./Button.vue",
        from_dir,
        Path::new("pages/Button.vue")
    ));
    assert!(!import_targets_path(
        "./Button.vue",
        from_dir,
        Path::new("admin/Button.vue")
    ));
}

/// When the parent is at the project root (flat in-memory/playground), its
/// `from_dir` is empty and `./Button.vue` normalizes to the bare
/// `Button.vue`. The relative-specifier guard prevents that bare candidate
/// from suffix-matching a nested `admin/Button.vue`, so the alias mapping
/// still does not cross directories. A root-level sibling does resolve.
#[test]
fn relative_import_from_root_targets_only_root_sibling() {
    let from_dir = Some(Path::new(""));
    assert!(import_targets_path(
        "./Button.vue",
        from_dir,
        Path::new("Button.vue")
    ));
    assert!(!import_targets_path(
        "./Button.vue",
        from_dir,
        Path::new("admin/Button.vue")
    ));
    assert!(!import_targets_path(
        "./Button.vue",
        None,
        Path::new("admin/Button.vue")
    ));
}

/// A bare specifier in a flat virtual/playground project still matches a
/// `target` that carries a directory prefix, via the component-suffix fallback.
#[test]
fn bare_import_targets_via_suffix_for_virtual_projects() {
    assert!(import_targets_path(
        "Button.vue",
        None,
        Path::new("components/Button.vue")
    ));
}
