//! Complete normal-runtime packets frozen before typecheck attribute repair.
use super::*;

#[test]
fn authored_attribute_runtime_output_and_metadata_remain_byte_exact() {
    let source = "// 日本語 😀\r\nconst View = () => <Host separator={choice || <span>{value}</span>} customSlots={{ checkable: () => <i>{count}</i> }} {...{ extra: <b>{value}</b> }} icon=<u>{count}</u> />;";
    for mode in [JsxOutputMode::Vdom, JsxOutputMode::Vapor] {
        let bump = Allocator::new();
        let mut config = JsxCompileConfig {
            default_mode: mode,
            ..Default::default()
        };
        config.vdom.source_map = true;
        let output = compile_jsx(&bump, source, JsxLang::Tsx, &config);
        assert_eq!(output.components.len(), 1);
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
            JsxOutputMode::Vdom => insta::assert_debug_snapshot!("attribute_runtime_vdom", packet),
            JsxOutputMode::Vapor => {
                insta::assert_debug_snapshot!("attribute_runtime_vapor", packet)
            }
        }
    }
}
