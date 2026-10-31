//! Actual provider bytes cannot borrow a different physical re-export parent.

use std::path::{Path, PathBuf};

use serde_json::json;

use super::super::super::super::tsconfig_gen::compiler_inputs::CompilerInputs;
use super::super::{admission, manifest::Refusal};

fn project() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let guard = tempfile::tempdir().unwrap();
    let root = vize_carton::path::canonicalize_non_verbatim(guard.path());
    std::fs::write(
        root.join("package.json"),
        r#"{"name":"physical-provider-fixture"}"#,
    )
    .unwrap();
    let package = root.join("node_modules/vue-router");
    for (path, source) in super::payloads("preview", &package) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, source.as_bytes()).unwrap();
    }
    (guard, root, package)
}

fn observe(root: &Path) -> Result<(), Refusal> {
    let compiler = json!({"moduleResolution":100});
    let inputs = CompilerInputs::default();
    let result = admission(root, compiler.as_object().unwrap(), &inputs, &|_, _| {});
    inputs.validate().unwrap();
    result
}

#[test]
fn real_provider_declarations_keep_their_physical_package_parents() {
    let (_guard, root, _) = project();
    assert_eq!(observe(&root), Ok(()));
}

#[cfg(unix)]
#[test]
fn symlinked_manifest_entry_and_reexport_declarations_are_refused() {
    for relative in [
        "package.json",
        "dist/experimental/index.d.ts",
        "dist/index-BA_Jc-EQ.d.ts",
    ] {
        let (_guard, root, package) = project();
        let outside = tempfile::tempdir().unwrap();
        let file = package.join(relative);
        let moved = outside.path().join("provider-file");
        std::fs::rename(&file, &moved).unwrap();
        std::os::unix::fs::symlink(&moved, &file).unwrap();
        assert_eq!(observe(&root).unwrap_err().code, "ambiguous-provider-file");
    }
}

#[cfg(unix)]
#[test]
fn whole_package_links_keep_the_same_physical_relative_graph() {
    let (_guard, root, package) = project();
    let physical = root.join("owned-provider");
    std::fs::rename(&package, &physical).unwrap();
    std::os::unix::fs::symlink(&physical, &package).unwrap();
    assert_eq!(observe(&root), Ok(()));
}
