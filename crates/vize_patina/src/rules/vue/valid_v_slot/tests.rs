use super::ValidVSlot;
use crate::linter::Linter;
use crate::rule::RuleRegistry;

fn create_linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(ValidVSlot));
    Linter::with_registry(registry)
}

#[test]
fn test_valid_default_slot() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<MyComponent v-slot="{ item }">{{ item }}</MyComponent>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_named_slot_template() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<MyComponent><template #header>Header</template></MyComponent>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_default_slot_argument_on_component() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<MyComponent v-slot:default="{ item }">{{ item }}</MyComponent>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_dynamic_component_slot() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<component :is="to ? NuxtLinkLocale : 'button'" #="scoped">{{ scoped }}</component>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_invalid_on_html_element() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<div v-slot:header></div>"#, "test.vue");
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_named_slot_on_component() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<MyComponent v-slot:header>Header</MyComponent>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_valid_multiple_named_slots() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<MyComponent>
            <template #header>Header</template>
            <template #footer>Footer</template>
        </MyComponent>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_dotted_vuetify_data_table_slots() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<v-data-table>
            <template v-slot:item.tagName="{ item }">{{ item.tagName }}</template>
            <template v-slot:item.memo="{ item }">{{ item.memo }}</template>
        </v-data-table>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_invalid_duplicate_dotted_slot_name() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<v-data-table>
            <template v-slot:item.memo="{ item }">{{ item.memo }}</template>
            <template v-slot:item.memo="{ item }">{{ item.memo }}</template>
        </v-data-table>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 1);
}
