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
fn vue_is_substitution_accepts_a_slot_template() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<div is="vue:my-list"><template #item>a</template></div>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
    let native = linter.lint_template(r#"<div><template #item>a</template></div>"#, "test.vue");
    assert_eq!(native.error_count, 1);
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
fn v_for_slot_name_must_use_an_iteration_binding() {
    let linter = create_linter();
    let invalid = linter.lint_template(
        r#"<MyTabs>
            <template v-for="tab in tabs" :key="tab.id" #panel>{{ tab.label }}</template>
            <template v-for="column in columns" :key="column.key" #[slotName]>{{ column.label }}</template>
        </MyTabs>"#,
        "test.vue",
    );
    assert_eq!(invalid.error_count, 2, "{:?}", invalid.diagnostics);

    let valid = linter.lint_template(
        r#"<MyTabs><template v-for="tab in tabs" :key="tab.id" #[tab.id] /></MyTabs>"#,
        "test.vue",
    );
    assert_eq!(valid.error_count, 0, "{:?}", valid.diagnostics);
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

#[test]
fn dynamic_slot_names_from_distinct_iteration_scopes_do_not_collide() {
    let linter = create_linter();
    for source in [
        r#"<MyComponent>
            <template v-for="id in first" #[`popup:${id}`]>{{ id }}</template>
            <template v-for="id in second" #[`popup:${id}`]>{{ id }}</template>
        </MyComponent>"#,
        r#"<MyComponent>
            <template v-for="({ id }) of first" #[`popup:${id}`]>{{ id }}</template>
            <template v-for="({ id }) of second" #[`popup:${id}`]>{{ id }}</template>
        </MyComponent>"#,
    ] {
        let result = linter.lint_template(source, "test.vue");
        assert_eq!(result.error_count, 0, "{source}");
    }
}

#[test]
fn equivalent_iteration_scopes_still_collide_across_siblings() {
    let linter = create_linter();
    for source in [
        r#"<MyComponent>
            <template v-for="id in items" #[`popup:${id}`]>{{ id }}</template>
            <template v-for="id of items" #[`popup:${id}`]>{{ id }}</template>
        </MyComponent>"#,
        "<MyComponent>\n\
            <template v-for=\"({ id }, index) in items\" #[`${index}:${id}`] />\n\
            <template v-for=\"( { id }, index)\tin\titems\" #[`${index}:${id}`] />\n\
        </MyComponent>",
    ] {
        let result = linter.lint_template(source, "test.vue");
        assert_eq!(result.error_count, 1, "{source}");
    }
}

#[test]
fn property_names_and_literals_do_not_impersonate_iteration_bindings() {
    let linter = create_linter();
    for source in [
        r#"<MyComponent>
            <template v-for="id in first" #[owner.id] />
            <template v-for="id in second" #[owner.id] />
        </MyComponent>"#,
        r#"<MyComponent>
            <template v-for="id in first" #[`literal:id`] />
            <template v-for="id in second" #[`literal:id`] />
        </MyComponent>"#,
    ] {
        let result = linter.lint_template(source, "test.vue");
        assert_eq!(result.error_count, 3, "{source}");
    }
}

#[test]
fn loop_scoped_names_differ_from_the_same_text_outside_the_loop() {
    let linter = create_linter();
    for source in [
        r#"<MyTable><template #[name]>a</template><template v-for="name in names" :key="name" #[name]>b</template></MyTable>"#,
        r#"<MyTable><template v-for="name in names" :key="name" #[name]>b</template><template #[name]>a</template></MyTable>"#,
        r#"<MyTable><template v-for="(key, value) in rows" :key="key" #[key]>b</template><template #[key]>a</template></MyTable>"#,
    ] {
        let result = linter.lint_template(source, "test.vue");
        assert_eq!(result.error_count, 0, "{source}: {:?}", result.diagnostics);
    }
}

#[test]
fn comment_only_default_slot_value_declares_no_parameter() {
    let linter = create_linter();
    for source in [
        r#"<MyTable v-slot="/* no parameters */">a</MyTable>"#,
        r#"<MyTable #default="/* none */">a</MyTable>"#,
        r#"<MyTable v-slot="/**/ /* */">a</MyTable>"#,
    ] {
        let result = linter.lint_template(source, "test.vue");
        assert_eq!(result.error_count, 1, "{source}: {:?}", result.diagnostics);
    }
    let result = linter.lint_template(r#"<MyTable v-slot="/* a */ props">a</MyTable>"#, "test.vue");
    assert_eq!(result.error_count, 0, "{:?}", result.diagnostics);
}
