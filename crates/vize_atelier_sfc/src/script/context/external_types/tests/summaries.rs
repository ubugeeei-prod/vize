use super::super::build_file_summary;
use super::temp_project_dir;

#[test]
fn follows_plain_value_imports_only_in_declaration_summaries() {
    let project = temp_project_dir("declaration-value-import-scope");
    std::fs::create_dir_all(&project).unwrap();
    let source = "import './side-effect.js'\nimport { PrimitiveProps } from './chunk.js'\nexport { PrimitiveProps }\n";
    let module = project.join("index.ts");
    std::fs::write(&module, source).unwrap();

    for declaration_name in ["index.d.ts", "index.d.mts", "index.d.cts"] {
        let declaration = project.join(declaration_name);
        std::fs::write(&declaration, source).unwrap();
        let declaration_summary = build_file_summary(&declaration).unwrap();
        assert_eq!(
            declaration_summary.specifiers.as_slice(),
            ["./chunk.js"],
            "{declaration_name}"
        );
    }

    let module_summary = build_file_summary(&module).unwrap();
    assert!(
        module_summary.specifiers.is_empty(),
        "ordinary TS runtime imports must not widen the type graph"
    );

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

    let mut ctx = super::super::ScriptCompileContext::new(source);
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

    let mut ctx = super::super::ScriptCompileContext::new(source);
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

    let mut ctx = super::super::ScriptCompileContext::new(source);
    ctx.collect_imported_types_from_path(source, current.to_string_lossy().as_ref(), false);
    assert!(ctx.interfaces.is_empty());
    assert!(ctx.type_aliases.is_empty());

    // Sanity: the same source *would* pull the interface in for a TS
    // block, so the assertions above genuinely exercise the gate.
    let mut ts_ctx = super::super::ScriptCompileContext::new(source);
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

    let mut ctx = super::super::ScriptCompileContext::new(source);
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
