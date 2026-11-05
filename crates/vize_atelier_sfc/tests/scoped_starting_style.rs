use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};

#[test]
fn starting_style_keeps_scoped_css_and_complete_ssr_module() {
    let source = include_str!(
        "../../../tests/_fixtures/differential/compiler/scoped-starting-style-11850/Layout.vue.txt"
    );
    let parse = SfcParseOptions {
        filename: "Layout.vue".into(),
        ..Default::default()
    };
    let descriptor = parse_sfc(source, parse.clone()).unwrap();
    let mut options = SfcCompileOptions {
        parse,
        ..Default::default()
    };
    options.template.ssr = true;
    let output = compile_sfc(&descriptor, options).unwrap();
    assert!(output.errors.is_empty(), "{:?}", output.errors);
    assert!(output.warnings.is_empty(), "{:?}", output.warnings);
    assert_eq!(
        output.code.as_str(),
        include_str!(
            "../../../tests/_fixtures/differential/compiler/scoped-starting-style-11850/module.expected.txt"
        )
    );
    assert_eq!(
        output.css.as_deref(),
        Some(include_str!(
            "../../../tests/_fixtures/differential/compiler/scoped-starting-style-11850/style.expected.css"
        ))
    );
}
