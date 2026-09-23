use super::resolution::{resolve_at_src_alias, resolve_import_path};
use std::path::{Path, PathBuf};

mod summaries;

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
