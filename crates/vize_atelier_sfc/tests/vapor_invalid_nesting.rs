//! Nestings the HTML parser would repair (`<form>` in `<form>`, a block in a
//! `<p>`, `<a>` in `<a>`, a row directly in a `<table>`) keep their authored
//! tree on every target. Vapor gives each such child a template of its own
//! and inserts it at runtime, as `@vue/compiler-vapor` 3.6 does, because
//! `innerHTML` would re-parent it inside one template string.

use vize_atelier_sfc::{
    SfcCompileOptions, SfcParseOptions, TemplateCompileOptions, compile_sfc, parse_sfc,
};

const SOURCE: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/vapor-invalid-nesting-templates/Layout.vue.txt"
);

fn compile(ssr: bool, vapor: bool) -> String {
    let parse = SfcParseOptions {
        filename: "Layout.vue".into(),
        ..Default::default()
    };
    let descriptor = parse_sfc(SOURCE, parse.clone()).expect("SFC");
    let result = compile_sfc(
        &descriptor,
        SfcCompileOptions {
            parse,
            template: TemplateCompileOptions {
                ssr,
                vapor,
                ..Default::default()
            },
            vapor,
            ..Default::default()
        },
    )
    .expect("compile");
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    result.code.as_str().to_owned()
}

#[test]
fn ssr_renders_the_authored_nesting() {
    assert_eq!(
        compile(true, false),
        include_str!(
            "../../../tests/_fixtures/differential/compiler/vapor-invalid-nesting-templates/module.expected.txt"
        )
    );
}

#[test]
fn vapor_inserts_each_invalid_child_from_its_own_template() {
    assert_eq!(
        compile(false, true),
        include_str!("fixtures/vapor_invalid_nesting/vapor.expected.txt")
    );
}
