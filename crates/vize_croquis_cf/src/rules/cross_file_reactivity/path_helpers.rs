use crate::module_paths::{import_candidates, normalize_logical_path};
use std::path::Path;
pub(super) fn import_targets_path(specifier: &str, from_dir: Option<&Path>, target: &Path) -> bool {
    let normalized_target = normalize_logical_path(target.to_path_buf());
    // The component-suffix fallback (`target` ends with the candidate path) lets
    // a flat in-memory/playground file matched by bare filename line up with a
    // `target` that carries a directory prefix. It must NOT apply to relative
    // specifiers (`./`, `../`): their directory is meaningful, so `./Child.vue`
    // may only match its sibling, never a same-named file in a different
    // directory. Relative specifiers therefore require exact canonical equality.
    let allow_suffix = !is_relative_specifier(specifier);
    import_candidates(specifier, from_dir)
        .into_iter()
        .any(|candidate| {
            candidate == normalized_target
                || (allow_suffix && normalized_target.ends_with(&candidate))
        })
}

/// Whether an import specifier is relative (`./` or `../`).
fn is_relative_specifier(specifier: &str) -> bool {
    specifier.starts_with("./")
        || specifier.starts_with("../")
        || specifier == "."
        || specifier == ".."
}

#[cfg(test)]
mod tests {
    use super::import_targets_path;
    use std::path::Path;

    /// Relative composable imports must resolve to their sibling only; a
    /// same-named module in a different directory must not be treated as the
    /// flow target.
    #[test]
    fn relative_composable_import_does_not_cross_directories() {
        let from_dir = Some(Path::new("features/cart"));
        assert!(import_targets_path(
            "./useCart.ts",
            from_dir,
            Path::new("features/cart/useCart.ts")
        ));
        assert!(!import_targets_path(
            "./useCart.ts",
            from_dir,
            Path::new("features/admin/useCart.ts")
        ));
    }

    /// The cross-directory hazard manifests when the importing file is at the
    /// project root (flat in-memory/playground), so `from_dir` is empty: a
    /// relative `./useCart.ts` then normalizes to the bare `useCart.ts`. Without
    /// the relative-specifier guard the bare candidate would suffix-match a
    /// `target` carrying any directory prefix (e.g. `features/admin/useCart.ts`),
    /// crossing directories. The guard requires exact equality for relative
    /// specifiers, so only a root-level sibling matches.
    #[test]
    fn relative_import_from_root_does_not_suffix_match_nested_target() {
        // Importing file is at the root: `parent.path.parent()` is empty.
        let from_dir = Some(Path::new(""));
        // Sibling at the root resolves.
        assert!(import_targets_path(
            "./useCart.ts",
            from_dir,
            Path::new("useCart.ts")
        ));
        // A nested same-named module must NOT be treated as the target.
        assert!(!import_targets_path(
            "./useCart.ts",
            from_dir,
            Path::new("features/admin/useCart.ts")
        ));
        // Same hazard with `from_dir = None`.
        assert!(!import_targets_path(
            "./useCart.ts",
            None,
            Path::new("features/admin/useCart.ts")
        ));
    }

    /// A bare specifier in a flat virtual/playground project still matches a
    /// directory-prefixed target via the component-suffix fallback.
    #[test]
    fn bare_composable_import_targets_via_suffix() {
        assert!(import_targets_path(
            "useCart",
            None,
            Path::new("composables/useCart.ts")
        ));
    }
}
