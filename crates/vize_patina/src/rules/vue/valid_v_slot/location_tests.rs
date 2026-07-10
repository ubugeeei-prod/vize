use super::ValidVSlot;
use crate::linter::Linter;
use crate::rule::RuleRegistry;

fn create_linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(ValidVSlot));
    Linter::with_registry(registry)
}

#[test]
fn invalid_slot_template_under_plain_element() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<div><template #header>Header</template></div>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 1);
}
