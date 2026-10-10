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
fn test_valid_named_slot_template_on_lowercase_component() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<draggable :list="items">
            <template #item="{ element }">
                <span>{{ element }}</span>
            </template>
        </draggable>"#,
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
fn test_valid_dotted_vuetify_data_table_slots_when_modifiers_are_allowed() {
    let linter = create_linter().with_valid_v_slot_allow_modifiers(true);
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
    let linter = create_linter().with_valid_v_slot_allow_modifiers(true);
    let result = linter.lint_template(
        r#"<v-data-table>
            <template v-slot:item.memo="{ item }">{{ item.memo }}</template>
            <template v-slot:item.memo="{ item }">{{ item.memo }}</template>
        </v-data-table>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 1);
}

#[test]
fn explicit_false_restores_the_unit_rule_after_true_without_enabling_a_missing_rule() {
    let source = "<Panel><template #item.label>ready</template></Panel>";
    let original = create_linter().lint_template(source, "test.vue");
    let restored = create_linter()
        .with_valid_v_slot_allow_modifiers(true)
        .with_valid_v_slot_allow_modifiers(false)
        .lint_template(source, "test.vue");
    assert_eq!(format!("{original:?}"), format!("{restored:?}"));
    assert_eq!(original.error_count, 1);
    for allow in [false, true] {
        let absent = Linter::with_registry(RuleRegistry::new())
            .with_valid_v_slot_allow_modifiers(allow)
            .lint_template(source, "test.vue");
        assert!(absent.diagnostics.is_empty());
    }
    assert_eq!(core::mem::size_of_val(&ValidVSlot), 0);
    assert_eq!(core::mem::size_of_val(&ValidVSlot::allowing_modifiers()), 0);
}
