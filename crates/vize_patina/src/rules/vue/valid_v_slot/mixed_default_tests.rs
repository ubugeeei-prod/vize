use super::ValidVSlot;
use crate::linter::Linter;
use crate::rule::RuleRegistry;

fn create_linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(ValidVSlot));
    Linter::with_registry(registry)
}

#[test]
fn invalid_default_owner_slot_with_named_child_slot() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<MyComponent v-slot="slotProps">
            <template #header>Header</template>
            {{ slotProps }}
        </MyComponent>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 1);
}

#[test]
fn invalid_default_owner_slot_argument_with_named_child_slot() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<MyComponent v-slot:default="slotProps">
            <template #header>Header</template>
            {{ slotProps }}
        </MyComponent>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 1);
}

#[test]
fn valid_default_template_slot_with_named_child_slot() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<MyComponent>
            <template #default="slotProps">{{ slotProps }}</template>
            <template #header>Header</template>
        </MyComponent>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn valid_duplicate_named_slot_in_if_else_chain() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<MyComponent>
            <template v-if="ok" #header>Header A</template>
            <template v-else #header>Header B</template>
        </MyComponent>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn invalid_duplicate_named_slot_templates() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<MyComponent>
            <template #header>Header A</template>
            <template #header>Header B</template>
        </MyComponent>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 1);
}

#[test]
fn invalid_default_owner_slot_with_default_child_slot() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<MyComponent v-slot="slotProps">
            <template #default>Fallback</template>
            {{ slotProps }}
        </MyComponent>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 1);
}
