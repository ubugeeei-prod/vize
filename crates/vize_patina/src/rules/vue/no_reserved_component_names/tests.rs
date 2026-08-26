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
fn test_explicit_pascal_case_html_name_is_invalid_like_eslint_plugin_vue() {
    let linter = create_linter();
    let result = linter.lint_sfc(
        r#"<script>export default { name: 'Button' }</script><template><div /></template>"#,
        "Button.vue",
    );
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_all_caps_html_name_is_valid_like_eslint_plugin_vue() {
    let linter = create_linter();
    let result = linter.lint_sfc(
        r#"<script>export default { name: 'BUTTON' }</script><template><div /></template>"#,
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
fn test_invalid_static_template_literal_names() {
    let linter = create_linter();
    let result = linter.lint_sfc(
        r#"<script>
export default { [`name`]: `button` }
</script>
<script setup>
defineOptions({ ['name']: `slot` })
</script>
<template><div /></template>"#,
        "Button.vue",
    );
    assert_eq!(result.error_count, 2);
}

#[test]
fn test_invalid_vue_component_html_registration() {
    let linter = create_linter();
    let result = linter.lint_sfc(
        r#"<script>Vue.component('Button', {})</script><template><div /></template>"#,
        "Demo.vue",
    );
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_vue_component_static_template_registration() {
    let linter = create_linter();
    let result = linter.lint_sfc(
        r#"<script>Vue.component(`button`, {})</script><template><div /></template>"#,
        "Demo.vue",
    );
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_app_component_html_registration() {
    let linter = create_linter();
    let result = linter.lint_sfc(
        r#"<script>
const app = createApp({})
app.component('button', {})
foo.component('Title', {})
</script>
<template><div /></template>"#,
        "Demo.vue",
    );
    assert_eq!(result.error_count, 2);
}

#[test]
fn test_dynamic_vue_component_registration_is_valid() {
    let linter = create_linter();
    let result = linter.lint_sfc(
        r#"<script>Vue.component(`button-${kind}`, {})</script><template><div /></template>"#,
        "Demo.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_vue_component_getter_is_valid() {
    let linter = create_linter();
    let result = linter.lint_sfc(
        r#"<script>Vue.component('Button')</script><template><div /></template>"#,
        "Demo.vue",
    );
    assert_eq!(result.error_count, 0);
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
fn test_invalid_registered_pascal_case_html_components() {
    let linter = create_linter();
    let result = linter.lint_sfc(
        r#"<script>
export default {
  components: {
    Title,
    Link,
    Header: SiteHeader,
  },
}
</script>
<template><div /></template>"#,
        "Demo.vue",
    );
    assert_eq!(result.error_count, 3);
}

#[test]
fn test_invalid_registered_pascal_case_svg_components() {
    let linter = create_linter();
    let result = linter.lint_sfc(
        r#"<script>
export default {
  components: {
    Text,
    Mask,
    feBlend,
  },
}
</script>
<template><div /></template>"#,
        "Demo.vue",
    );
    assert_eq!(result.error_count, 3);
}

#[test]
fn test_registered_non_upstream_case_variants_are_valid() {
    let linter = create_linter();
    let result = linter.lint_sfc(
        r#"<script>
export default {
  components: {
    BUTTON,
    FeBlend,
    TextPath,
  },
}
</script>
<template><div /></template>"#,
        "Demo.vue",
    );
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_invalid_registered_pascal_case_kebab_reserved_component() {
    let linter = create_linter();
    let result = linter.lint_sfc(
        r#"<script>
export default {
  components: {
    AnnotationXml,
  },
}
</script>
<template><div /></template>"#,
        "Demo.vue",
    );
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_registered_static_string_and_template_keys() {
    let linter = create_linter();
    let result = linter.lint_sfc(
        r#"<script>
export default {
  [`components`]: {
    'font-face': FontFace,
    [`missing-glyph`]: MissingGlyph,
  },
}
</script>
<template><div /></template>"#,
        "Demo.vue",
    );
    assert_eq!(result.error_count, 2);
}

#[test]
fn test_dynamic_registered_component_key_is_valid() {
    let linter = create_linter();
    let result = linter.lint_sfc(
        r#"<script>
const Button = 'CustomButton'
export default {
  components: {
    [Button]: Button,
    [`button-${kind}`]: Dynamic,
  },
}
</script>
<template><div /></template>"#,
        "Demo.vue",
    );
    assert_eq!(result.error_count, 0);
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
