use super::super::helpers::is_valid_js_identifier;
use super::params::prefix_slot_defaults;
use crate::compile;

fn result_output(result: &super::super::CodegenResult) -> vize_carton::String {
    let mut output =
        vize_carton::String::with_capacity(result.preamble.len() + result.code.len() + 1);
    output.push_str(&result.preamble);
    output.push('\n');
    output.push_str(&result.code);
    output
}

#[test]
fn test_is_valid_js_identifier_valid() {
    assert!(is_valid_js_identifier("foo"));
    assert!(is_valid_js_identifier("_bar"));
    assert!(is_valid_js_identifier("$baz"));
    assert!(is_valid_js_identifier("foo123"));
    assert!(is_valid_js_identifier("camelCase"));
    assert!(is_valid_js_identifier("PascalCase"));
}

#[test]
fn test_is_valid_js_identifier_invalid() {
    assert!(!is_valid_js_identifier("123foo")); // starts with number
    assert!(!is_valid_js_identifier("")); // empty
    assert!(!is_valid_js_identifier("foo-bar")); // contains hyphen
    assert!(!is_valid_js_identifier("foo.bar")); // contains dot
    assert!(!is_valid_js_identifier("foo bar")); // contains space
    assert!(!is_valid_js_identifier("item-header")); // hyphenated slot name
}

#[test]
fn test_hyphenated_slot_names_need_quotes() {
    assert!(!is_valid_js_identifier("item-header"));
    assert!(!is_valid_js_identifier("card-body"));
    assert!(!is_valid_js_identifier("main-content"));
    assert!(!is_valid_js_identifier("list-item"));
}

#[test]
fn test_regular_slot_names_are_valid_identifiers() {
    assert!(is_valid_js_identifier("default"));
    assert!(is_valid_js_identifier("header"));
    assert!(is_valid_js_identifier("footer"));
    assert!(is_valid_js_identifier("content"));
}

#[test]
fn test_prefix_slot_defaults() {
    // Default values should get _ctx. prefix
    assert_eq!(
        prefix_slot_defaults("{ item = defaultItem }"),
        "{ item = _ctx.defaultItem }"
    );
    assert_eq!(prefix_slot_defaults("{ count = 0 }"), "{ count = 0 }");
    assert_eq!(
        prefix_slot_defaults("{ name = 'test' }"),
        "{ name = 'test' }"
    );
    // Literals should not be prefixed
    assert_eq!(prefix_slot_defaults("{ x = true }"), "{ x = true }");
    assert_eq!(prefix_slot_defaults("{ x = false }"), "{ x = false }");
    assert_eq!(prefix_slot_defaults("{ x = null }"), "{ x = null }");
    assert_eq!(
        prefix_slot_defaults("{ x = undefined }"),
        "{ x = undefined }"
    );
}

#[test]
fn slot_outlet_vbind_object_preserves_optional_chaining() {
    let result = compile!(
        r#"<slot v-bind="external ? { isActive: undefined } : { isActive: scope?.isActive }" />"#
    );
    let output = result_output(&result);

    assert!(
        output.contains(r#"external ? { isActive: undefined } : { isActive: scope?.isActive }"#),
        "slot outlet ternary v-bind object must preserve optional chaining:\n{output}"
    );
    assert!(
        !output.contains(r#"{ isActive: scope.isActive }"#),
        "slot outlet ternary v-bind object must not emit an unguarded member access:\n{output}"
    );
}
