//! Prop declarations on the generated `_defineComponent` wrapper (#3861).
//!
//! Vue only fills `setup`'s first argument with *declared* props, so the names a
//! component destructures have to reach the wrapper's `props` option. The list
//! is derived from the destructuring pattern alone, and is emitted only when
//! every name is statically known — a partial list would look authoritative
//! while silently routing the rest to `attrs`.

use vize_atelier_jsx::{JsxCompileConfig, JsxLang, compile_jsx};
use vize_s0::Allocator;

fn module_code(source: &str, lang: JsxLang) -> std::string::String {
    let bump = Allocator::new();
    let out = compile_jsx(&bump, source, lang, &JsxCompileConfig::default());
    assert!(!out.has_errors(), "diagnostics: {:?}", out.diagnostics);
    out.module_code().as_str().to_string()
}

#[test]
fn destructured_props_are_declared_so_caller_values_reach_the_binding() {
    let module = module_code(
        r#"
        const Counter = ({ label = "TSX", step }) => {
          const next = step;
          return <button>{label}{next}</button>;
        };
        "#,
        JsxLang::Jsx,
    );

    assert!(module.contains(r#"props: ["label", "step"],"#), "{module}");
    assert!(
        module.contains(r#"setup({ label = "TSX", step }) {"#),
        "the default stays in the pattern rather than moving into props: {module}"
    );
}

#[test]
fn a_renamed_binding_declares_the_prop_name_not_the_local_name() {
    let module = module_code(
        r#"
        const Renamed = ({ label: text }) => {
          const shown = text;
          return <span>{shown}</span>;
        };
        "#,
        JsxLang::Jsx,
    );

    assert!(module.contains(r#"props: ["label"],"#), "{module}");
    assert!(!module.contains(r#""text""#), "{module}");
}

#[test]
fn a_rest_element_declares_nothing_rather_than_a_partial_list() {
    let module = module_code(
        r#"
        const Spread = ({ label, ...rest }) => {
          const extra = rest;
          return <div>{label}{extra}</div>;
        };
        "#,
        JsxLang::Jsx,
    );

    assert!(module.contains("setup({ label, ...rest }) {"), "{module}");
    assert!(!module.contains("props:"), "{module}");
}

#[test]
fn a_computed_key_declares_nothing_rather_than_a_partial_list() {
    let module = module_code(
        r#"
        const Computed = ({ label, [dynamic]: value }) => {
          const shown = value;
          return <div>{label}{shown}</div>;
        };
        "#,
        JsxLang::Jsx,
    );

    assert!(!module.contains("props:"), "{module}");
}

#[test]
fn a_plain_props_parameter_declares_nothing() {
    let module = module_code(
        r#"
        const Plain = (props) => {
          const shown = props.label;
          return <div>{shown}</div>;
        };
        "#,
        JsxLang::Jsx,
    );

    assert!(module.contains("setup(props) {"), "{module}");
    assert!(!module.contains("props:"), "{module}");
}

#[test]
fn a_parameterless_component_declares_nothing() {
    let module = module_code(
        r#"
        const Standalone = () => {
          const label = "hi";
          return <div>{label}</div>;
        };
        "#,
        JsxLang::Jsx,
    );

    assert!(module.contains("setup() {"), "{module}");
    assert!(!module.contains("props:"), "{module}");
}

#[test]
fn only_the_first_parameter_contributes_prop_names() {
    let module = module_code(
        r#"
        const WithContext = ({ label }, { emit }) => {
          const shout = () => emit("shout");
          return <button onClick={shout}>{label}</button>;
        };
        "#,
        JsxLang::Jsx,
    );

    assert!(module.contains(r#"props: ["label"],"#), "{module}");
    assert!(!module.contains(r#""emit""#), "{module}");
}

#[test]
fn typed_tsx_components_declare_the_destructured_names() {
    let module = module_code(
        r#"
        type Props = { label?: string };

        const Typed = ({ label = "TSX" }: Props) => {
          const shown = label;
          return <div>{shown}</div>;
        };
        "#,
        JsxLang::Tsx,
    );

    assert!(module.contains(r#"props: ["label"],"#), "{module}");
}
