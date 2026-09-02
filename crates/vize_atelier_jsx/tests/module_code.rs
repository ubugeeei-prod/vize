//! Module emission for compiled JSX/TSX components (`module_code`).
//!
//! A module whose components are all block-body VDOM components is rebuilt as
//! `_defineComponent({ name, setup(…) { … } })` around the authored source;
//! everything else falls back to plain render exports.

use vize_atelier_jsx::{JsxCompileConfig, JsxLang, compile_jsx};
use vize_s0::Allocator;

#[test]
fn module_code_renames_multiple_render_exports_to_component_names() {
    let bump = Allocator::new();
    let out = compile_jsx(
        &bump,
        r#"
        export const InspectTable = () => <table>{rows}</table>;
        const LabelContent = () => <span>{label}</span>;
        export const InspectTableRow = () => <tr>{row}</tr>;
        "#,
        JsxLang::Tsx,
        &JsxCompileConfig::default(),
    );
    let module = out.module_code();

    assert!(module.contains("export function InspectTable("));
    assert!(module.contains("export function LabelContent("));
    assert!(module.contains("export function InspectTableRow("));
    assert!(!module.contains("export function render("));
}

#[test]
fn module_code_wraps_block_body_component_setup_state() {
    let bump = Allocator::new();
    let out = compile_jsx(
        &bump,
        r#"
        import { computed, ref } from "vue";

        export const App = () => {
          const count = ref(0);
          const doubled = computed(() => count.value * 2);
          const increment = () => {
            count.value += 1;
          };

          return (
            <section>
              <button onClick={increment}>Increment</button>
              <p>{count.value}</p>
              <p>{doubled.value}</p>
            </section>
          );
        };
        "#,
        JsxLang::Jsx,
        &JsxCompileConfig::default(),
    );
    assert!(
        out.components[0].component_setup().is_some(),
        "component setup metadata missing"
    );
    assert_eq!(out.components[0].component_name(), Some("App"));
    let module = out.module_code();

    assert!(
        module.contains("import { defineComponent as _defineComponent } from \"vue\""),
        "{module}"
    );
    assert!(module.contains("export const App = _defineComponent({"));
    assert!(module.contains("const count = ref(0);"));
    assert!(module.contains("const doubled = computed(() => count.value * 2);"));
    assert!(module.contains("count.value += 1;"));
    assert!(module.contains("function render(_ctx, _cache)"));
    assert!(module.contains("return render"));
    assert!(!module.contains("export function render("));
    assert!(out.source_map().is_none());
}

#[test]
fn tsx_component_keeps_destructured_props_signature_bindings() {
    let bump = Allocator::new();
    let out = compile_jsx(
        &bump,
        r#"
        import { ref } from "vue";

        type TsxCounterProps = {
          label?: string;
        };

        const TsxCounter = ({ label = "TSX" }: TsxCounterProps, _ctx: Ctx) => {
          const count = ref(0);
          const increment = () => count.value++;
          return (
            <button type="button" class="tsx-counter" onClick={increment}>
              {label}: {count.value}
            </button>
          );
        };

        export default TsxCounter;
        "#,
        JsxLang::Tsx,
        &JsxCompileConfig::default(),
    );
    let module = out.module_code();

    assert!(
        module.contains(r#"setup({ label = "TSX" }: TsxCounterProps, _ctx: Ctx) {"#),
        "{module}"
    );
    assert!(!module.contains("setup() {"), "{module}");
    insta::assert_snapshot!(module);
}

#[test]
fn tsx_generic_component_keeps_its_type_parameters_on_setup() {
    let bump = Allocator::new();
    let out = compile_jsx(
        &bump,
        r#"
        const GenericList = <T extends string,>({ items }: { items: Array<T> }) => {
          const first = items[0];
          return <ul>{first}</ul>;
        };

        export default GenericList;
        "#,
        JsxLang::Tsx,
        &JsxCompileConfig::default(),
    );
    let module = out.module_code();

    assert!(
        module.contains("setup<T extends string>({ items }: { items: Array<T> }) {"),
        "{module}"
    );
}

#[test]
fn async_component_keeps_its_async_modifier_on_setup() {
    let bump = Allocator::new();
    let out = compile_jsx(
        &bump,
        r#"
        const AsyncPanel = async ({ id }) => {
          const data = await load(id);
          return <div>{data}</div>;
        };
        "#,
        JsxLang::Jsx,
        &JsxCompileConfig::default(),
    );
    let module = out.module_code();

    // Dropping `async` would leave `await` inside a synchronous method.
    assert!(module.contains("async setup({ id }) {"), "{module}");
    assert!(module.contains("const data = await load(id);"), "{module}");
}

#[test]
fn module_code_forwards_plain_props_and_context_parameters_to_setup() {
    let bump = Allocator::new();
    let out = compile_jsx(
        &bump,
        r#"
        const Greeting = (props, { emit }) => {
          const shout = () => emit("shout");
          return <button onClick={shout}>{props.label}</button>;
        };
        "#,
        JsxLang::Jsx,
        &JsxCompileConfig::default(),
    );
    let module = out.module_code();

    assert!(module.contains("setup(props, { emit }) {"), "{module}");
}

#[test]
fn module_code_forwards_rest_parameters_to_setup() {
    let bump = Allocator::new();
    let out = compile_jsx(
        &bump,
        r#"
        const Spread = (props, ...rest) => {
          const first = rest[0];
          return <div>{first}</div>;
        };
        "#,
        JsxLang::Jsx,
        &JsxCompileConfig::default(),
    );
    let module = out.module_code();

    assert!(module.contains("setup(props, ...rest) {"), "{module}");
}

#[test]
fn module_code_keeps_an_empty_setup_signature_for_parameterless_components() {
    let bump = Allocator::new();
    let out = compile_jsx(
        &bump,
        r#"
        const Standalone = () => {
          const label = "hi";
          return <div>{label}</div>;
        };
        "#,
        JsxLang::Jsx,
        &JsxCompileConfig::default(),
    );
    let module = out.module_code();

    assert!(module.contains("setup() {"), "{module}");
}

#[test]
fn module_code_forwards_each_components_own_parameters() {
    let bump = Allocator::new();
    let out = compile_jsx(
        &bump,
        r#"
        const First = ({ a }) => {
          const one = a;
          return <p>{one}</p>;
        };
        const Second = ({ b }) => {
          const two = b;
          return <p>{two}</p>;
        };
        "#,
        JsxLang::Jsx,
        &JsxCompileConfig::default(),
    );
    let module = out.module_code();

    assert!(module.contains("setup({ a }) {"), "{module}");
    assert!(module.contains("setup({ b }) {"), "{module}");
}

#[test]
fn jsx_in_a_parameter_default_falls_back_to_plain_render_exports() {
    let bump = Allocator::new();
    let out = compile_jsx(
        &bump,
        r#"
        const Fallback = () => <i/>;
        const App = ({ slot = <Fallback /> }) => {
          const node = slot;
          return <div>{node}</div>;
        };
        "#,
        JsxLang::Jsx,
        &JsxCompileConfig::default(),
    );
    let module = out.module_code();

    // The default's JSX is its own render root without setup metadata, so the
    // stateful wrapper is skipped and no raw JSX can leak into `setup(...)`.
    assert!(!module.contains("_defineComponent"), "{module}");
    assert!(!module.contains("setup("), "{module}");
}

#[test]
fn module_code_leaves_synchronous_components_without_an_async_setup() {
    let bump = Allocator::new();
    let out = compile_jsx(
        &bump,
        r#"
        const SyncApp = () => {
          const data = load();
          return <div>{data}</div>;
        };
        "#,
        JsxLang::Jsx,
        &JsxCompileConfig::default(),
    );
    let module = out.module_code();

    assert!(module.contains("  setup() {"), "{module}");
    assert!(!module.contains("async setup"), "{module}");
}
