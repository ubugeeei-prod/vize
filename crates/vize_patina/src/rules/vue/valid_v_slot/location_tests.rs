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

/// eslint-plugin-vue reports `v-slot` modifiers at the modifiers themselves.
#[test]
fn modifiers_are_reported_at_their_own_range() {
    let linter = create_linter();
    for (source, expected) in [
        (
            r#"<MyList><template #item.name="{ item }">{{ item }}</template></MyList>"#,
            "name",
        ),
        (
            r#"<MyList><template v-slot:item.raw.value="{ item }">{{ item }}</template></MyList>"#,
            "raw.value",
        ),
        (
            r#"<MyList v-slot.trim="{ item }">{{ item }}</MyList>"#,
            "trim",
        ),
    ] {
        let result = linter.lint_template(source, "test.vue");
        assert_eq!(result.error_count, 1, "{source}: {:?}", result.diagnostics);
        let diagnostic = &result.diagnostics[0];
        assert_eq!(
            &source[diagnostic.start as usize..diagnostic.end as usize],
            expected,
            "{source}"
        );
    }
}
