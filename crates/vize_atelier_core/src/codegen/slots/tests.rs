use super::super::helpers::is_valid_js_identifier;
use super::params::prefix_slot_defaults;

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
