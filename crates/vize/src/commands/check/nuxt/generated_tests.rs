use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use vize_canon::virtual_ts::VirtualTsOptions;

use super::detect_nuxt_auto_imports;

fn unique_case_dir(name: &str) -> PathBuf {
    static NEXT_CASE_ID: AtomicUsize = AtomicUsize::new(0);

    let case_id = NEXT_CASE_ID.fetch_add(1, Ordering::Relaxed);
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("vize-tests")
        .join(format!(
            "check-nuxt-generated-{name}-{}-{case_id}",
            std::process::id()
        ))
}

#[test]
fn detects_module_declaration_generated_imports() {
    let project_root = unique_case_dir("module-declaration-imports");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join(".nuxt/types")).unwrap();
    std::fs::create_dir_all(project_root.join("app/composables")).unwrap();
    std::fs::write(project_root.join("nuxt.config.ts"), "export default {}").unwrap();
    std::fs::write(
        project_root.join("app/composables/modern.ts"),
        "export const useModernImport = () => 'ok';\n",
    )
    .unwrap();
    std::fs::write(
        project_root.join(".nuxt/types/imports.d.mts"),
        r#"declare global {
  const useModernImport: typeof import('../../app/composables/modern')['useModernImport']
}
export {}
"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join(".nuxt/types/components.d.cts"),
        r#"declare module 'vue' {
  export interface GlobalComponents {
    ModernCard: typeof import('../../app/components/ModernCard.vue')['default']
  }
  export interface ComponentCustomProperties {
    $modern: string
  }
}
export {}
"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join(".nuxt/types/nitro-imports.d.mts"),
        r#"declare global {
  const shouldNotAppear: typeof import('../../app/composables/nitro')['shouldNotAppear']
}
export {}
"#,
    )
    .unwrap();

    let mut options = VirtualTsOptions::default();
    let _ = detect_nuxt_auto_imports(&mut options, &project_root);
    let stubs = options.auto_import_stubs.join("\n");

    assert!(
        stubs.contains("declare const useModernImport: typeof import("),
        "expected imports.d.mts auto-import stub, got:\n{stubs}"
    );
    assert!(
        stubs.contains("declare const ModernCard: typeof import("),
        "expected components.d.cts component stub, got:\n{stubs}"
    );
    assert!(
        !stubs.contains("shouldNotAppear"),
        "nitro-imports.d.mts should stay skipped, got:\n{stubs}"
    );
    assert!(
        !stubs.contains("declare function useRouter(): any;"),
        "imports.d.mts should count as generated imports and suppress fallback stubs:\n{stubs}"
    );
    assert!(
        options
            .template_globals
            .iter()
            .any(|global| global.name == "$modern"),
        "expected ComponentCustomProperties from .d.cts, got: {:#?}",
        options.template_globals
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn expands_generated_import_type_aliases() {
    let project_root = unique_case_dir("import-type-aliases");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join(".nuxt/types")).unwrap();
    std::fs::create_dir_all(project_root.join("app/types")).unwrap();
    std::fs::write(project_root.join("nuxt.config.ts"), "export default {}").unwrap();
    std::fs::write(
        project_root.join("app/types/user.ts"),
        "export type User = { id: string };\nexport type Wrapped = { ok: true };\n",
    )
    .unwrap();
    std::fs::write(
        project_root.join("app/types/default.ts"),
        "export default interface DefaultThing { id: string }\n",
    )
    .unwrap();
    std::fs::write(
        project_root.join("app/types/shared.ts"),
        "export interface Thing { name: string }\n",
    )
    .unwrap();
    std::fs::write(
        project_root.join(".nuxt/types/imports.d.ts"),
        r#"import type { User, Wrapped as AliasedWrapped } from '../../app/types/user'
import type DefaultThing from '../../app/types/default'
import type * as Shared from '../../app/types/shared'
declare global {
  const useTypedUser: () => User
  const useWrapped: () => AliasedWrapped
  const useDefaultThing: () => DefaultThing
  const useSharedThing: () => Shared.Thing
}
export {}
"#,
    )
    .unwrap();

    let mut options = VirtualTsOptions::default();
    let _ = detect_nuxt_auto_imports(&mut options, &project_root);
    let stubs = options.auto_import_stubs.join("\n");
    let user = project_root
        .join("app/types/user")
        .to_string_lossy()
        .replace('\\', "/");
    let default = project_root
        .join("app/types/default")
        .to_string_lossy()
        .replace('\\', "/");
    let shared = project_root
        .join("app/types/shared")
        .to_string_lossy()
        .replace('\\', "/");

    assert!(
        stubs.contains(&format!(
            "declare const useTypedUser: () => import('{user}').User;"
        )),
        "expected named import type alias expansion, got:\n{stubs}"
    );
    assert!(
        stubs.contains(&format!(
            "declare const useWrapped: () => import('{user}').Wrapped;"
        )),
        "expected aliased import type expansion, got:\n{stubs}"
    );
    assert!(
        stubs.contains(&format!(
            "declare const useDefaultThing: () => import('{default}').default;"
        )),
        "expected default import type expansion, got:\n{stubs}"
    );
    assert!(
        stubs.contains(&format!(
            "declare const useSharedThing: () => import('{shared}').Thing;"
        )),
        "expected namespace import type expansion, got:\n{stubs}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn detects_root_module_declaration_import_manifest() {
    let project_root = unique_case_dir("root-module-declaration-imports");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join(".nuxt")).unwrap();
    std::fs::create_dir_all(project_root.join("app/composables")).unwrap();
    std::fs::write(project_root.join("nuxt.config.ts"), "export default {}").unwrap();
    std::fs::write(
        project_root.join("app/composables/root.ts"),
        "export const useRootImport = () => 'ok';\n",
    )
    .unwrap();
    std::fs::write(
        project_root.join(".nuxt/imports.d.cts"),
        r#"export { useRootImport } from '../app/composables/root';
"#,
    )
    .unwrap();

    let mut options = VirtualTsOptions::default();
    let _ = detect_nuxt_auto_imports(&mut options, &project_root);
    let stubs = options.auto_import_stubs.join("\n");

    assert!(
        stubs.contains("declare const useRootImport: typeof import("),
        "expected root imports.d.cts auto-import stub, got:\n{stubs}"
    );
    assert!(
        !stubs.contains("declare function useRouter(): any;"),
        "root imports.d.cts should suppress fallback stubs:\n{stubs}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
