use super::NoUnusedComponents;
use crate::rule::{Rule, RuleCategory};

#[test]
fn test_meta() {
    let rule = NoUnusedComponents::default();
    assert_eq!(rule.meta().name, "vue/no-unused-components");
    assert_eq!(rule.meta().category, RuleCategory::Essential);
}

#[test]
fn test_should_ignore() {
    let rule = NoUnusedComponents::default();
    assert!(rule.should_ignore("_Internal"));
    assert!(!rule.should_ignore("MyComponent"));
}
