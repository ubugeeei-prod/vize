use vize_atelier_sfc::{
    css::{CssCompileOptions, compile_css},
    style::apply_scoped_css,
};

#[test]
fn bare_where_keeps_scope_attribute_inside_zero_specificity_pseudo() {
    let css = ":where(.card), .container > :where(.item):hover { display: flex; }";
    insta::assert_snapshot!("sfc", apply_scoped_css(css, "data-v-abc123"));

    let compiled = compile_css(
        css,
        &CssCompileOptions {
            scope_id: Some("data-v-abc123".into()),
            scoped: true,
            ..Default::default()
        },
    );
    assert!(compiled.errors.is_empty(), "{:?}", compiled.errors);
    insta::assert_snapshot!("css", compiled.code);
}
