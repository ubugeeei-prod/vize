//! Literal names use the head boundary without interpreting the argument.

use super::frozen_attribute_name;
use vize_l0::Allocator;

#[test]
fn frozen_longhand_names_keep_quoted_brackets_and_all_modifiers() {
    let allocator = Allocator::new();
    for (authored, expected) in [
        (
            "v-bind:[keys['name]']].camel.stop",
            "v-bind[keys['name]']].camel.stop",
        ),
        (
            "v-on:[keys[\"event]\"]].once.capture",
            "v-on[keys[\"event]\"]].once.capture",
        ),
        ("v-例:[鍵['名]']].修飾", "v-例[鍵['名]']].修飾"),
        ("v-bind:[unterminated..camel", "v-bind[unterminated..camel"),
        ("v-bind:id..camel", "v-bindid..camel"),
    ] {
        assert_eq!(frozen_attribute_name(&allocator, authored), expected);
    }
}

#[test]
fn frozen_plain_and_shorthand_names_keep_the_original_borrow() {
    let allocator = Allocator::new();
    for authored in [
        "is",
        ":[keys['name]']].camel.stop",
        ".id.camel",
        "@[keys['event]']].once",
        "#default",
        "v-pre",
        "v-bind.camel:literal",
    ] {
        let frozen = frozen_attribute_name(&allocator, authored);
        assert_eq!(frozen, authored);
        assert!(core::ptr::eq(frozen, authored));
    }
}
