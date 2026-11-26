#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
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
    // `$modern` is declared by the generated `ComponentCustomProperties`, so it
    // keeps its own declaration. Everything the generated graph does not declare
    // resolves on the component instance instead.
    assert!(options.strict_instance_globals);
    assert_eq!(
        options
            .template_globals
            .iter()
            .map(|global| global.name.as_str())
            .collect::<Vec<_>>(),
        vec!["$modern"],
        "expected only the declared ComponentCustomProperties global, got: {:#?}",
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

/// The stubs that put a type in scope: the fallback aliases of the global
/// script and the re-export stub of the module file.
fn type_stubs(options: &VirtualTsOptions) -> Vec<&str> {
    options
        .auto_import_stubs
        .iter()
        .map(|stub| stub.as_str())
        .filter(|stub| {
            stub.starts_with("type ") || stub.starts_with("// @vize-module-augmentation")
        })
        .collect()
}

/// A generated manifest re-exports its auto-imported types from inside a
/// global augmentation. Every one of them is declared once: a plain type, a
/// generic type, a renamed export, a package type, and `Ref`, which the
/// fallback aliases would otherwise declare a second time with one type
/// parameter instead of two.
#[test]
fn reexports_the_types_a_generated_manifest_auto_imports() {
    let project_root = unique_case_dir("type-reexports");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join(".nuxt/types")).unwrap();
    std::fs::create_dir_all(project_root.join("app/composables")).unwrap();
    std::fs::write(project_root.join("nuxt.config.ts"), "export default {}").unwrap();
    std::fs::write(
        project_root.join("app/composables/states.ts"),
        "export type ColorName = 'pink' | 'blue';\nexport type Box<T> = { value: T };\n",
    )
    .unwrap();
    std::fs::write(
        project_root.join(".nuxt/types/imports.d.ts"),
        r#"export {}
declare global {
  const useColor: typeof import('../../app/composables/states').useColor
}
// for type re-export
declare global {
  // @ts-ignore
  export type { ColorName, Box, Box as Crate } from '../../app/composables/states'
  import('../../app/composables/states')
  // @ts-ignore
  export type { PageMeta } from '../../node_modules/nuxt/dist/pages/runtime/composables'
  import('../../node_modules/nuxt/dist/pages/runtime/composables')
  // @ts-ignore
  export type { ComputedRef, InjectionKey, Ref } from 'vue'
  import('vue')
}
"#,
    )
    .unwrap();

    let mut options = VirtualTsOptions::default();
    let _ = detect_nuxt_auto_imports(&mut options, &project_root);
    let root = project_root.to_string_lossy().replace('\\', "/");

    assert_eq!(
        type_stubs(&options),
        vec![
            format!(
                "// @vize-module-augmentation\n\
                 declare global {{\n  \
                 export type {{ ColorName, Box, Box as Crate }} from '{root}/app/composables/states'\n  \
                 import('{root}/app/composables/states')\n  \
                 export type {{ PageMeta }} from '{root}/node_modules/nuxt/dist/pages/runtime/composables'\n  \
                 import('{root}/node_modules/nuxt/dist/pages/runtime/composables')\n  \
                 export type {{ ComputedRef, InjectionKey, Ref }} from 'vue'\n  \
                 import('vue')\n\
                 }}\n"
            )
            .as_str(),
        ]
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

/// The program reads every generated declaration except the import manifest
/// itself, so only the manifest's re-exports become a stub. A manifest
/// without re-exports declares no auto-imported type, and no fallback alias
/// stands in for one: vue-tsc reports each such name as TS2304.
#[test]
fn only_an_import_manifest_supplies_type_reexport_stubs() {
    let project_root = unique_case_dir("type-reexports-manifest-only");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join(".nuxt/types")).unwrap();
    std::fs::write(project_root.join("nuxt.config.ts"), "export default {}").unwrap();
    std::fs::write(
        project_root.join(".nuxt/types/imports.d.ts"),
        "export {}\ndeclare global {\n  const useColor: () => string\n}\n",
    )
    .unwrap();
    std::fs::write(
        project_root.join(".nuxt/types/shared-imports.d.ts"),
        r#"export {}
declare global {
  // @ts-ignore
  export type { Ref, SharedUser } from '../../shared/types/user'
  import('../../shared/types/user')
}
"#,
    )
    .unwrap();

    let mut options = VirtualTsOptions::default();
    let _ = detect_nuxt_auto_imports(&mut options, &project_root);

    assert_eq!(type_stubs(&options), Vec::<&str>::new());

    let _ = std::fs::remove_dir_all(&project_root);
}

/// Without generated types the fallback scans the directories Nuxt would.
/// A type found there is re-exported from its module, generics included,
/// instead of collapsing to `any`; a name the fallback aliases already declare
/// keeps that alias.
#[test]
fn reexports_scanned_source_types_without_a_generated_manifest() {
    let project_root = unique_case_dir("type-reexports-source-scan");
    let _ = std::fs::remove_dir_all(&project_root);
    for dir in ["app/composables", "app/types", "shared/types"] {
        std::fs::create_dir_all(project_root.join(dir)).unwrap();
    }
    std::fs::write(project_root.join("nuxt.config.ts"), "export default {}").unwrap();
    std::fs::write(
        project_root.join("app/composables/states.ts"),
        "export type ColorName = 'pink' | 'blue';\nexport const useColor = () => 'pink';\n",
    )
    .unwrap();
    std::fs::write(
        project_root.join("app/types/box.ts"),
        "export interface Box<T> { value: T }\ntype Hidden = string;\nexport { type Hidden as Shown };\nexport type Ref = never;\n",
    )
    .unwrap();
    std::fs::write(
        project_root.join("shared/types/user.d.ts"),
        "export type SharedUser = { id: number };\nexport type Box<T> = T[];\n",
    )
    .unwrap();

    let mut options = VirtualTsOptions::default();
    let _ = detect_nuxt_auto_imports(&mut options, &project_root);
    let root = project_root.to_string_lossy().replace('\\', "/");

    assert_eq!(
        type_stubs(&options),
        vec![
            "type Composer = any;",
            "type Ref<T = any> = import('vue').Ref<T>;",
            "type ComputedRef<T = any> = import('vue').ComputedRef<T>;",
            "type WritableComputedRef<T = any> = import('vue').WritableComputedRef<T>;",
            "type ShallowRef<T = any> = import('vue').ShallowRef<T>;",
            "type UnwrapRef<T> = import('vue').UnwrapRef<T>;",
            "type UnwrapNestedRefs<T> = import('vue').UnwrapNestedRefs<T>;",
            "type MaybeRef<T = any> = import('vue').MaybeRef<T>;",
            "type MaybeRefOrGetter<T = any> = import('vue').MaybeRefOrGetter<T>;",
            "type Component = import('vue').Component;",
            format!(
                "// @vize-module-augmentation\n\
                 declare global {{\n  \
                 export type {{ Box, Shown }} from '{root}/app/types/box'\n  \
                 import('{root}/app/types/box')\n  \
                 export type {{ ColorName }} from '{root}/app/composables/states'\n  \
                 import('{root}/app/composables/states')\n  \
                 export type {{ SharedUser }} from '{root}/shared/types/user'\n  \
                 import('{root}/shared/types/user')\n\
                 }}\n"
            )
            .as_str(),
        ]
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
