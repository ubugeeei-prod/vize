//! Exhaustive downstream option literals from the published 0.429.0 API.
use vize_atelier_sfc::{
    SfcCompileOptions, SfcParseOptions, TemplateCompileOptions, compile_sfc, parse_sfc,
};

#[test]
#[expect(
    clippy::expect_used,
    reason = "existing authored fixtures must parse and compile"
)]
fn published_template_options_literal_preserves_complete_scoped_module() {
    let template = TemplateCompileOptions {
        id: None,
        ssr: true,
        ssr_css_vars: None,
        scoped: false,
        is_prod: false,
        is_ts: false,
        custom_renderer: false,
        dialect: Default::default(),
        compiler_options: None,
    };
    let parse = SfcParseOptions {
        filename: "Layout.vue".into(),
        ..Default::default()
    };
    let source = include_str!("../../../tests/fixtures/sfc/ssr-slot-scope/plain-scoped.vue");
    let descriptor = parse_sfc(source, parse.clone()).expect("authored SFC parses");
    let result = compile_sfc(
        &descriptor,
        SfcCompileOptions {
            parse,
            scope_id: Some("layout".into()),
            template,
            ..Default::default()
        },
    )
    .expect("published literal compiles");
    assert_eq!(
        result.code.as_str(),
        include_str!("../../../tests/fixtures/sfc/ssr-slot-scope/scoped.expected.txt")
    );
    assert!(result.errors.is_empty());
    assert!(result.warnings.is_empty());
}
