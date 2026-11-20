use super::NoDuplicateClass;
use crate::linter::{LintResult, Linter};
use crate::rule::RuleRegistry;

fn create_linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(NoDuplicateClass));
    Linter::with_registry(registry)
}

/// Each finding as `(start, end, message)`.
fn found(result: &LintResult) -> Vec<(u32, u32, &str)> {
    result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.start,
                diagnostic.end,
                diagnostic.message.as_str(),
            )
        })
        .collect()
}

#[test]
fn unique_single_and_missing_classes_are_valid() {
    let linter = create_linter();
    for result in [
        linter.lint_template(r#"<div class="btn primary">x</div>"#, "test.vue"),
        linter.lint_template(r#"<div class="btn">x</div>"#, "test.vue"),
        linter.lint_template(r#"<div>x</div>"#, "test.vue"),
        linter.lint_template(r#"<div class="  btn   primary  ">x</div>"#, "test.vue"),
    ] {
        assert_eq!(found(&result), []);
    }
}

#[test]
fn a_static_value_reports_every_repeated_name_once_on_the_value() {
    let linter = create_linter();
    // `"btn btn primary"`, `"a a b b"` and `"x x x"`.
    assert_eq!(
        found(&linter.lint_template(r#"<div class="btn btn primary">x</div>"#, "test.vue")),
        [(11, 28, "Duplicate class name 'btn'")]
    );
    assert_eq!(
        found(&linter.lint_template(r#"<div class="a a b b">x</div>"#, "test.vue")),
        [(11, 20, "Duplicate class names 'a', 'b'")]
    );
    assert_eq!(
        found(&linter.lint_template(r#"<div class="x x x">y</div>"#, "test.vue")),
        [(11, 18, "Duplicate class name 'x'")]
    );
}

#[test]
fn dynamic_duplicates_report_the_nodes_eslint_plugin_vue_reports() {
    // The start tag for a name the static class repeats, then the `'c c'`
    // string, the `'d d'` key and the `` `e e ${ `` quasi.
    assert_eq!(
        found(&create_linter().lint_template(
            r#"<div class="a" :class="active ? 'a' : 'b'" />
<div :class="['c c', { 'd d': active }]" />
<div :class="`e e ${active ? 'f' : ''}`" />"#,
            "test.vue",
        )),
        [
            (0, 45, "Duplicate class name 'a'"),
            (60, 65, "Duplicate class name 'c'"),
            (69, 74, "Duplicate class name 'd'"),
            (103, 110, "Duplicate class name 'e'"),
        ]
    );
}

#[test]
fn names_two_parts_render_report_the_first_part_holder() {
    // The array, the `+` expression and the array again; an object key that
    // only the static class does not repeat stays clean.
    assert_eq!(
        found(&create_linter().lint_template(
            r#"<div :class="['flex', 'flex', cond ? 'gap' : '']" />
<div :class="'p-1 ' + (cond ? 'm-1' : 'p-1')" />
<div :class="{ 'grid': cond }" class="x" :style="s" />
<div :class="['grid', { grid: cond }]" />"#,
            "test.vue",
        )),
        [
            (13, 48, "Duplicate class name 'flex'"),
            (66, 97, "Duplicate class name 'p-1'"),
            (170, 194, "Duplicate class name 'grid'"),
        ]
    );
}

#[test]
fn conditional_branches_and_template_prefixes_do_not_collide() {
    let linter = create_linter();
    for result in [
        linter.lint_template(
            r#"<div :class="{ 'same': top, 'same': bottom }" />
<div :class="side ? 'same' : 'same'" />"#,
            "test.vue",
        ),
        linter.lint_template(
            r#"<div :class="[`badge--${variant}`, `badge--${size}`]" />"#,
            "test.vue",
        ),
    ] {
        assert_eq!(found(&result), []);
    }
}
