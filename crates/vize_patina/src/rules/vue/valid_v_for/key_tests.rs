use super::ValidVFor;
use crate::linter::Linter;
use crate::rule::RuleRegistry;

fn create_linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(ValidVFor));
    Linter::with_registry(registry)
}

#[test]
fn valid_v_for_key_uses_alias() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<div v-for="item in items" :key="item.id"></div>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn valid_v_for_key_uses_second_alias() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<div v-for="(value, key, index) in items" :key="key"></div>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn valid_v_for_key_uses_destructured_tuple_alias() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<ul>
            <li v-for="({ color }, index) of colors" :key="color">{{ index }}</li>
            <li v-for="([name, count], index) of entries" :key="name">{{ count }}</li>
        </ul>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn valid_v_for_static_key_is_ignored() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<div v-for="item in items" key="static"></div>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn invalid_v_for_key_without_alias_reference() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<div v-for="item in items" :key="other"></div>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 1);
}

#[test]
fn invalid_v_for_key_does_not_match_substring() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<div v-for="item in items" :key="itemId"></div>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 1);
}
