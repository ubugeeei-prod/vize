use super::super::ScriptCompileContext;
use super::super::resolution::resolve_import_path;
use std::path::PathBuf;

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

    let mut ctx = ScriptCompileContext::new(source);
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
fn collects_type_reexports_from_vue_files() {
    let project = temp_project_dir("vue-type-reexport");
    let components = project.join("src/components");
    std::fs::create_dir_all(&components).unwrap();
    std::fs::write(
        components.join("Base.vue"),
        r#"<script lang="ts">
export interface BaseProps {
  as?: string;
  asChild?: boolean;
}
</script>"#,
    )
    .unwrap();
    std::fs::write(
        components.join("index.ts"),
        r#"export { type BaseProps } from "./Base.vue";"#,
    )
    .unwrap();

    let parent = components.join("Parent.vue");
    let source = r#"
import type { BaseProps } from "./index";

interface ParentProps extends BaseProps {}

const props = defineProps<ParentProps>();
"#;

    let mut ctx = ScriptCompileContext::new(source);
    ctx.collect_imported_types_from_path(source, parent.to_string_lossy().as_ref(), true);
    ctx.analyze();

    assert!(ctx.interfaces.contains_key("BaseProps"));
    assert_eq!(
        ctx.bindings.bindings.get("as"),
        Some(&crate::types::BindingType::Props)
    );
    assert_eq!(
        ctx.bindings.bindings.get("asChild"),
        Some(&crate::types::BindingType::Props)
    );

    let _ = std::fs::remove_dir_all(project);
}

#[test]
fn collects_mixed_type_reexports_from_vue_files() {
    let project = temp_project_dir("mixed-vue-type-reexport");
    let components = project.join("src/components");
    std::fs::create_dir_all(&components).unwrap();
    std::fs::write(
        components.join("Content.vue"),
        r#"<script lang="ts">
export interface ContentProps {
  as?: string;
  asChild?: boolean;
}
</script>"#,
    )
    .unwrap();
    std::fs::write(
        components.join("index.ts"),
        r#"export {
  default as Content,
  type ContentProps,
} from "./Content.vue";
"#,
    )
    .unwrap();

    let parent = components.join("Parent.vue");
    let source = r#"
import type { ContentProps } from "./index";

interface ParentProps extends ContentProps {}

const props = defineProps<ParentProps>();
"#;

    let mut ctx = ScriptCompileContext::new(source);
    ctx.collect_imported_types_from_path(source, parent.to_string_lossy().as_ref(), true);
    ctx.analyze();

    assert!(ctx.interfaces.contains_key("ContentProps"));
    assert_eq!(
        ctx.bindings.bindings.get("as"),
        Some(&crate::types::BindingType::Props)
    );
    assert_eq!(
        ctx.bindings.bindings.get("asChild"),
        Some(&crate::types::BindingType::Props)
    );

    let _ = std::fs::remove_dir_all(project);
}

#[test]
fn skips_type_import_collection_for_plain_js_scripts() {
    // Regression: the substring pre-check matches plain-JS object keys
    // like `type: 'text'` next to any `import`/`export`, which used to
    // fire the whole stat/realpath resolution walk for every generated
    // JS script. Non-TS blocks must skip collection entirely.
    let project = temp_project_dir("plain-js-gate");
    let components = project.join("src/components");
    std::fs::create_dir_all(&components).unwrap();
    std::fs::write(
        components.join("shared.ts"),
        "export interface InjectedProps { injected?: boolean }",
    )
    .unwrap();

    let current = components.join("Field.vue");
    let source = r#"
import { reactive } from 'vue'
export * from './shared'

const field = reactive({ type: 'text', name: 'email' })
"#;

    let mut ctx = ScriptCompileContext::new(source);
    ctx.collect_imported_types_from_path(source, current.to_string_lossy().as_ref(), false);
    assert!(ctx.interfaces.is_empty());
    assert!(ctx.type_aliases.is_empty());

    // Sanity: the same source *would* pull the interface in for a TS
    // block, so the assertions above genuinely exercise the gate.
    let mut ts_ctx = ScriptCompileContext::new(source);
    ts_ctx.collect_imported_types_from_path(source, current.to_string_lossy().as_ref(), true);
    assert!(ts_ctx.interfaces.contains_key("InjectedProps"));

    let _ = std::fs::remove_dir_all(project);
}

#[test]
fn collects_types_through_plain_star_reexport_barrel() {
    // Regression: a types barrel using plain `export * from './X.vue'`
    // (not `export type *`) still forwards every interface in TS, but the
    // collector skipped non-type re-exports entirely — nuxt-ui's Button
    // lost all `Omit<LinkProps, ...>` props this way.
    let project = temp_project_dir("plain-star-reexport");
    let components = project.join("src/components");
    let types = project.join("src/types");
    std::fs::create_dir_all(&components).unwrap();
    std::fs::create_dir_all(&types).unwrap();
    std::fs::write(
        components.join("Link.vue"),
        r#"<script lang="ts">
export interface LinkProps {
  disabled?: boolean;
  type?: string;
  raw?: boolean;
}
</script>"#,
    )
    .unwrap();
    std::fs::write(
        types.join("index.ts"),
        "export * from '../components/Link.vue'\n",
    )
    .unwrap();

    let parent = components.join("Button.vue");
    let source = r#"
import type { LinkProps } from "../types";

interface ButtonProps extends Omit<LinkProps, 'raw'> {
  label?: string;
}

const props = defineProps<ButtonProps>();
"#;

    let mut ctx = ScriptCompileContext::new(source);
    ctx.collect_imported_types_from_path(source, parent.to_string_lossy().as_ref(), true);
    ctx.analyze();

    assert!(ctx.interfaces.contains_key("LinkProps"));
    assert_eq!(
        ctx.bindings.bindings.get("disabled"),
        Some(&crate::types::BindingType::Props)
    );
    assert_eq!(
        ctx.bindings.bindings.get("type"),
        Some(&crate::types::BindingType::Props)
    );
    assert_eq!(ctx.bindings.bindings.get("raw"), None);

    let _ = std::fs::remove_dir_all(project);
}
