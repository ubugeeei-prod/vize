//! Detection over the generated types of a Nuxt 4 project: components on
//! `_GlobalComponents`, file-local component types, and package declarations
//! the generated `nuxt.d.ts` references.

#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use vize_canon::virtual_ts::VirtualTsOptions;
use vize_l0::{String, config::VueVersion};

use super::super::tsconfig_inputs::normalize_input_path;
use super::detect;
use super::stubs::declared_name;

fn unique_case_dir(name: &str) -> PathBuf {
    static NEXT_CASE_ID: AtomicUsize = AtomicUsize::new(0);

    let case_id = NEXT_CASE_ID.fetch_add(1, Ordering::Relaxed);
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("vize-tests")
        .join(format!(
            "check-nuxt4-generated-{name}-{}-{case_id}",
            std::process::id()
        ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    normalize_input_path(&root)
}

fn write(root: &Path, path: &str, content: &str) -> PathBuf {
    let path = root.join(path);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, content).unwrap();
    path
}

/// A Nuxt 4 project as `nuxi prepare` leaves it, trimmed to one component of
/// each kind. `program` lists the declarations the program loads itself.
fn nuxt4_project(name: &str) -> (PathBuf, Vec<PathBuf>) {
    let root = unique_case_dir(name);
    write(
        &root,
        "nuxt.config.ts",
        "export default defineNuxtConfig({})\n",
    );
    write(
        &root,
        "tsconfig.json",
        r#"{ "include": [".nuxt/nuxt.d.ts", "app/**/*"] }"#,
    );
    write(
        &root,
        ".nuxt/nuxt.d.ts",
        "/// <reference types=\"nitro-kit\" />\n\
         /// <reference path=\"../node_modules/.pnpm/nitro-server/dist/augments.d.mts\" />\n\
         /// <reference path=\"types/components.d.ts\" />\n\
         /// <reference path=\"types/imports.d.ts\" />\n\
         export {}\n",
    );
    write(
        &root,
        "node_modules/.pnpm/nitro-server/dist/augments.d.mts",
        "declare module 'nuxt/schema' { interface ServerTypes { event: { path: string } } }\nexport {}\n",
    );
    write(
        &root,
        "node_modules/nitro-kit/package.json",
        r#"{ "name": "nitro-kit", "types": "./index.d.ts" }"#,
    );
    write(
        &root,
        "node_modules/nitro-kit/index.d.ts",
        "export interface NitroKit { ready: boolean }\n",
    );
    write(
        &root,
        "node_modules/nuxt/dist/app/components/nuxt-time.vue.d.ts",
        "declare const NuxtTime: import('vue').DefineComponent<{ datetime: string | number | Date }>;\nexport default NuxtTime;\n",
    );
    let components = write(
        &root,
        ".nuxt/types/components.d.ts",
        r#"import type { DefineComponent } from 'vue'

type HydrationStrategies = {
  hydrateNever?: true
}
type LazyComponent<T> = DefineComponent<HydrationStrategies> & T

interface _GlobalComponents {
  NuxtLink: typeof import("../../node_modules/nuxt/dist/app/components/nuxt-link")['default']
  NuxtTime: typeof import("../../node_modules/nuxt/dist/app/components/nuxt-time.vue")['default']
  LazyCard: LazyComponent<typeof import("../../app/components/Card.vue")['default']>
}

declare module 'vue' {
  export interface GlobalComponents extends _GlobalComponents { }
}

export {}
"#,
    );
    write(
        &root,
        ".nuxt/types/imports.d.ts",
        "declare global {\n  const useCard: typeof import('../../app/composables/card')['useCard']\n}\nexport {}\n",
    );
    (root, vec![components])
}

fn detect_program(root: &Path, program: &[PathBuf]) -> Vec<String> {
    detect_program_under(root, root, program)
}

fn detect_program_under(root: &Path, program_root: &Path, program: &[PathBuf]) -> Vec<String> {
    let mut options = VirtualTsOptions::default();
    detect(
        &mut options,
        root,
        program_root,
        Some(&root.join("tsconfig.json")),
        Some(program),
        false,
        VueVersion::V3,
    )
    .unwrap();
    options.auto_import_stubs
}

fn stubs_named<'a>(stubs: &'a [String], names: &[&str]) -> Vec<&'a str> {
    stubs
        .iter()
        .filter(|stub| declared_name(stub).is_some_and(|name| names.contains(&name)))
        .map(String::as_str)
        .collect()
}

#[test]
fn installed_declarations_of_the_reference_graph_lead_the_stubs() {
    let (root, program) = nuxt4_project("installed-references");
    let stubs = detect_program(&root, &program);
    let root = root.to_str().unwrap();

    let references = stubs
        .iter()
        .take_while(|stub| stub.starts_with("///"))
        .map(String::as_str)
        .collect::<Vec<_>>();
    assert_eq!(
        references,
        vec![
            format!(
                "/// <reference path=\"{root}/node_modules/.pnpm/nitro-server/dist/augments.d.mts\" />"
            ),
            format!("/// <reference path=\"{root}/node_modules/nitro-kit/index.d.ts\" />"),
        ]
    );
}

#[test]
fn an_installed_declaration_the_program_holds_is_not_referenced_again() {
    let (root, mut program) = nuxt4_project("held-references");
    program.push(root.join("node_modules/nitro-kit/index.d.ts"));
    let stubs = detect_program(&root, &program);
    let root = root.to_str().unwrap();

    let references = stubs
        .iter()
        .take_while(|stub| stub.starts_with("///"))
        .map(String::as_str)
        .collect::<Vec<_>>();
    assert_eq!(
        references,
        vec![format!(
            "/// <reference path=\"{root}/node_modules/.pnpm/nitro-server/dist/augments.d.mts\" />"
        )]
    );
}

#[test]
fn nuxt4_components_are_typed_with_their_local_types() {
    let (root, program) = nuxt4_project("components");
    let stubs = detect_program(&root, &program);
    let root = root.to_str().unwrap();

    assert_eq!(
        stubs_named(&stubs, &["NuxtLink", "NuxtTime", "LazyCard"]),
        vec![
            format!(
                "declare const NuxtLink: typeof import(\"{root}/node_modules/nuxt/dist/app/components/nuxt-link\")['default'];"
            ),
            // The package's own declaration types the installed `.vue` file.
            format!(
                "declare const NuxtTime: typeof import(\"{root}/node_modules/nuxt/dist/app/components/nuxt-time.vue\")['default'];"
            ),
            format!(
                "declare const LazyCard: __VizeNuxtLocalTypes__2e_nuxt_2f_types_2f_components_2e_d_2e_ts.LazyComponent<typeof import(\"./app/components/Card.vue.ts\")['default']>;"
            ),
        ]
    );
    let namespaces = stubs
        .iter()
        .filter(|stub| stub.starts_with("declare namespace "))
        .map(String::as_str)
        .collect::<Vec<_>>();
    assert_eq!(
        namespaces,
        vec![
            "declare namespace __VizeNuxtLocalTypes__2e_nuxt_2f_types_2f_components_2e_d_2e_ts {\n\
type HydrationStrategies = {\n  hydrateNever?: true\n}\n\
type LazyComponent<T> = import('vue').DefineComponent<HydrationStrategies> & T\n\
}"
        ]
    );
}

#[test]
fn an_app_inside_a_workspace_names_component_mirrors_from_the_workspace() {
    let (root, program) = nuxt4_project("workspace-app");
    let workspace = root.parent().unwrap();
    let app = root.file_name().unwrap().to_str().unwrap();
    let stubs = detect_program_under(&root, workspace, &program);

    assert_eq!(
        stubs_named(&stubs, &["LazyCard"]),
        vec![format!(
            "declare const LazyCard: __VizeNuxtLocalTypes_{}_2f__2e_nuxt_2f_types_2f_components_2e_d_2e_ts.LazyComponent<typeof import(\"./{app}/app/components/Card.vue.ts\")['default']>;",
            app.replace('-', "_2d_")
        )]
    );
}

#[test]
fn generated_types_leave_no_fallback_type_alias_or_builtin_any() {
    let (root, program) = nuxt4_project("no-invented-globals");
    let stubs = detect_program(&root, &program);

    let type_aliases = stubs
        .iter()
        .filter(|stub| stub.starts_with("type "))
        .map(String::as_str)
        .collect::<Vec<_>>();
    assert_eq!(type_aliases, Vec::<&str>::new());
    // The built-in the generated types do not name keeps its fallback.
    let any_builtins = stubs_named(&stubs, &["NuxtLink", "NuxtPage"])
        .into_iter()
        .filter(|stub| stub.ends_with(": any;"))
        .collect::<Vec<_>>();
    assert_eq!(any_builtins, vec!["declare const NuxtPage: any;"]);
}

#[test]
fn local_type_namespaces_keep_distinct_declaration_paths_distinct() {
    use super::generated::local_types_namespace;
    let root = Path::new("/app");
    let names = [
        ".nuxt/types/ui-kit.d.ts",
        ".nuxt/types/ui_kit.d.ts",
        ".nuxt/types/ui_2d_kit.d.ts",
        ".nuxt/types/État.d.ts",
    ]
    .map(|path| local_types_namespace(root, &root.join(path)));
    let unique = names.iter().collect::<std::collections::HashSet<_>>();
    assert_eq!(unique.len(), names.len());
}

#[test]
fn local_type_aliases_rewrite_vue_imports_inside_the_namespace() {
    let (root, program) = nuxt4_project("namespace-vue-import");
    write(
        &root,
        ".nuxt/types/components.d.ts",
        "type LazyFoo = typeof import('../../app/Foo.vue')['default']; declare module 'vue' { interface GlobalComponents { LazyFoo: LazyFoo } } export {};",
    );
    let stubs = detect_program(&root, &program);
    let namespace = stubs
        .iter()
        .find(|stub| stub.starts_with("declare namespace "))
        .unwrap();
    let allocator = oxc_allocator::Allocator::default();
    let parsed =
        oxc_parser::Parser::new(&allocator, namespace.as_str(), oxc_span::SourceType::d_ts())
            .parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert_eq!(
        namespace.as_str(),
        "declare namespace __VizeNuxtLocalTypes__2e_nuxt_2f_types_2f_components_2e_d_2e_ts {\ntype LazyFoo = typeof import('./app/Foo.vue.ts')['default']\n}"
    );
}

#[cfg(unix)]
#[test]
fn linked_installed_declarations_keep_their_installed_membership() {
    let (root, program) = nuxt4_project("linked-nitro");
    let installed = root.join("node_modules/.pnpm/nitro-server");
    let real = root.join("packages/nitro-server");
    std::fs::create_dir_all(real.parent().unwrap()).unwrap();
    std::fs::rename(&installed, &real).unwrap();
    std::os::unix::fs::symlink(&real, &installed).unwrap();
    let stubs = detect_program(&root, &program);
    let augmentation = real.join("dist/augments.d.mts");
    let references = stubs
        .iter()
        .filter(|stub| stub.starts_with("///"))
        .map(String::as_str)
        .collect::<Vec<_>>();
    assert_eq!(
        references,
        vec![
            format!(
                "/// <reference path=\"{}\" />",
                root.join("node_modules/nitro-kit/index.d.ts").display()
            ),
            format!("/// <reference path=\"{}\" />", augmentation.display()),
        ]
    );
}

#[test]
fn nested_shadowing_references_keep_safe_parent_declarations_in_snapshots() {
    let (root, program) = nuxt4_project("nested-shadowing");
    let parent = write(
        &root,
        "node_modules/.pnpm/nitro-server/dist/augments.d.mts",
        "/// <reference path=\"child.d.ts\" />\nexport {}; declare global { const safeParent: number }\n",
    );
    let child = write(
        &root,
        "node_modules/.pnpm/nitro-server/dist/child.d.ts",
        "/// <reference path=\"shim.d.ts\" />\nexport {}; declare global { const safeChild: true }\n",
    );
    let shim = write(
        &root,
        "node_modules/.pnpm/nitro-server/dist/shim.d.ts",
        "declare module 'vue' { export const unrelated: true }\n",
    );
    let originals = [&parent, &child, &shim].map(|path| std::fs::read(path).unwrap());
    let mut options = VirtualTsOptions::default();
    let detection = detect(
        &mut options,
        &root,
        &root,
        Some(&root.join("tsconfig.json")),
        Some(&program),
        false,
        VueVersion::V3,
    )
    .unwrap();
    assert_eq!(detection.declaration_snapshots, vec![
        (root.join(".nuxt/nuxt.d.ts"), format!("/// <reference path=\"{}\" />\n/// <reference path=\"{}\" />\n/// <reference path=\"{}\" />\n/// <reference path=\"{}\" />\nexport {{}}\n",
            root.join("node_modules/nitro-kit/index.d.ts").display(), parent.display(), root.join(".nuxt/types/components.d.ts").display(), root.join(".nuxt/types/imports.d.ts").display()).into()),
        (parent.clone(), format!("/// <reference path=\"{}\" />\nexport {{}}; declare global {{ const safeParent: number }}\n", child.display()).into()),
        (child.clone(), "export {}; declare global { const safeChild: true }\n".into()),
    ]);
    let references = options
        .auto_import_stubs
        .iter()
        .filter(|stub| stub.starts_with("///"))
        .map(String::as_str)
        .collect::<Vec<_>>();
    assert_eq!(
        references,
        vec![
            format!("/// <reference path=\"{}\" />", parent.display()),
            format!("/// <reference path=\"{}\" />", child.display()),
            format!(
                "/// <reference path=\"{}\" />",
                root.join("node_modules/nitro-kit/index.d.ts").display()
            ),
        ]
    );
    assert_eq!(
        [&parent, &child, &shim].map(|path| std::fs::read(path).unwrap()),
        originals
    );
}

#[test]
fn snapshots_preserve_script_references_and_unresolved_type_directives() {
    let (root, program) = nuxt4_project("snapshot-script-reference");
    let script = write(
        &root,
        "workspace-globals.ts",
        "declare const fromReferencedScript: 41;\n",
    );
    let parent = root.join("node_modules/.pnpm/nitro-server/dist/augments.d.mts");
    let source = format!(
        "/// <reference path=\"shim.d.ts\" />\n\
         /// <reference path=\"{}\" preserve=\"true\" />\n\
         /// <reference types=\"missing-reference-type\" />\n\
         export {{}}; declare global {{ const safeParent: 42 }}\n",
        script.display()
    );
    std::fs::write(&parent, &source).unwrap();
    write(
        &root,
        "node_modules/.pnpm/nitro-server/dist/shim.d.ts",
        "declare module 'vue' { export const unrelated: true }\n",
    );
    let mut options = VirtualTsOptions::default();
    let detection = detect(
        &mut options,
        &root,
        &root,
        Some(&root.join("tsconfig.json")),
        Some(&program),
        false,
        VueVersion::V3,
    )
    .unwrap();
    let snapshot = detection
        .declaration_snapshots
        .iter()
        .find(|(path, _)| path == &parent)
        .unwrap();
    assert_eq!(
        snapshot.1.as_str(),
        format!(
            "/// <reference path=\"{}\" preserve=\"true\" />\n\
         /// <reference types=\"missing-reference-type\" />\n\
         export {{}}; declare global {{ const safeParent: 42 }}\n",
            script.display()
        )
    );
    assert_eq!(std::fs::read_to_string(&parent).unwrap(), source);
}

#[cfg(unix)]
#[test]
fn linked_type_package_stubs_keep_the_registered_snapshot_spelling() {
    let (root, program) = nuxt4_project("snapshot-linked-types");
    let installed = root.join("node_modules/nitro-kit");
    std::fs::write(installed.join("index.d.ts"), "/// <reference path=\"shim.d.ts\" />\nexport {}; declare global { const safeLinked: 42 }\n").unwrap();
    std::fs::write(
        installed.join("shim.d.ts"),
        "declare module 'vue' { export const unrelated: true }\n",
    )
    .unwrap();
    let real = root.join("workspace/nitro-kit");
    std::fs::create_dir_all(real.parent().unwrap()).unwrap();
    std::fs::rename(&installed, &real).unwrap();
    std::os::unix::fs::symlink(&real, &installed).unwrap();
    let mut options = VirtualTsOptions::default();
    let detection = detect(
        &mut options,
        &root,
        &root,
        Some(&root.join("tsconfig.json")),
        Some(&program),
        false,
        VueVersion::V3,
    )
    .unwrap();
    assert_eq!(
        detection
            .declaration_snapshots
            .iter()
            .map(|(path, _)| path.clone())
            .collect::<Vec<_>>(),
        vec![root.join(".nuxt/nuxt.d.ts"), installed.join("index.d.ts")]
    );
    let references = options
        .auto_import_stubs
        .iter()
        .filter(|stub| stub.starts_with("///"))
        .map(String::as_str)
        .collect::<Vec<_>>();
    assert_eq!(
        references,
        vec![
            format!(
                "/// <reference path=\"{}\" />",
                root.join("node_modules/.pnpm/nitro-server/dist/augments.d.mts")
                    .display()
            ),
            format!(
                "/// <reference path=\"{}\" />",
                installed.join("index.d.ts").display()
            )
        ]
    );
}
