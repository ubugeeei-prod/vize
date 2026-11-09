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

/// The same dynamic name built from the loop alias is fine in loops over different sources.
#[test]
fn test_valid_same_dynamic_slot_expression_in_separate_loops() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<MyComponent>
            <template v-for="id in first" :key="id" #[`popup:${id}`]="{ item }">{{ item }}</template>
            <template v-for="id in second" :key="id" #[`popup:${id}`]="{ item }">{{ item }}</template>
        </MyComponent>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

/// Plain static duplicates are still reported.
#[test]
fn test_invalid_duplicate_static_slot_is_still_reported() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<MyComponent>
            <template #header>A</template>
            <template #header>B</template>
        </MyComponent>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 1);
}

/// Identical loops produce the same slot names, so the second slot is a duplicate.
#[test]
fn test_invalid_same_dynamic_slot_in_identical_loops() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<MyComponent>
            <template v-for="id in ids" :key="id" #[`popup:${id}`]="{ item }">{{ item }}</template>
            <template v-for="id in ids" :key="id" #[`popup:${id}`]="{ item }">{{ item }}</template>
        </MyComponent>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 1);
}

/// Number of errors `vue/valid-v-slot` reports for `template`.
fn error_count(template: &str) -> usize {
    create_linter()
        .lint_template(template, "test.vue")
        .error_count
}

/// `of` loops behave like `in` loops.
#[test]
fn test_valid_same_dynamic_slot_in_separate_of_loops() {
    assert_eq!(
        error_count(
            r#"<MyComponent>
            <template v-for="id of first" :key="id" #[`p:${id}`]="{ item }">{{ item }}</template>
            <template v-for="id of second" :key="id" #[`p:${id}`]="{ item }">{{ item }}</template>
        </MyComponent>"#
        ),
        0
    );
}

/// Destructured aliases count as loop aliases.
#[test]
fn test_valid_same_dynamic_slot_with_destructured_aliases_in_separate_loops() {
    assert_eq!(
        error_count(
            r#"<MyComponent>
            <template v-for="{ id } in first" :key="id" #[`p:${id}`]="{ item }">{{ item }}</template>
            <template v-for="{ id } in second" :key="id" #[`p:${id}`]="{ item }">{{ item }}</template>
        </MyComponent>"#
        ),
        0
    );
}

/// Tabs, newlines and extra spaces around `in` do not make a loop different.
#[test]
fn test_invalid_identical_loops_with_different_spacing_are_duplicates() {
    assert_eq!(
        error_count(
            "<MyComponent>\n\
             <template v-for=\"id in ids\" :key=\"id\" #[`p:${id}`]=\"{ a }\">{{ a }}</template>\n\
             <template v-for=\"id\tin\n  ids\" :key=\"id\" #[`p:${id}`]=\"{ a }\">{{ a }}</template>\n\
             </MyComponent>"
        ),
        1
    );
}

/// An alias like `inner` is not split at the `in` inside it.
#[test]
fn test_loop_aliases_containing_in_or_of_are_not_split() {
    let template = |second: &str| {
        format!(
            r#"<MyComponent>
            <template v-for="inner of items" :key="inner" #[`p:${{inner}}`]="{{ a }}">{{{{ a }}}}</template>
            <template v-for="inner of {second}" :key="inner" #[`p:${{inner}}`]="{{ a }}">{{{{ a }}}}</template>
        </MyComponent>"#
        )
    };
    assert_eq!(error_count(&template("items")), 1);
    assert_eq!(error_count(&template("other")), 0);
}

/// A dynamic name that reads no loop alias is still a duplicate across loops.
#[test]
fn test_invalid_dynamic_slot_without_loop_alias_is_duplicate_across_loops() {
    assert_eq!(
        error_count(
            r#"<MyComponent>
            <template v-for="id in first" :key="id" #[name]="{ a }">{{ a }}</template>
            <template v-for="id in second" :key="id" #[name]="{ a }">{{ a }}</template>
        </MyComponent>"#
        ),
        1
    );
}

/// `x.id` reads `x`, not the loop alias `id`, so sibling slots are duplicates.
#[test]
fn test_invalid_property_named_like_loop_alias_is_not_a_loop_read() {
    assert_eq!(
        error_count(
            r#"<MyComponent>
            <template v-for="id in first" :key="id" #[x.id]="{ a }">{{ a }}</template>
            <template v-for="id in second" :key="id" #[x.id]="{ a }">{{ a }}</template>
        </MyComponent>"#
        ),
        1
    );
}
