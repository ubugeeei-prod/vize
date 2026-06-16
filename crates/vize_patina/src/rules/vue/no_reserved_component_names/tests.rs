use super::NoReservedComponentNames;
use crate::linter::Linter;
use crate::rule::RuleRegistry;

fn create_linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(NoReservedComponentNames::default()));
    Linter::with_registry(registry)
}

#[test]
fn test_valid_custom_component() {
    let linter = create_linter();
    let result = linter.lint_sfc(
        r#"<script>export default { name: 'MyComponent' }</script><template><div>hello</div></template>"#,
        "MyComponent.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_pascal_case_html_filename_is_valid_without_explicit_name() {
    let linter = create_linter();
    let result = linter.lint_sfc(
        r#"<script setup></script><template><div /></template>"#,
        "Button.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_explicit_pascal_case_html_name_is_valid() {
    let linter = create_linter();
    let result = linter.lint_sfc(
        r#"<script>export default { name: 'Button' }</script><template><div /></template>"#,
        "Button.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_invalid_explicit_html_name() {
    let linter = create_linter();
    let result = linter.lint_sfc(
        r#"<script>export default { name: 'button' }</script><template><div>hello</div></template>"#,
        "Button.vue",
    );
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_define_options_html_name() {
    let linter = create_linter();
    let result = linter.lint_sfc(
        r#"<script setup>defineOptions({ name: 'button' })</script><template><div /></template>"#,
        "Button.vue",
    );
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_explicit_vue_builtin() {
    let linter = create_linter();
    let result = linter.lint_sfc(
        r#"<script>export default { name: 'Transition' }</script><template><div>hello</div></template>"#,
        "Transition.vue",
    );
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_using_transition_in_template_is_valid() {
    let linter = create_linter();
    let result = linter.lint_sfc(
        r#"<script>export default { name: 'MyComponent' }</script><template><Transition name="fade"><div>hello</div></Transition></template>"#,
        "MyComponent.vue",
    );
    assert_eq!(
        result.error_count, 0,
        "Using Vue built-in <Transition> in template should not be flagged"
    );
}

#[test]
fn test_using_keep_alive_in_template_is_valid() {
    let linter = create_linter();
    let result = linter.lint_sfc(
        r#"<script>export default { name: 'MyComponent' }</script><template><KeepAlive><div>hello</div></KeepAlive></template>"#,
        "MyComponent.vue",
    );
    assert_eq!(
        result.error_count, 0,
        "Using Vue built-in <KeepAlive> in template should not be flagged"
    );
}

#[test]
fn test_using_teleport_in_template_is_valid() {
    let linter = create_linter();
    let result = linter.lint_sfc(
        r#"<script>export default { name: 'MyComponent' }</script><template><Teleport to="body"><div>hello</div></Teleport></template>"#,
        "MyComponent.vue",
    );
    assert_eq!(
        result.error_count, 0,
        "Using Vue built-in <Teleport> in template should not be flagged"
    );
}

#[test]
fn test_using_suspense_in_template_is_valid() {
    let linter = create_linter();
    let result = linter.lint_sfc(
        r#"<script>export default { name: 'MyComponent' }</script><template><Suspense><div>hello</div></Suspense></template>"#,
        "MyComponent.vue",
    );
    assert_eq!(
        result.error_count, 0,
        "Using Vue built-in <Suspense> in template should not be flagged"
    );
}

#[test]
fn test_non_vue_file() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<div>hello</div>"#, "test.html");
    assert_eq!(result.error_count, 0);
}
