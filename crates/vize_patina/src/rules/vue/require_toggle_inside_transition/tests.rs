use super::RequireToggleInsideTransition;
use crate::linter::Linter;
use crate::rule::RuleRegistry;

fn create_linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(RequireToggleInsideTransition));
    Linter::with_registry(registry)
}

#[test]
fn test_invalid_plain_element() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<transition><div>content</div></transition>"#, "test.vue");
    assert_eq!(result.error_count, 1);
    assert_eq!(
        result.diagnostics[0].rule_name,
        "vue/require-toggle-inside-transition"
    );
    insta::with_settings!({ snapshot_path => "../snapshots" }, {
        insta::assert_debug_snapshot!(result.diagnostics);
    });
}

#[test]
fn test_invalid_pascal_case_transition() {
    let linter = create_linter();
    // `<Transition>` (PascalCase) resolves to the same built-in.
    let result = linter.lint_template(r#"<Transition><p>x</p></Transition>"#, "test.vue");
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_static_key_only() {
    let linter = create_linter();
    // A *static* `key` attribute does not change, so it does not trigger an
    // enter/leave the way a bound `:key` does.
    let result = linter.lint_template(
        r#"<transition><div key="static">x</div></transition>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_unrelated_binding() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<transition><div :class="cls">x</div></transition>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_valid_v_if() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<transition><div v-if="show">x</div></transition>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_v_show() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<transition><div v-show="show">x</div></transition>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_v_else() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<transition><div v-else>x</div></transition>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_v_else_if() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<transition><div v-else-if="b">x</div></transition>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_bound_key() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<transition><div :key="k">x</div></transition>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_v_bind_key_longhand() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<transition><div v-bind:key="k">x</div></transition>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_transition_appear() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"
        <Transition name="fade" appear>
          <div>loading</div>
        </Transition>
        <Transition name="fade">
          <div>static</div>
        </Transition>
        "#,
        "test.vue",
    );
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_valid_transition_bound_appear() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<transition :appear="shouldAppear"><div>loading</div></transition>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_dynamic_component() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<transition><component :is="view" /></transition>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_custom_component_child() {
    let linter = create_linter();
    // A custom component may toggle itself internally; do not flag it.
    let result = linter.lint_template(r#"<transition><MyModal /></transition>"#, "test.vue");
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_slot_child() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<transition><slot /></transition>"#, "test.vue");
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_empty_transition() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<transition></transition>"#, "test.vue");
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_multiple_children() {
    let linter = create_linter();
    // A v-if/v-else pair is two element children; the toggle lives on the
    // pair, not a single wrapped element, so the rule does not apply.
    let result = linter.lint_template(
        r#"<transition><div v-if="a">a</div><div v-else>b</div></transition>"#,
        "test.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_invalid_ignores_surrounding_comment_and_whitespace() {
    let linter = create_linter();
    // Comments and whitespace around the wrapped element are ignored when
    // locating the single child, so this still reports.
    let result = linter.lint_template(
        "<transition>\n  <!-- wrap --><div>x</div>\n</transition>",
        "test.vue",
    );
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_valid_non_transition_element() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<div><span>x</span></div>"#, "test.vue");
    assert_eq!(result.error_count, 0);
}
