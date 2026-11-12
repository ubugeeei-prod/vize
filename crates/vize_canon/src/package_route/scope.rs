//! Which package a file under a package root belongs to.
//!
//! A library repository often nests its own consumers: nuxt/ui keeps `docs/`
//! and `playgrounds/nuxt/` below the package root, each with its own
//! `package.json`, and each reaches the library through `node_modules` like
//! any other dependency. Path containment alone would make those consumers a
//! part of the library.

use std::path::Path;

use serde_json::Value;

/// Whether a bare specifier for `package_root`'s own name, written in
/// `importer`, is a Node self-reference. Node resolves one only from the
/// importer's nearest package scope, and any `package.json` starts a scope.
pub fn is_package_self_reference_scope(importer: &Path, package_root: &Path) -> bool {
    importer.starts_with(package_root)
        && importer
            .ancestors()
            .skip(1)
            .take_while(|dir| *dir != package_root)
            .all(|dir| !dir.join("package.json").is_file())
}

/// Whether `source` is a member of the package rooted at `package_root`, not
/// of a named package nested inside it. A nameless `package.json` (a
/// `{"type": "module"}` marker under `dist/`) only scopes module format, so it
/// does not split the package.
pub fn is_package_member(source: &Path, package_root: &Path) -> bool {
    source.starts_with(package_root)
        && source
            .ancestors()
            .skip(1)
            .take_while(|dir| *dir != package_root)
            .all(|dir| !is_named_package_root(dir))
}

/// The root of the named package `source` belongs to: its nearest ancestor
/// whose `package.json` has a `name`.
pub fn named_package_root(source: &Path) -> Option<&Path> {
    source
        .ancestors()
        .skip(1)
        .find(|dir| is_named_package_root(dir))
}

fn is_named_package_root(dir: &Path) -> bool {
    let manifest = dir.join("package.json");
    if !manifest.is_file() {
        return false;
    }
    std::fs::read(&manifest)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
        .is_some_and(|manifest| manifest.get("name").and_then(Value::as_str).is_some())
}

#[cfg(test)]
mod tests {
    use super::{is_package_member, is_package_self_reference_scope};

    #[test]
    fn nested_named_packages_are_neither_members_nor_self_references() {
        let root = tempfile::tempdir().unwrap();
        let library = root.path();
        for (path, manifest) in [
            ("package.json", r#"{"name":"@x/ui"}"#),
            ("docs/package.json", r#"{"name":"docs"}"#),
            ("dist/esm/package.json", r#"{"type":"module"}"#),
        ] {
            let path = library.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, manifest).unwrap();
        }
        let docs = library.join("docs/app/Page.vue");
        let source = library.join("src/runtime/Chip.vue");
        let marked = library.join("dist/esm/index.d.ts");

        assert!(!is_package_self_reference_scope(&docs, library));
        assert!(!is_package_member(&docs, library));
        assert!(is_package_self_reference_scope(&source, library));
        assert!(is_package_member(&source, library));
        // A format marker starts a Node scope, but not a nested package.
        assert!(!is_package_self_reference_scope(&marked, library));
        assert!(is_package_member(&marked, library));
    }
}
