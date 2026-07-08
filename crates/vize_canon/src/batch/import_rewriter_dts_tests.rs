use super::ImportRewriter;
use oxc_span::SourceType;
use std::fs;
use std::path::{Path, PathBuf};
use vize_carton::cstr;

fn unique_case_dir(name: &str) -> PathBuf {
    static NEXT_CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    std::env::temp_dir().join(
        cstr!(
            "vize-import-rewriter-dts-{name}-{}-{case_id}",
            std::process::id()
        )
        .as_str(),
    )
}

fn write(dir: &Path, rel: &str, contents: &str) -> PathBuf {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, contents).unwrap();
    path
}

#[test]
fn rewrite_relative_generated_dts_import_type_to_real_path_for_virtual_project() {
    let raw_root = unique_case_dir("relative-generated-dts-import-type");
    let _ = fs::remove_dir_all(&raw_root);
    let schema = write(
        &raw_root,
        "types/codegen/schema.d.ts",
        "export type Schema = { __typename: 'Schema' }\n",
    );
    let barrel = write(
        &raw_root,
        "types/index.ts",
        "export type Schema = import('./codegen/schema').Schema\n",
    );
    let root = vize_carton::path::canonicalize_non_verbatim(&raw_root);
    let schema = vize_carton::path::canonicalize_non_verbatim(&schema);
    let rewriter = ImportRewriter::new();
    let source = "export type Schema = import('./codegen/schema').Schema";
    let virtual_root = root.join("node_modules/.vize/canon");
    let roots = (root.as_path(), virtual_root.as_path());
    let result =
        rewriter.rewrite_for_virtual_project(source, SourceType::ts(), roots, barrel.parent());
    assert_eq!(
        result.code.as_str(),
        cstr!(
            "export type Schema = import('{}').Schema",
            schema.with_file_name("schema").display()
        )
        .as_str()
    );

    let _ = fs::remove_dir_all(&raw_root);
}

#[test]
fn ignores_relative_dts_text_outside_module_specifiers_for_virtual_project() {
    let raw_root = unique_case_dir("relative-generated-dts-text");
    let _ = fs::remove_dir_all(&raw_root);
    write(
        &raw_root,
        "types/codegen/schema.d.ts",
        "export type Schema = { __typename: 'Schema' }\n",
    );
    let barrel = write(
        &raw_root,
        "types/index.ts",
        "const note = './codegen/schema'\n// export * from './codegen/schema'\n",
    );
    let root = vize_carton::path::canonicalize_non_verbatim(&raw_root);
    let rewriter = ImportRewriter::new();
    let source = "const note = './codegen/schema'\n// export * from './codegen/schema'\n";
    let virtual_root = root.join("node_modules/.vize/canon");
    let roots = (root.as_path(), virtual_root.as_path());
    let result =
        rewriter.rewrite_for_virtual_project(source, SourceType::ts(), roots, barrel.parent());
    assert_eq!(result.code.as_str(), source);

    let _ = fs::remove_dir_all(&raw_root);
}
