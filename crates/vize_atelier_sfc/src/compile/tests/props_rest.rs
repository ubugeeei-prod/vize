use super::super::compile_sfc;
use crate::types::SfcCompileOptions;
use crate::{SfcParseOptions, parse_sfc};

#[test]
fn reported_props_rest_has_a_live_binding_in_both_script_modes() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/compiler/props-rest-runtime-8835/Child.vue.txt"
    ));
    for inline_template in [false, true] {
        for ssr in [false, true] {
            let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
            let mut options = SfcCompileOptions::default();
            options.script.inline_template = inline_template;
            options.template.ssr = ssr;
            let result = compile_sfc(&descriptor, options).unwrap();
            assert!(
                result.code.contains("createPropsRestProxy"),
                "{}",
                result.code
            );
            assert!(
                result
                    .code
                    .contains("const rest = _createPropsRestProxy(__props, [\"label\"]);"),
                "{}",
                result.code
            );
            assert!(!result.code.contains("__props.rest"), "{}", result.code);
            assert!(!result.code.contains("$props.rest"), "{}", result.code);
            assert_eq!(
                result.bindings.as_ref().unwrap().bindings.get("rest"),
                Some(&crate::types::BindingType::SetupReactiveConst)
            );
        }
    }
}

#[test]
fn props_rest_declaration_is_terminated_before_a_following_call_statement() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/compiler/props-rest-runtime-8835/FollowedByCall.vue.txt"
    ));
    for inline_template in [false, true] {
        let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
        let mut options = SfcCompileOptions::default();
        options.script.inline_template = inline_template;
        let result = compile_sfc(&descriptor, options).unwrap();
        // Without the terminator the IIFE would call the rest proxy itself.
        assert!(
            result
                .code
                .contains("const rest = _createPropsRestProxy(__props, [\"label\"]);"),
            "{}",
            result.code
        );
    }
}
