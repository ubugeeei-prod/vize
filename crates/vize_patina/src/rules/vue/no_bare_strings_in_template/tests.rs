use super::NoBareStringsInTemplate;
use crate::linter::Linter;
use crate::rule::RuleRegistry;

fn create_linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(NoBareStringsInTemplate));
    Linter::with_registry(registry)
}

#[test]
fn reports_bare_text_content() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<div>hello</div>"#, "App.vue");
    assert_eq!(result.warning_count, 1);
    insta::with_settings!({ snapshot_path => "../snapshots" }, {
        insta::assert_debug_snapshot!(result.diagnostics);
    });
}

#[test]
fn reports_bare_text_in_nested_element() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<div><span>Save</span></div>"#, "App.vue");
    assert_eq!(result.warning_count, 1);
}

#[test]
fn reports_bare_text_at_the_template_root() {
    let result = create_linter().lint_template(
        "Base Empty <CompSwitch v-model:value=\"loadData\" />",
        "App.vue",
    );
    assert_eq!(result.warning_count, 1, "{:?}", result.diagnostics);
}

#[test]
fn reports_non_ascii_bare_text() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<div>こんにちは</div>"#, "App.vue");
    assert_eq!(result.warning_count, 1);
}

#[test]
fn reports_bare_alt_attribute() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<img alt="a cat" />"#, "App.vue");
    assert_eq!(result.warning_count, 1);
}

#[test]
fn reports_bare_title_attribute() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<button title="Close">x</button>"#, "App.vue");
    // Both the title attribute and the "x"-free text? "x" is alphabetic, so
    // the button text "x" also reports. Expect 2.
    assert_eq!(result.warning_count, 2);
}

#[test]
fn reports_bare_placeholder_attribute() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<input placeholder="Search" />"#, "App.vue");
    assert_eq!(result.warning_count, 1);
}

#[test]
fn reports_bare_aria_label_attribute() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<div aria-label="Menu"></div>"#, "App.vue");
    assert_eq!(result.warning_count, 1);
}

#[test]
fn allows_interpolation() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<div>{{ $t('hello') }}</div>"#, "App.vue");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn allows_bound_attribute() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<img :alt="$t('cat')" />"#, "App.vue");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn allows_whitespace_only_text() {
    let linter = create_linter();
    let result = linter.lint_template("<div>   </div>", "App.vue");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn allows_punctuation_and_symbol_only_text() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<div>-</div>"#, "App.vue");
    assert_eq!(result.warning_count, 0);
    let result = linter.lint_template(r#"<span>|</span>"#, "App.vue");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn reports_number_only_text() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<div>123</div>"#, "App.vue");
    assert_eq!(result.warning_count, 1);
}

#[test]
fn limits_alt_and_placeholder_to_the_upstream_default_tags() {
    let linter = create_linter();
    let result = linter.lint_template(
        r#"<div alt="copy" placeholder="copy"></div><img alt="copy"><input placeholder="copy">"#,
        "App.vue",
    );
    assert_eq!(result.warning_count, 2);
}

#[test]
fn reports_the_static_attribute_value_range() {
    let linter = create_linter();
    let source = r#"<img alt="a cat">"#;
    let result = linter.lint_template(source, "App.vue");
    assert_eq!(result.warning_count, 1);
    let start = source.find('"').unwrap() as u32;
    assert_eq!(result.diagnostics[0].start, start);
    assert_eq!(result.diagnostics[0].end, source.len() as u32 - 1);
}

#[test]
fn reports_static_v_text_values() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<p v-text="'Save'"></p>"#, "App.vue");
    assert_eq!(result.warning_count, 1);
}

#[test]
fn allows_non_target_attribute() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<div class="container"></div>"#, "App.vue");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn allows_id_attribute() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<input type="text" name="q" />"#, "App.vue");
    assert_eq!(result.warning_count, 0);
}
