use std::path::{Path, PathBuf};

use vize_carton::cstr;

use super::collect_passthrough_modules;

fn unique_case_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(cstr!("vize-passthrough-{name}-{}", std::process::id()).as_str())
}

fn write(root: &Path, rel: &str, content: &str) -> PathBuf {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, content).unwrap();
    path
}

#[test]
fn collects_declaration_for_explicit_ts_relative_import() {
    let case_dir = unique_case_dir("explicit-ts-declaration");
    let _ = std::fs::remove_dir_all(&case_dir);
    std::fs::create_dir_all(&case_dir).unwrap();
    let root = vize_carton::path::canonicalize_non_verbatim(&case_dir);
    let entry = write(
        &root,
        ".nuxt/types/i18n-plugin.d.ts",
        "import type { ComposerCustomProperties } from '../../node_modules/@nuxtjs/i18n/dist/runtime/types.ts'\nexport {}\n",
    );
    write(
        &root,
        "node_modules/@nuxtjs/i18n/dist/runtime/types.d.ts",
        "export interface ComposerCustomProperties {}\n",
    );
    let virtual_root = crate::batch::project_virtual_root(&root);

    let mut files = collect_passthrough_modules(
        &entry,
        &std::fs::read_to_string(&entry).unwrap(),
        &root,
        &virtual_root,
    );
    files.sort();

    assert_eq!(
        files
            .into_iter()
            .map(|(virtual_path, original_path)| {
                (
                    virtual_path
                        .strip_prefix(&virtual_root)
                        .unwrap()
                        .to_path_buf(),
                    original_path.strip_prefix(&root).unwrap().to_path_buf(),
                )
            })
            .collect::<Vec<_>>(),
        vec![(
            PathBuf::from("node_modules/@nuxtjs/i18n/dist/runtime/types.d.ts"),
            PathBuf::from("node_modules/@nuxtjs/i18n/dist/runtime/types.d.ts"),
        )]
    );

    let _ = std::fs::remove_dir_all(&case_dir);
}
