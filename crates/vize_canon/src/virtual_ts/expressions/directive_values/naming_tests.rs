use super::directive_binding_name;

#[test]
fn resolves_vue_directive_naming_convention() {
    assert_eq!(directive_binding_name("focus").as_str(), "vFocus");
    assert_eq!(
        directive_binding_name("my-directive").as_str(),
        "vMyDirective"
    );
    assert_eq!(directive_binding_name("a").as_str(), "vA");
}
