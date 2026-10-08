use super::{CssCompileOptions, compile_css, compile_scoped_css_without_whitespace};

#[test]
fn direct_scoped_css_matches_vite_pipeline_for_nested_and_slotted_rules() {
    let css = ".c { & .d { color: red; } }\n.a > :slotted(.b) { color: blue; }";
    let scope_id = "data-v-x";
    let pipeline = crate::vite_plugin::scope_css_for_pipeline(css, scope_id);
    let expected = compile_css(&pipeline, &CssCompileOptions::default());
    let actual = compile_css(
        css,
        &CssCompileOptions {
            scoped: true,
            scope_id: Some(scope_id.into()),
            ..Default::default()
        },
    );
    assert!(actual.errors.is_empty(), "{:?}", actual.errors);
    assert_eq!(actual.code, expected.code);
}

#[test]
fn test_compile_scoped_css_keeps_functional_pseudo_selector_list_intact() {
    let code = compile_scoped_css_without_whitespace(".btn:is(.a, .b) { color: red; }");

    assert_eq!(code, ".btn[data-v-123]:is(.a,.b){color:red;}");
}

#[test]
fn test_compile_scoped_css_scopes_before_functional_pseudo() {
    let code = compile_scoped_css_without_whitespace(".foo:not(:hover) { color: red; }");

    assert_eq!(code, ".foo[data-v-123]:not(:hover){color:red;}");
}

#[test]
fn test_compile_scoped_css_keeps_functional_pseudo_whitespace_intact() {
    let code = compile_scoped_css_without_whitespace(".card:has(.icon + .label) { color: red; }");

    assert_eq!(code, ".card[data-v-123]:has(.icon+.label){color:red;}");
}

#[test]
fn test_compile_scoped_css_scopes_parent_before_trailing_universal() {
    let code = compile_scoped_css_without_whitespace(".dialog__action-buttons > * { flex: 1; }");

    assert_eq!(code, ".dialog__action-buttons[data-v-123]>*{flex:1;}");
}

#[test]
fn test_compile_scoped_css_scopes_parent_before_universal_pseudo() {
    let code =
        compile_scoped_css_without_whitespace(".dialog__action-buttons>*:hover { flex: 1; }");

    assert_eq!(code, ".dialog__action-buttons[data-v-123]>:hover{flex:1;}");
}

#[test]
fn test_compile_scoped_css_keeps_slotted_parent_combinator() {
    let code = compile_scoped_css_without_whitespace(
        ".card > :slotted(*) { flex: 1; }\
         .card > :slotted(.title),\
         .card > :slotted(.subtitle) { line-height: 1.2; }\
         .card > :slotted(*):not(:last-child) { margin-bottom: 2px; }",
    );

    assert_eq!(
        code,
        ".card>[data-v-123-s]{flex:1;}\
         .card>.title[data-v-123-s],\
         .card>.subtitle[data-v-123-s]{line-height:1.2;}\
         .card>[data-v-123-s]:not(:last-child){margin-bottom:2px;}"
    );
}
