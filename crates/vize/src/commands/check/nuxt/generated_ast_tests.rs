use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use vize_canon::virtual_ts::VirtualTsOptions;
use vize_s0::cstr;

use super::detect_nuxt_auto_imports;

#[test]
fn generated_component_import_rewrite_ignores_string_literal_import_text() {
    let project_root = unique_case_dir("nuxt-component-string-import-text");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join(".nuxt")).unwrap();
    std::fs::write(project_root.join("nuxt.config.ts"), "export default {}").unwrap();
    std::fs::write(
        project_root.join(".nuxt/components.d.ts"),
        r#"declare module 'vue' {
  export interface GlobalComponents {
    StringImportText: "import('../components/Fake.vue')"
  }
}
export {}
"#,
    )
    .unwrap();

    let mut options = VirtualTsOptions::default();
    let _ = detect_nuxt_auto_imports(&mut options, &project_root);
    let stubs = options.auto_import_stubs.join("\n");

    assert!(
        stubs.contains("declare const StringImportText: \"import("),
        "expected string-literal import text to remain a string type, got:\n{stubs}"
    );
    assert!(
        !stubs.contains("StringImportText: \"import('./components/Fake.vue.ts')"),
        "string-literal import text should not be rewritten as a component import type, got:\n{stubs}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn detects_root_generated_manifest_multiline_exports() {
    let project_root = unique_case_dir("nuxt-root-import-manifest");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join(".nuxt")).unwrap();
    std::fs::create_dir_all(project_root.join("app/composables")).unwrap();
    std::fs::write(project_root.join("nuxt.config.ts"), "export default {}").unwrap();
    std::fs::write(
        project_root.join("app/composables/use-alpha.ts"),
        "export const useAlpha = () => true\nexport const useBeta = () => true\n",
    )
    .unwrap();
    std::fs::write(
        project_root.join(".nuxt/imports.d.ts"),
        r#"export {
  useAlpha,
  useBeta as useRenamedBeta,
} from '../app/composables/use-alpha'
"#,
    )
    .unwrap();

    let mut options = VirtualTsOptions::default();
    let _ = detect_nuxt_auto_imports(&mut options, &project_root);
    let stubs = options.auto_import_stubs.join("\n");

    assert!(
        stubs.contains("declare const useAlpha: typeof import("),
        "expected multiline manifest export useAlpha stub, got:\n{stubs}"
    );
    assert!(
        stubs.contains("declare const useRenamedBeta: typeof import(")
            && stubs.contains(")['useBeta'];"),
        "expected aliased multiline manifest export stub, got:\n{stubs}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn generated_import_missing_check_ignores_string_literal_import_text() {
    let project_root = unique_case_dir("nuxt-string-import-text");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join(".nuxt/types")).unwrap();
    std::fs::write(project_root.join("nuxt.config.ts"), "export default {}").unwrap();
    std::fs::write(
        project_root.join(".nuxt/types/imports.d.ts"),
        r#"declare global {
  const literalImportText: "import('../composables/missing')"
}
export {}
"#,
    )
    .unwrap();

    let mut options = VirtualTsOptions::default();
    let _ = detect_nuxt_auto_imports(&mut options, &project_root);
    let stubs = options.auto_import_stubs.join("\n");

    assert!(
        stubs.contains("declare const literalImportText: \"import("),
        "expected string-literal import text to remain precise, got:\n{stubs}"
    );
    assert!(
        !stubs.contains("declare const literalImportText: any;"),
        "string-literal import text should not trigger missing import fallback, got:\n{stubs}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

fn unique_case_dir(name: &str) -> std::path::PathBuf {
    static NEXT_CASE_ID: AtomicUsize = AtomicUsize::new(0);

    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist");
    let case_id = NEXT_CASE_ID.fetch_add(1, Ordering::Relaxed);
    workspace_root
        .join("target")
        .join("vize-tests")
        .join(cstr!("{name}-{}-{case_id}", std::process::id()).as_str())
}
