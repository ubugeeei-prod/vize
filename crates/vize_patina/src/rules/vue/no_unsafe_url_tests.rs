use super::NoUnsafeUrl;
use crate::linter::Linter;
use crate::rule::RuleRegistry;

fn create_linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(NoUnsafeUrl));
    Linter::with_registry(registry)
}

#[test]
fn test_valid_static_href() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<a href="/about">About</a>"#, "test.vue");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_warns_static_javascript_src() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<iframe src="javascript:alert(1)"></iframe>"#, "test.vue");
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_warns_static_obfuscated_javascript_href() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<a href="java&#x0A;script:alert(1)">Link</a>"#,
        "test.vue",
    );
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_warns_static_vbscript_formaction() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<button formaction="vbscript:msgbox(1)">Submit</button>"#,
        "test.vue",
    );
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_warns_static_executable_data_url() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<iframe src="data:text/html;base64,PHNjcmlwdD5hbGVydCgxKTwvc2NyaXB0Pg=="></iframe>"#,
        "test.vue",
    );
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_allows_static_image_data_url() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<img src="data:image/png;base64,iVBORw0KGgo=">"#,
        "test.vue",
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_warns_static_unsafe_srcset_candidate() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<img srcset="/safe.png 1x, javascript:alert(1) 2x">"#,
        "test.vue",
    );
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_valid_router_link() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<router-link :to="{ name: 'profile' }">Profile</router-link>"#,
        "test.vue",
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_warns_dynamic_href() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<a :href="userUrl">Link</a>"#, "test.vue");
    assert_eq!(result.warning_count, 1);
    assert_eq!(
        result.diagnostics[0].message,
        "Dynamic :href binding may be vulnerable to XSS via javascript: protocol"
    );
}

#[test]
fn test_allows_hash_template_href_binding() {
    let linter = create_linter();
    let result = linter.lint_template(r##"<a :href="`#${props.id}`">Link</a>"##, "test.vue");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_allows_hash_concat_href_binding() {
    let linter = create_linter();
    let result = linter.lint_template(r##"<a :href="'#' + props.id">Link</a>"##, "test.vue");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_warns_non_hash_template_href_binding() {
    let linter = create_linter();
    let result = linter.lint_template(r##"<a :href="`${scheme}:${path}`">Link</a>"##, "test.vue");
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_hash_template_only_skips_href() {
    let linter = create_linter();
    let result = linter.lint_template(r##"<iframe :src="`#${props.id}`"></iframe>"##, "test.vue");
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_warns_dynamic_src() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<iframe :src="url"></iframe>"#, "test.vue");
    assert_eq!(result.warning_count, 1);
    assert_eq!(
        result.diagnostics[0].message,
        "Dynamic :src binding may be vulnerable to XSS via javascript: protocol"
    );
}

#[test]
fn test_allows_slot_prop_bindings() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<slot name="item" :data="item" />"#, "test.vue");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_class_binding() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<div :class="classes"></div>"#, "test.vue");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_warns_dynamic_object_data() {
    // `data` is a URL attribute on <object>.
    let linter = create_linter();
    let result = linter.lint_template(r#"<object :data="url"></object>"#, "test.vue");
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_allows_dynamic_data_prop_on_plain_element() {
    // `data` is not a URL attribute on a <div>; it is a plain attribute.
    let linter = create_linter();
    let result = linter.lint_template(r#"<div :data="rows"></div>"#, "test.vue");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_allows_dynamic_data_prop_on_component() {
    // `:data` on a custom component is an ordinary prop, not a URL.
    let linter = create_linter();
    let result = linter.lint_template(r#"<MyComponent :data="rows" />"#, "test.vue");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_allows_dynamic_action_prop_on_component() {
    // `:action` on a component is a prop; it is a URL only on <form>.
    let linter = create_linter();
    let result = linter.lint_template(r#"<MyForm :action="doThing" />"#, "test.vue");
    assert_eq!(result.warning_count, 0);
}
