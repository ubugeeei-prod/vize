use super::compile_scoped_css_without_whitespace;

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
