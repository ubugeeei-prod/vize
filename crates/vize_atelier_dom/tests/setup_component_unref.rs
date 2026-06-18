use vize_atelier_core::options::{BindingMetadata, BindingType, CodegenMode};
use vize_atelier_dom::{DomCompilerOptions, compile_template_with_options};
use vize_carton::{Bump, FxHashMap, String};

fn full_output(preamble: &str, code: &str) -> String {
    let mut full = String::with_capacity(preamble.len() + code.len() + 1);
    full.push_str(preamble);
    full.push('\n');
    full.push_str(code);
    full
}

#[test]
fn inline_setup_ref_component_tag_uses_unref() {
    let allocator = Bump::new();
    let mut bindings = FxHashMap::default();
    bindings.insert("Menu".into(), BindingType::SetupRef);
    bindings.insert("show".into(), BindingType::SetupRef);
    bindings.insert("items".into(), BindingType::SetupRef);

    let options = DomCompilerOptions {
        mode: CodegenMode::Module,
        prefix_identifiers: true,
        inline: true,
        binding_metadata: Some(BindingMetadata {
            bindings,
            props_aliases: FxHashMap::default(),
            is_script_setup: true,
        }),
        ..Default::default()
    };

    let (_, errors, result) = compile_template_with_options(
        &allocator,
        r#"<div><Menu>hello</Menu><Menu v-if="show" /><Menu v-for="item in items" :key="item" /><Menu v-once /><Menu.Item /></div>"#,
        options,
    );

    assert!(errors.is_empty(), "Errors: {:?}", errors);
    let full = full_output(&result.preamble, &result.code);
    assert!(full.contains("unref as _unref"), "{full}");
    assert_eq!(
        full.matches("_unref(Menu").count(),
        5,
        "every setup ref component tag path should be unref'd:\n{full}"
    );
    assert!(
        full.contains("_createBlock(_unref(Menu)") || full.contains("_createVNode(_unref(Menu)"),
        "setup ref component tags must be unref'd:\n{full}"
    );
    assert!(
        full.contains("_createVNode(_unref(Menu))"),
        "v-once component tags must be unref'd:\n{full}"
    );
    assert!(
        full.contains("_unref(Menu).Item"),
        "dotted setup ref component tags must unref the base binding:\n{full}"
    );
    assert!(
        !full.contains("_createBlock(Menu") && !full.contains("_createVNode(Menu"),
        "raw ref component tag must not be emitted:\n{full}"
    );
}
