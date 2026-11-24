use super::resolution::{resolve_at_src_alias, resolve_import_path};
use std::path::{Path, PathBuf};

mod differential;
mod reference;
mod sharing;
mod summaries;

/// A batch generation no open batch has: live generations count up from one,
/// these count down from the top. Tests pass them explicitly instead of
/// opening a process-wide batch, which every other test would observe.
fn synthetic_epoch() -> u64 {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(u64::MAX);
    NEXT.fetch_sub(1, std::sync::atomic::Ordering::Relaxed)
}

fn temp_project_dir(test_name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "vize-sfc-external-types-{}-{}-{}",
        std::process::id(),
        test_name,
        nonce
    ))
}

#[test]
fn resolves_at_alias_from_nearest_src_directory() {
    let project = temp_project_dir("at-alias");
    let components = project.join("packages/frontend/src/components");
    std::fs::create_dir_all(&components).unwrap();
    let target = components.join("Base.vue");
    std::fs::write(&target, "").unwrap();

    let current = components.join("Child.vue");
    let resolved = resolve_at_src_alias(&current, "@/components/Base.vue");
    let target = target.canonicalize().unwrap();

    assert_eq!(resolved.as_deref(), Some(target.as_path()));

    let _ = std::fs::remove_dir_all(project);
}

#[test]
fn ignores_at_alias_without_src_ancestor() {
    let current = Path::new("/repo/packages/frontend/components/Child.vue");

    assert!(resolve_at_src_alias(current, "@/components/Base.vue").is_none());
}

#[test]
fn leaves_non_at_alias_specifiers_to_existing_resolution() {
    let current = Path::new("/repo/src/components/Child.vue");

    assert!(resolve_import_path(current, "vue").is_none());
}

#[test]
fn resolves_package_import_patterns_from_the_nearest_scope() {
    let project = temp_project_dir("package-import-pattern");
    let src = project.join("src/components");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(
        project.join("package.json"),
        r##"{ "imports": { "#lib/*": { "types": "./src/*.ts", "default": "./dist/*.js" } } }"##,
    )
    .unwrap();
    let target = project.join("src/types.ts");
    std::fs::write(&target, "export type Ripple = boolean | object").unwrap();

    let current = src.join("Button.vue");
    let resolved = resolve_import_path(&current, "#lib/types");

    assert_eq!(
        resolved.as_deref(),
        Some(target.canonicalize().unwrap().as_path())
    );
    let _ = std::fs::remove_dir_all(project);
}

#[test]
fn resolves_bare_specifier_through_node_modules_types_field() {
    let project = temp_project_dir("bare-types-field");
    let package = project.join("node_modules/some-ui");
    std::fs::create_dir_all(package.join("dist")).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{ "name": "some-ui", "types": "./dist/index.d.ts" }"#,
    )
    .unwrap();
    std::fs::write(
        package.join("dist/index.d.ts"),
        "export interface RootProps { autocomplete?: string }",
    )
    .unwrap();
    let components = project.join("src/components");
    std::fs::create_dir_all(&components).unwrap();

    let current = components.join("Select.vue");
    let resolved = resolve_import_path(&current, "some-ui");
    let target = package.join("dist/index.d.ts").canonicalize().unwrap();
    assert_eq!(resolved.as_deref(), Some(target.as_path()));

    let _ = std::fs::remove_dir_all(project);
}

#[test]
fn resolves_scoped_bare_specifier_through_exports_types() {
    let project = temp_project_dir("bare-exports-types");
    let package = project.join("node_modules/@scope/pkg");
    std::fs::create_dir_all(package.join("dist")).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{ "name": "@scope/pkg", "exports": { ".": { "import": { "types": "./dist/main.d.mts", "default": "./dist/main.mjs" } } } }"#,
    )
    .unwrap();
    std::fs::write(package.join("dist/main.d.mts"), "export type T = string").unwrap();
    let src = project.join("src");
    std::fs::create_dir_all(&src).unwrap();

    let current = src.join("App.vue");
    let resolved = resolve_import_path(&current, "@scope/pkg");
    let target = package.join("dist/main.d.mts").canonicalize().unwrap();
    assert_eq!(resolved.as_deref(), Some(target.as_path()));

    let _ = std::fs::remove_dir_all(project);
}

#[test]
fn does_not_follow_bare_specifiers_from_inside_node_modules() {
    let project = temp_project_dir("bare-from-node-modules");
    let nested = project.join("node_modules/vue");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(
        nested.join("package.json"),
        r#"{ "types": "./index.d.ts" }"#,
    )
    .unwrap();
    std::fs::write(nested.join("index.d.ts"), "export type X = 1").unwrap();

    let current = project.join("node_modules/some-ui/dist/index.d.ts");
    assert!(resolve_import_path(&current, "vue").is_none());

    let _ = std::fs::remove_dir_all(project);
}

#[test]
fn collects_props_from_node_modules_package_types() {
    let project = temp_project_dir("bare-props-collection");
    let package = project.join("node_modules/some-ui");
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{ "name": "some-ui", "types": "./index.d.ts" }"#,
    )
    .unwrap();
    std::fs::write(
        package.join("index.d.ts"),
        "interface RootProps { autocomplete?: string; dir?: string }\nexport { RootProps }",
    )
    .unwrap();
    let components = project.join("src/components");
    std::fs::create_dir_all(&components).unwrap();

    let current = components.join("Select.vue");
    let source = r#"
import type { RootProps } from "some-ui";

interface SelectProps extends Omit<RootProps, 'dir'> {
  label?: string;
}

const props = defineProps<SelectProps>();
"#;

    let mut ctx = super::ScriptCompileContext::new(source);
    ctx.collect_imported_types_from_path(source, current.to_string_lossy().as_ref(), true);
    ctx.analyze();

    assert!(ctx.interfaces.contains_key("RootProps"));
    assert_eq!(
        ctx.bindings.bindings.get("autocomplete"),
        Some(&crate::types::BindingType::Props)
    );
    assert_eq!(ctx.bindings.bindings.get("dir"), None);

    let _ = std::fs::remove_dir_all(project);
}

#[test]
fn resolves_js_type_specifiers_to_ts_sources() {
    let project = temp_project_dir("js-to-ts-type-import");
    let utility = project.join("src/utility");
    let components = project.join("src/components");
    std::fs::create_dir_all(&utility).unwrap();
    std::fs::create_dir_all(&components).unwrap();
    let target = utility.join("paginator.ts");
    std::fs::write(
        &target,
        "export type ExtractorFunction<T> = (item: T) => T;",
    )
    .unwrap();

    let current = components.join("UserList.vue");
    let resolved = resolve_import_path(&current, "@/utility/paginator.js");
    let target = target.canonicalize().unwrap();

    assert_eq!(resolved.as_deref(), Some(target.as_path()));

    let _ = std::fs::remove_dir_all(project);
}

#[test]
fn resolves_javascript_specifiers_to_declaration_sources() {
    let project = temp_project_dir("javascript-to-declaration-imports");
    let dist = project.join("dist");
    std::fs::create_dir_all(&dist).unwrap();
    let current = dist.join("index.d.ts");

    for (specifier, declaration) in [
        ("./index4.js", "index4.d.ts"),
        ("./index4.jsx", "index4.d.ts"),
        ("./index4.mjs", "index4.d.mts"),
        ("./index4.cjs", "index4.d.cts"),
        ("./index4.vue", "index4.vue.d.ts"),
    ] {
        std::fs::write(dist.join(&specifier[2..]), "export {};\n").unwrap();
        let target = dist.join(declaration);
        std::fs::write(&target, "export interface PrimitiveProps {}").unwrap();

        let resolved = resolve_import_path(&current, specifier);
        let target = target.canonicalize().unwrap();
        assert_eq!(resolved.as_deref(), Some(target.as_path()), "{specifier}");
    }

    let _ = std::fs::remove_dir_all(project);
}

#[test]
fn package_barrels_resolve_props_from_declarations_beside_compiled_vue_files() {
    let project = tempfile::tempdir().unwrap();
    let package = project.path().join("node_modules/component-library");
    let components = package.join("dist/components");
    std::fs::create_dir_all(&components).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"exports":{".":{"types":"./dist/module.d.mts"}}}"#,
    )
    .unwrap();
    std::fs::write(
        package.join("dist/module.d.mts"),
        "export * from './index.js';",
    )
    .unwrap();
    std::fs::write(
        package.join("dist/index.d.ts"),
        "export * from './components/Navbar.vue';",
    )
    .unwrap();
    std::fs::write(
        components.join("Navbar.vue"),
        "<script setup>defineProps({ title: String })</script><template><div /></template>",
    )
    .unwrap();
    let declaration = components.join("Navbar.vue.d.ts");
    std::fs::write(
        &declaration,
        "export interface NavbarProps { title?: string; toggle?: boolean }",
    )
    .unwrap();
    let filename = project.path().join("App.vue");
    let script = "import type { NavbarProps } from 'component-library'; defineProps<NavbarProps>()";
    let mut ctx = crate::script::ScriptCompileContext::new(script);
    ctx.collect_imported_types_from_path(script, filename.to_str().unwrap(), true);
    ctx.analyze();
    assert_eq!(
        ctx.resolve_type_props("NavbarProps")
            .iter()
            .map(|prop| prop.name.as_str())
            .collect::<Vec<_>>(),
        vec!["title", "toggle"],
    );
}

#[test]
fn compiles_the_pinned_vue_declaration_project() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/compiler/vue-declaration-props-13577");
    let project = tempfile::tempdir().unwrap();
    for (input, output) in [
        ("Layout.vue.txt", "Layout.vue"),
        (
            "lib.package.json.txt",
            "node_modules/component-library/package.json",
        ),
        (
            "lib.module.d.mts.txt",
            "node_modules/component-library/dist/module.d.mts",
        ),
        (
            "lib.index.d.ts.txt",
            "node_modules/component-library/dist/index.d.ts",
        ),
        (
            "lib.Navbar.vue.txt",
            "node_modules/component-library/dist/components/Navbar.vue",
        ),
        (
            "lib.Navbar.vue.d.ts.txt",
            "node_modules/component-library/dist/components/Navbar.vue.d.ts",
        ),
    ] {
        let target = project.path().join(output);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::copy(fixture.join(input), target).unwrap();
    }
    let filename = project.path().join("Layout.vue");
    let source = std::fs::read_to_string(&filename).unwrap();
    let mut options = crate::SfcCompileOptions::default();
    options.parse.filename = filename.to_str().unwrap().into();
    options.template.ssr = true;
    let descriptor = crate::parse_sfc(&source, options.parse.clone()).unwrap();
    let result = crate::compile_sfc(&descriptor, options).unwrap();
    assert_eq!(
        serde_json::to_value(&result.errors).unwrap(),
        serde_json::json!([])
    );
    assert_eq!(
        serde_json::to_value(&result.warnings).unwrap(),
        serde_json::json!([])
    );
    assert_eq!(
        result.code.as_str(),
        std::fs::read_to_string(fixture.join("module.expected.txt")).unwrap(),
    );
}

#[test]
fn local_props_alias_wins_over_an_imported_interface_of_the_same_name() {
    let project = temp_project_dir("local-props-shadow");
    std::fs::create_dir_all(&project).unwrap();
    std::fs::write(
        project.join("features.ts"),
        "interface Props { label: string }\nexport type Features = { bold?: boolean; props?: Props }\n",
    )
    .unwrap();
    let filename = project.join("App.vue");
    let script = "import type { Features } from './features';\ntype Props = { msg?: string; features?: Features }\nwithDefaults(defineProps<Props>(), { msg: 'hello' })\n";
    let mut ctx = crate::script::ScriptCompileContext::new(script);
    ctx.collect_imported_types_from_path(script, filename.to_str().unwrap(), true);
    ctx.analyze();
    let resolved = ctx.resolve_type_props("Props");
    let names: Vec<_> = resolved.iter().map(|prop| prop.name.as_str()).collect();
    assert_eq!(names, vec!["msg", "features"]);
    let _ = std::fs::remove_dir_all(project);
}

#[test]
fn normal_script_props_alias_wins_over_a_later_imported_interface() {
    let project = temp_project_dir("normal-props-shadow");
    std::fs::create_dir_all(&project).unwrap();
    std::fs::write(
        project.join("features.ts"),
        "interface Props { label: string }\nexport type Features = { bold?: boolean }\n",
    )
    .unwrap();
    let filename = project.join("App.vue");
    let setup = "import type { Features } from './features';\ndefineProps<Props>()\n";
    let mut ctx = crate::script::ScriptCompileContext::new(setup);
    ctx.collect_types_from("type Props = { msg?: string; features?: Features }\n");
    ctx.collect_imported_types_from_path(setup, filename.to_str().unwrap(), true);
    ctx.analyze();
    let resolved = ctx.resolve_type_props("Props");
    let names: Vec<_> = resolved.iter().map(|prop| prop.name.as_str()).collect();
    assert_eq!(names, vec!["msg", "features"]);
    let _ = std::fs::remove_dir_all(project);
}
