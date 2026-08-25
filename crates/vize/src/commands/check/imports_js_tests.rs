use super::*;
use vize_s0::path::canonicalize_non_verbatim;

fn write(root: &Path, relative: &str, contents: &str) -> PathBuf {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&path, contents).unwrap();
    path
}

#[test]
fn allow_js_collects_the_imported_javascript_family() {
    let root = std::env::temp_dir().join(cstr!("vize-imports-js-family-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let entry = write(
        &root,
        "src/entry.ts",
        "import '../support/plain'\nimport '../support/esm.mjs'\nimport '../support/common.cjs'\nimport '../support/view.jsx'\n",
    );
    let expected = [
        write(&root, "support/plain.js", "const plain = true\n"),
        write(&root, "support/esm.mjs", "const esm = true\n"),
        write(&root, "support/common.cjs", "const common = true\n"),
        write(&root, "support/view.jsx", "const view = true\n"),
    ]
    .map(|path| canonicalize_non_verbatim(&path));

    let disabled = collect_transitive_local_imports(
        std::slice::from_ref(&entry),
        &root,
        &mut CanonicalPathCache::default(),
        ImportFileOptions::default(),
        None,
    );
    let enabled = collect_transitive_local_imports(
        &[entry],
        &root,
        &mut CanonicalPathCache::default(),
        ImportFileOptions {
            include_js: true,
            include_jsx: false,
        },
        None,
    );

    assert!(disabled.registrations.is_empty());
    assert_eq!(enabled.registrations, expected);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn allow_js_keeps_typescript_substitution_ahead_of_javascript() {
    let root =
        std::env::temp_dir().join(cstr!("vize-imports-js-precedence-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let entry = write(&root, "src/entry.ts", "import '../support/module.js'\n");
    write(&root, "support/module.js", "export const source = 'js'\n");
    let typescript = write(&root, "support/module.ts", "export const source = 'ts'\n");

    let discovered = collect_transitive_local_imports(
        &[entry],
        &root,
        &mut CanonicalPathCache::default(),
        ImportFileOptions {
            include_js: true,
            include_jsx: false,
        },
        None,
    );

    assert_eq!(
        discovered.registrations,
        vec![canonicalize_non_verbatim(&typescript)]
    );
    let _ = std::fs::remove_dir_all(root);
}
