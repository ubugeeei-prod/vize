use super::{reference_type_packages, resolve_type_reference_declaration_files};
use std::path::{Path, PathBuf};

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn unique_case_dir(name: &str) -> PathBuf {
    static NEXT_CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "vize-type-reference-{name}-{}-{case_id}",
        std::process::id()
    ))
}

fn relative_paths(root: &Path, files: &[PathBuf]) -> Vec<std::string::String> {
    files
        .iter()
        .map(|path| {
            path.strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect()
}

#[test]
fn reference_type_packages_collects_quoted_directives() {
    assert_eq!(
        reference_type_packages(
            r#"
/// <reference types="vitest/importMeta" />
  /// <reference types='@vizejs/vite-plugin-musea/client' />
/// <reference path="./local.d.ts" />
"#,
        ),
        vec![
            "vitest/importMeta".to_string(),
            "@vizejs/vite-plugin-musea/client".to_string(),
        ]
    );
}

#[test]
fn type_reference_resolution_supports_subpaths_exports_and_graphs() {
    let root = unique_case_dir("subpath");
    let _ = std::fs::remove_dir_all(&root);
    write(
        &root,
        "node_modules/vitest/importMeta.d.ts",
        "/// <reference path=\"./globals.d.ts\" />\ndeclare global { interface ImportMeta { vitest: boolean; } }\n",
    );
    write(
        &root,
        "node_modules/vitest/globals.d.ts",
        "declare const describe: (name: string) => void;\n",
    );
    write(
        &root,
        "node_modules/@vizejs/vite-plugin-musea/package.json",
        r#"{ "exports": { "./client": { "types": "./client.d.ts" } } }"#,
    );
    write(
        &root,
        "node_modules/@vizejs/vite-plugin-musea/client.d.ts",
        "declare function defineArt(source: string): void;\n",
    );

    let root = root.canonicalize().unwrap();
    let vitest = resolve_type_reference_declaration_files(&root, "vitest/importMeta");
    assert_eq!(
        relative_paths(&root, &vitest),
        vec![
            "node_modules/vitest/importMeta.d.ts",
            "node_modules/vitest/globals.d.ts",
        ]
    );
    let musea = resolve_type_reference_declaration_files(&root, "@vizejs/vite-plugin-musea/client");
    assert_eq!(
        relative_paths(&root, &musea),
        vec!["node_modules/@vizejs/vite-plugin-musea/client.d.ts"]
    );

    let _ = std::fs::remove_dir_all(&root);
}
