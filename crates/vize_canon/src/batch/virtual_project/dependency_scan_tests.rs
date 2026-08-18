//! Tests for the reachability registration pass (#3887).

use crate::batch::virtual_project::dependency_scan::*;
use std::fs;

fn case_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("vize-tests")
        .join("dependency-scan")
        .join(cstr!("{name}-{}", std::process::id()).as_str());
    let _ = fs::remove_dir_all(&dir);
    dir
}

#[test]
fn a_published_package_alias_does_not_force_a_parse() {
    // #3898: the `vue` alias every Vue tsconfig carries is wildcard-free and
    // points into `node_modules`, so the walk always rejects it. Matching it
    // as a bare substring made `may_resolve_a_dependency` true for every
    // generated file — each one contains "vue" — and reintroduced the
    // per-file parse the prefilter exists to avoid.
    //
    // The layout matters: `vue` declares `dist/vue.d.ts` through `types` and
    // ships no root `index.d.ts`, so probing the package directory finds
    // nothing. An earlier fix kept the prefix on probe failure and left the
    // benchmark regression in place.
    let root = case_dir("published-alias");
    let package = root.join("node_modules").join("vue");
    fs::create_dir_all(package.join("dist")).unwrap();
    fs::write(
        package.join("package.json"),
        "{ \"types\": \"dist/vue.d.ts\" }\n",
    )
    .unwrap();
    fs::write(package.join("dist").join("vue.d.ts"), "export {};\n").unwrap();

    assert!(!alias_may_reach_first_party(
        "vue",
        "./node_modules/vue",
        &root
    ));
    assert!(!may_resolve_a_dependency(
        "import { ref } from 'vue'\n",
        &[],
        &[],
    ));
    assert!(!may_resolve_a_dependency(
        "import { ref } from 'vue'\n",
        &[],
        &[CompactString::from("@scope/ui/widget")],
    ));
    assert!(may_resolve_a_dependency(
        "import Widget from '@scope/ui/widget'\n",
        &[],
        &[CompactString::from("@scope/ui/widget")],
    ));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn a_package_resolver_prefilters_only_approved_workspace_specifiers() {
    // #4000: the editor walk first resolves the host's package imports, then
    // admits only those exact workspace specifiers. Published packages and
    // unrelated bare imports must not reinstate the per-file parse #3898
    // removed.
    assert!(!may_resolve_a_dependency(
        "export const count = 1\nexport type Ref = { value: number }\n",
        &[],
        &[CompactString::from("@scope/ui/widget")],
    ));
    assert!(may_resolve_a_dependency(
        "import Widget from '@scope/ui/widget'\n",
        &[],
        &[CompactString::from("@scope/ui/widget")],
    ));
    assert!(!may_resolve_a_dependency(
        "import Widget from '@scope/ui/widget'\n",
        &[],
        &[],
    ));
    assert!(may_resolve_a_dependency(
        "const mod = await import(\"@scope/ui/widget\")\n",
        &[],
        &[CompactString::from("@scope/ui/widget")],
    ));
    assert!(may_resolve_a_dependency(
        "export { Widget } from '#internal/widget'\n",
        &[],
        &[CompactString::from("#internal/widget")],
    ));
}

#[test]
fn a_published_package_alias_with_a_declaration_barrel_does_not_force_a_parse() {
    // The other published layout: a root `index.d.ts` does probe, and the
    // walk still refuses it for being a declaration file inside
    // `node_modules`, so the prefix is just as droppable.
    let root = case_dir("published-alias-barrel");
    let package = root.join("node_modules").join("vue");
    fs::create_dir_all(&package).unwrap();
    fs::write(package.join("index.d.ts"), "export declare const a: 1;\n").unwrap();

    assert!(!alias_may_reach_first_party(
        "vue",
        "./node_modules/vue",
        &root
    ));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn an_unresolvable_first_party_alias_does_not_force_a_parse() {
    // Nothing to probe means the walk resolves nothing from this alias, so
    // its prefix must not drag every module through a parse either.
    let root = case_dir("missing-alias");
    fs::create_dir_all(&root).unwrap();
    assert!(!alias_may_reach_first_party("@ui", "./packages/ui", &root));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn dependency_probe_supports_javascript_and_does_not_append_to_explicit_extensions() {
    let root = case_dir("javascript-probe");
    std::fs::create_dir_all(&root).unwrap();
    let javascript = root.join("entry.mjs");
    std::fs::write(&javascript, "export {};\n").unwrap();
    std::fs::write(root.join("missing.vue.ts"), "export {};\n").unwrap();

    assert_eq!(probe_candidates(&javascript), Some(javascript));
    assert_eq!(probe_candidates(&root.join("missing.vue")), None);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_reachable_declaration_file_is_not_registered() {
    // #3898: `ecosystem-products` keeps its ambient shims in a script
    // `src/shims.d.ts` that every SFC pulls in with `import type {} from
    // "./shims"`. Registering it made its `declare module "vue"` an ambient
    // module declaration in the generated program, replacing Vue's real
    // typings, and every `import { computed, ref, watch } from "vue"`
    // collapsed into `TS2305`. Sibling implementation modules must still
    // register.
    let root = case_dir("declaration-file");
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        src.join("shims.d.ts"),
        "declare module \"vue\" {\n  export interface GlobalComponents {}\n}\n",
    )
    .unwrap();
    fs::write(
        src.join("types.ts"),
        "export type Product = { id: string };\n",
    )
    .unwrap();
    let vue_path = src.join("App.vue");
    fs::write(
            &vue_path,
            "<script setup lang=\"ts\">\nimport type {} from \"./shims\";\nimport type { Product } from \"./types\";\nconst product: Product = { id: \"a\" };\n</script>\n",
        )
        .unwrap();

    let mut project = VirtualProject::new(&root).unwrap();
    project.register_path(&vue_path).unwrap();
    project.register_reachable_dependencies().unwrap();

    let registered: Vec<CompactString> = project
        .registered_original_paths_sorted()
        .iter()
        .filter_map(|path| path.file_name()?.to_str().map(CompactString::from))
        .collect();
    assert!(
        !registered.iter().any(|name| name == "shims.d.ts"),
        "{registered:?}"
    );
    // An in-root script stays out too: real-tree resolution already serves
    // its types, the scan collector owns in-root discovery, and registering
    // it would grow the scanned set that incremental sessions and the Tier-L
    // gate pin to an exact count.
    assert!(
        !registered.iter().any(|name| name == "types.ts"),
        "{registered:?}"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn an_editor_paths_declaration_is_mirrored_but_not_a_program_root() {
    let root = case_dir("editor-paths-declaration");
    let src = root.join("src");
    fs::create_dir_all(src.join("api")).unwrap();
    fs::write(
        root.join("tsconfig.json"),
        r#"{"compilerOptions":{"paths":{"@/*":["./src/*"]}}}"#,
    )
    .unwrap();
    let declaration = src.join("api/remote-search.d.ts");
    fs::write(&declaration, "export declare const search: () => void;\n").unwrap();
    let vue_path = src.join("App.vue");
    fs::write(
        &vue_path,
        "<script setup lang=\"ts\">import { search } from '@/api/remote-search'; search()</script>\n",
    )
    .unwrap();

    let mut project = VirtualProject::new(&root).unwrap();
    project.set_session_script_registration(true);
    project.set_declaration_roots(std::slice::from_ref(&vue_path));
    project.register_path(&vue_path).unwrap();
    project.register_reachable_dependencies().unwrap();

    let registered = project.registered_original_paths_sorted();
    assert!(registered.contains(&declaration), "{registered:?}");
    assert!(!project.is_declaration_root(&declaration));
    assert_eq!(
        project
            .find_by_original(&declaration)
            .unwrap()
            .virtual_path
            .file_name()
            .unwrap(),
        "remote-search.d.ts"
    );
    let _ = fs::remove_dir_all(&root);
}

/// The batch mirror renames a declaration file to `.d.cts` only when the file
/// uses a CommonJS-only form the virtual root's `"type": "module"` boundary
/// rejects. Editor mirrors keep every declaration's spelling, and that
/// difference is what this pins.
///
/// The spelling is load-bearing: a `.d.cts` resolves its specifiers under the
/// `require` condition, so a `declare module "vue"` inside one binds to a
/// different `vue` than the generated `.vue.ts` modules import, and the
/// `ComponentCustomProperties` members it declares stop reaching the template
/// context.
#[test]
fn batch_declaration_spelling_stays_isolated_from_editor_mirrors() {
    let root = case_dir("batch-declaration-spelling");
    fs::create_dir_all(root.join("src")).unwrap();
    let plain = root.join("src/globals.d.ts");
    let plain_source = "declare const batchOnly: string;\n";
    fs::write(&plain, plain_source).unwrap();
    let commonjs = root.join("src/legacy.d.ts");
    let commonjs_source = "declare function legacy(): void;\nexport = legacy;\n";
    fs::write(&commonjs, commonjs_source).unwrap();

    let mut project = VirtualProject::new(&root).unwrap();
    project
        .register_declaration_file(&plain, plain_source)
        .unwrap();
    project
        .register_declaration_file(&commonjs, commonjs_source)
        .unwrap();
    assert_eq!(
        project
            .find_by_original(&plain)
            .unwrap()
            .virtual_path
            .file_name()
            .unwrap(),
        "globals.d.ts"
    );
    assert_eq!(
        project
            .find_by_original(&commonjs)
            .unwrap()
            .virtual_path
            .file_name()
            .unwrap(),
        "legacy.d.cts"
    );

    let mut editor = VirtualProject::new(&root).unwrap();
    editor.set_session_script_registration(true);
    editor
        .register_declaration_file(&commonjs, commonjs_source)
        .unwrap();
    assert_eq!(
        editor
            .find_by_original(&commonjs)
            .unwrap()
            .virtual_path
            .file_name()
            .unwrap(),
        "legacy.d.ts"
    );
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn external_esm_declaration_preserves_import_condition_spelling() {
    let root = case_dir("external-esm-declaration-spelling");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("package.json"), r#"{"type":"module"}"#).unwrap();
    let declaration = root.join("src/globals.d.ts");
    let content = "import 'dual-package';\ndeclare module 'dual-package/subpath.js' {}\n";
    fs::write(&declaration, content).unwrap();
    let mut project = VirtualProject::new(&root).unwrap();
    project
        .register_declaration_file(&declaration, content)
        .unwrap();
    let virtual_path = &project.find_by_original(&declaration).unwrap().virtual_path;
    assert_eq!(virtual_path.file_name().unwrap(), "globals.d.ts");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn a_wildcard_alias_into_node_modules_is_kept() {
    // A pnpm workspace link lives under `node_modules/<scope>/<pkg>`, so a
    // wildcard target's entries can each canonicalize out and be first
    // party. Only the wildcard-free shape is provably rejectable.
    let root = case_dir("wildcard-alias");
    fs::create_dir_all(root.join("node_modules").join("@scope")).unwrap();
    assert!(alias_may_reach_first_party(
        "@scope/*",
        "./node_modules/@scope/*",
        &root
    ));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn a_first_party_alias_is_kept() {
    let root = case_dir("first-party-alias");
    fs::create_dir_all(root.join("packages").join("ui")).unwrap();
    fs::write(root.join("packages").join("ui").join("index.ts"), "\n").unwrap();
    assert!(alias_may_reach_first_party("@ui", "./packages/ui", &root));
    let _ = fs::remove_dir_all(&root);
}
