//! Tests for [`super::compile`].
//!
//! Split out of that module so it stays inside the per-file source-length
//! budget.
#![expect(clippy::disallowed_macros, reason = "insta and fixtures use format!")]

use super::*;

#[test]
fn typed_slot_runtime_output_and_metadata_remain_byte_exact() {
    let source = "// 日本語 😀\r\nconst View = () => <Host>{{ named: (props: { value: string }) => <span>{props.value}</span>, other: function ({ value }: { value: number }) { return <b>{value}</b>; } }}</Host>;\r\nconst Styled = () => <div><style scoped>{`.view { color: red; }`}</style><p class=\"view\">ok</p></div>;";
    for mode in [JsxOutputMode::Vdom, JsxOutputMode::Vapor] {
        let bump = Allocator::new();
        let mut config = JsxCompileConfig::default();
        config.default_mode = mode;
        config.vdom.source_map = true;
        let output = compile_jsx(&bump, source, JsxLang::Tsx, &config);
        assert_eq!(output.components.len(), 2);
        assert!(output.diagnostics.is_empty());
        let components: Vec<_> = output
            .components
            .iter()
            .map(|component| {
                let templates = match component {
                    JsxComponent::Vapor(component) => component.templates.as_slice(),
                    _ => &[],
                };
                (
                    component.component_name(),
                    component.component_setup(),
                    component.mode(),
                    component.code(),
                    component.preamble(),
                    component.map(),
                    component.scoped_style(),
                    templates,
                )
            })
            .collect();
        let packet = (
            output.module_code(),
            output.source_map(),
            components,
            &output.diagnostics,
        );
        match mode {
            JsxOutputMode::Vdom => insta::assert_debug_snapshot!("typed_slot_runtime_vdom", packet),
            JsxOutputMode::Vapor => {
                insta::assert_debug_snapshot!("typed_slot_runtime_vapor", packet)
            }
        }
    }
}

#[test]
fn module_code_prepends_merged_preamble_to_render_code() {
    // The authored VDOM declaration follows its deduplicated runtime imports.
    let bump = Allocator::new();
    let out = compile_jsx(
        &bump,
        "const A = () => <div>{x}</div>;",
        JsxLang::Jsx,
        &JsxCompileConfig::default(),
    );
    let module = out.module_code();
    insta::assert_snapshot!(module);
}

#[test]
fn source_map_covers_single_and_multiple_component_modules() {
    let bump = Allocator::new();
    let mut config = JsxCompileConfig::default();
    config.vdom.source_map = true;

    let single = compile_jsx(
        &bump,
        "const A = () => <div>{x}</div>;",
        JsxLang::Jsx,
        &config,
    );
    assert_eq!(single.components.len(), 1);
    let map = single.source_map().expect("single component carries a map");
    insta::assert_snapshot!(map);

    let multi = compile_jsx(
        &bump,
        "const A = () => <div>{x}</div>;\nconst B = () => <span>{y}</span>;",
        JsxLang::Jsx,
        &config,
    );
    assert!(multi.components.len() >= 2);
    assert!(
        multi.source_map().is_some(),
        "multi-component module composes maps for every retained declaration"
    );
}
