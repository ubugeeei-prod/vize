//! Unit coverage for `attribute_priority`'s patina `vue/attribute-order` group
//! ordering. Kept in its own file so the already-large `template.rs` stays
//! within the source-file-length budget (#3251).

use super::attributes::attribute_priority;

#[test]
fn test_attribute_priority_order() {
    // Group order mirrors patina's vue/attribute-order (#3251).
    assert!(attribute_priority("is") < attribute_priority("v-for"));
    assert!(attribute_priority("v-for") < attribute_priority("v-if"));
    // Conditionals are one group: v-if, v-else-if, v-else, v-show, v-cloak.
    assert_eq!(attribute_priority("v-if"), attribute_priority("v-show"));
    assert_eq!(attribute_priority("v-if"), attribute_priority("v-cloak"));
    // Render modifiers sit between conditionals and id.
    assert!(attribute_priority("v-show") < attribute_priority("v-pre"));
    assert_eq!(attribute_priority("v-pre"), attribute_priority("v-once"));
    assert!(attribute_priority("v-once") < attribute_priority("id"));
    assert!(attribute_priority("id") < attribute_priority("ref"));
    assert_eq!(attribute_priority("ref"), attribute_priority("slot"));
    assert_eq!(attribute_priority("slot"), attribute_priority("slot-scope"));
    assert_eq!(attribute_priority("ref"), attribute_priority(":key"));
    assert!(attribute_priority(":key") < attribute_priority("v-model"));
    // Slots and custom directives precede plain attributes and bindings.
    assert!(attribute_priority("v-model") < attribute_priority("#default"));
    assert_eq!(
        attribute_priority("#default"),
        attribute_priority("v-tooltip")
    );
    assert!(attribute_priority("v-tooltip") < attribute_priority(":class"));
    // :class and class share the same priority so they stay adjacent;
    // patina treats :ref and :id as plain bindings.
    assert_eq!(attribute_priority(":class"), attribute_priority("class"));
    assert_eq!(attribute_priority(":style"), attribute_priority("style"));
    assert_eq!(attribute_priority(":ref"), attribute_priority(":class"));
    assert_eq!(attribute_priority(":id"), attribute_priority(":class"));
    // Events come after attributes, content comes last.
    assert!(attribute_priority("class") < attribute_priority("@click"));
    assert!(attribute_priority("@click") < attribute_priority("v-html"));
    assert_eq!(attribute_priority("v-html"), attribute_priority("v-text"));
}

#[test]
fn custom_directives_are_not_matched_by_builtin_prefixes() {
    // `v-models`/`v-onboarding`/`v-binding` are custom directives, not the
    // built-ins `v-model`/`v-on`/`v-bind`, so they must land in OtherDirectives
    // (7), the same group as any other unmatched `v-` directive.
    let other_directives = attribute_priority("v-tooltip");
    assert_eq!(attribute_priority("v-models"), other_directives);
    assert_eq!(attribute_priority("v-onboarding"), other_directives);
    assert_eq!(attribute_priority("v-binding"), other_directives);
    // Valid argument/modifier forms of the built-ins still match.
    assert_eq!(
        attribute_priority("v-model"),
        attribute_priority("v-model.trim")
    );
    assert_eq!(
        attribute_priority("v-on:click"),
        attribute_priority("@click")
    );
    assert_eq!(
        attribute_priority(":class"),
        attribute_priority("v-bind:class")
    );
}
