use vize_atelier_sfc::{
    SfcParseOptions, compile_sfc,
    css::{CssCompileOptions, compile_css},
    parse_sfc,
    types::SfcCompileOptions,
};

const CSS: &str = ".card > :slotted(.title) { font-weight: bold; }";

#[test]
fn scoped_slotted_selector_keeps_its_unscoped_parent() {
    let source = "<template><div class=\"card\"><slot /></div></template>\n<style scoped>.card > :slotted(.title) { font-weight: bold; }</style>";
    let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
    let result = compile_sfc(&descriptor, SfcCompileOptions::default()).unwrap();
    insta::assert_snapshot!("sfc", result.css.unwrap());

    let compiled = compile_css(
        CSS,
        &CssCompileOptions {
            scope_id: Some("data-v-abc123".into()),
            scoped: true,
            ..Default::default()
        },
    );
    assert!(compiled.errors.is_empty(), "{:?}", compiled.errors);
    insta::assert_snapshot!("css", compiled.code);
}
