use super::*;

#[test]
fn explicit_subset_registers_relative_import_above_tsconfig() {
    let root = unique_case_dir("explicit-relative-outside-tsconfig");
    let _ = std::fs::remove_dir_all(&root);
    let app = root.join("app");
    let main = write(
        &root,
        "app/src/main.ts",
        "import { VALUE } from '../../shared/data'; export const doubled = VALUE * 2;\n",
    );
    let shared = write(&root, "shared/data.ts", "export const VALUE: number = 1;\n");
    let tsconfig = write(
        &root,
        "app/tsconfig.json",
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"Bundler"},"include":["src/**/*.ts"]}"#,
    );
    let mut files = vec![canonicalize_non_verbatim(&main)];
    let mut canonical_paths = super::super::CanonicalPathCache::default();
    let mut resolver = vize_canon::PackageRouteResolver::default();
    let mut session = super::super::LocalImportSession::new(&mut resolver);
    super::super::register_transitive_local_imports_with_session(
        &mut files,
        super::super::LocalImportContext {
            cwd: &app,
            tsconfig_path: Some(&tsconfig),
            import_options: super::super::ImportFileOptions::default(),
        },
        &mut canonical_paths,
        &mut resolver,
        &mut session,
    );
    assert!(files.contains(&canonicalize_non_verbatim(&shared)));
    let _ = std::fs::remove_dir_all(&root);
}
