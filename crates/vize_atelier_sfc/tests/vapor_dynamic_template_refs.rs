//! A Vapor template ref that can change value is set inside a render effect,
//! so the runtime setter moves it to the new key, and a `:ref` naming a
//! script-setup binding binds that binding by name. Static and literal refs
//! are set once. The SSR module is pinned as a compiler differential input,
//! and the Vapor module beside it.

use vize_atelier_sfc::{
    SfcCompileOptions, SfcParseOptions, TemplateCompileOptions, compile_sfc, parse_sfc,
};

const SOURCE: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/vapor-dynamic-template-refs/Layout.vue.txt"
);

fn compile(ssr: bool, vapor: bool) -> std::string::String {
    let parse = SfcParseOptions {
        filename: "Layout.vue".into(),
        ..Default::default()
    };
    let descriptor = parse_sfc(SOURCE, parse.clone()).expect("SFC");
    let result = compile_sfc(
        &descriptor,
        SfcCompileOptions {
            parse,
            vapor,
            template: TemplateCompileOptions {
                ssr,
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .expect("compile SFC");
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    result.code.as_str().to_owned()
}

#[test]
fn ssr_module_matches_the_pinned_differential_reference() {
    assert_eq!(
        compile(true, false),
        include_str!(
            "../../../tests/_fixtures/differential/compiler/vapor-dynamic-template-refs/module.expected.txt"
        )
    );
}

#[test]
fn vapor_module_sets_dynamic_refs_in_render_effects() {
    assert_eq!(
        compile(false, true),
        include_str!("fixtures/vapor_dynamic_template_refs/vapor.expected.txt")
    );
}
