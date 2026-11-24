use crate::linter::Linter;
use crate::markup::SameNameSpelling;
use crate::rule::RuleRegistry;
use crate::rules::vue::{VBindStyle, VBindStyleOption};

const EXPECTED: &str = "Expected same-name shorthand.";
const UNEXPECTED: &str = "Unexpected same-name shorthand.";
const PREFER_SHORTHAND: &str = "Prefer shorthand `:` over `v-bind:`";
const PREFER_LONGFORM: &str = "Prefer `v-bind:` over shorthand `:`";
const PREFER_LONGFORM_PROP: &str = "Expected 'v-bind:' instead of '.'.";

fn linter(style: VBindStyleOption, required: SameNameSpelling) -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(VBindStyle { style }.requiring_same_name(required));
    Linter::with_registry(registry)
}

/// Every message in report order, and the source with every fix applied.
fn observe(linter: &Linter, source: &str) -> (Vec<std::string::String>, std::string::String) {
    let result = linter.lint_template(source, "test.vue");
    let mut edits = result
        .diagnostics
        .iter()
        .filter_map(|diagnostic| diagnostic.fix.as_ref())
        .flat_map(|fix| fix.edits.iter())
        .collect::<Vec<_>>();
    edits.sort_by_key(|edit| std::cmp::Reverse((edit.start, edit.end)));
    let mut fixed = source.to_owned();
    for edit in edits {
        fixed.replace_range(edit.start as usize..edit.end as usize, &edit.new_text);
    }
    let messages = result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.to_string())
        .collect();
    (messages, fixed)
}

fn assert_fixes(linter: &Linter, cases: &[(&str, &[&str], &str)]) {
    for (source, messages, expected) in cases {
        let (reported, fixed) = observe(linter, source);
        assert_eq!(reported, *messages, "{source}");
        assert_eq!(fixed, *expected, "{source}");
        let (remaining, unchanged) = observe(linter, &fixed);
        assert_eq!(remaining, Vec::<std::string::String>::new(), "{fixed}");
        assert_eq!(unchanged, fixed);
    }
}

fn assert_quiet(linter: &Linter, sources: &[&str]) {
    for source in sources {
        let (reported, fixed) = observe(linter, source);
        assert_eq!(reported, Vec::<std::string::String>::new(), "{source}");
        assert_eq!(fixed, *source);
    }
}

#[test]
fn always_reports_a_written_same_name_value_and_removes_it() {
    assert_fixes(
        &linter(VBindStyleOption::Shorthand, SameNameSpelling::Shorthand),
        &[
            (r#"<div :foo="foo" />"#, &[EXPECTED], r#"<div :foo />"#),
            (r#"<div :foo='foo' />"#, &[EXPECTED], r#"<div :foo />"#),
            (r#"<div :foo=foo />"#, &[EXPECTED], r#"<div :foo />"#),
            (r#"<div :foo = " foo " />"#, &[EXPECTED], r#"<div :foo />"#),
            (
                r#"<div :foo-bar="fooBar" />"#,
                &[EXPECTED],
                r#"<div :foo-bar />"#,
            ),
            (
                r#"<div :fooBar="fooBar" />"#,
                &[EXPECTED],
                r#"<div :fooBar />"#,
            ),
            (
                r#"<div :foo_bar="foo_bar" />"#,
                &[EXPECTED],
                r#"<div :foo_bar />"#,
            ),
            (r#"<div :$foo="$foo" />"#, &[EXPECTED], r#"<div :$foo />"#),
            (
                r#"<div :foo.prop="foo" />"#,
                &[EXPECTED],
                r#"<div :foo.prop />"#,
            ),
            (
                r#"<div :foo.attr="foo" />"#,
                &[EXPECTED],
                r#"<div :foo.attr />"#,
            ),
            (r#"<div .foo="foo" />"#, &[EXPECTED], r#"<div .foo />"#),
            (
                r#"<Comp :a="a" :b="c" :d="d" />"#,
                &[EXPECTED, EXPECTED],
                r#"<Comp :a :b="c" :d />"#,
            ),
            (
                r#"<div v-bind:foo="foo" />"#,
                &[EXPECTED, PREFER_SHORTHAND],
                r#"<div :foo />"#,
            ),
        ],
    );
}

#[test]
fn always_leaves_every_other_binding_alone() {
    assert_quiet(
        &linter(VBindStyleOption::Shorthand, SameNameSpelling::Shorthand),
        &[
            r#"<div :foo />"#,
            r#"<div :foo="bar" />"#,
            r#"<div :foo="Foo" />"#,
            r#"<div :foo="foo.bar" />"#,
            r#"<div :foo="foo()" />"#,
            r#"<div :foo="(foo)" />"#,
            r#"<div :foo="'foo'" />"#,
            r#"<div :foo="" />"#,
            r#"<div :foo-bar="foo-bar" />"#,
            r#"<div :foo-bar="foobar" />"#,
            r#"<div :fooBar="foo-bar" />"#,
            r#"<div :1="1" />"#,
            r#"<div :[foo]="foo" />"#,
            r#"<div v-bind="foo" />"#,
            r#"<div foo="foo" />"#,
            r#"<div @foo="foo" />"#,
            r#"<Comp v-model:foo="foo" />"#,
        ],
    );
}

#[test]
fn never_reports_the_shorthand_and_writes_the_camelized_value() {
    assert_fixes(
        &linter(VBindStyleOption::Shorthand, SameNameSpelling::Explicit),
        &[
            (r#"<div :foo />"#, &[UNEXPECTED], r#"<div :foo="foo" />"#),
            (
                r#"<div :foo></div>"#,
                &[UNEXPECTED],
                r#"<div :foo="foo"></div>"#,
            ),
            (
                r#"<div :foo-bar />"#,
                &[UNEXPECTED],
                r#"<div :foo-bar="fooBar" />"#,
            ),
            (
                r#"<div :fooBar />"#,
                &[UNEXPECTED],
                r#"<div :fooBar="fooBar" />"#,
            ),
            (
                r#"<div :foo.prop />"#,
                &[UNEXPECTED],
                r#"<div :foo.prop="foo" />"#,
            ),
            (r#"<div .foo />"#, &[UNEXPECTED], r#"<div .foo="foo" />"#),
            (
                r#"<Comp :a :b="c" :d />"#,
                &[UNEXPECTED, UNEXPECTED],
                r#"<Comp :a="a" :b="c" :d="d" />"#,
            ),
            (
                r#"<div v-bind:foo />"#,
                &[UNEXPECTED, PREFER_SHORTHAND],
                r#"<div :foo="foo" />"#,
            ),
        ],
    );
}

#[test]
fn never_leaves_written_values_and_non_bindings_alone() {
    assert_quiet(
        &linter(VBindStyleOption::Shorthand, SameNameSpelling::Explicit),
        &[
            r#"<div :foo="foo" />"#,
            r#"<div :foo="bar" />"#,
            r#"<div :foo="" />"#,
            r#"<div :[foo]="foo" />"#,
            r#"<div v-bind="foo" />"#,
            r#"<div foo />"#,
            r#"<div @foo />"#,
        ],
    );
}

#[test]
fn the_longform_style_composes_with_either_policy() {
    assert_fixes(
        &linter(VBindStyleOption::Longform, SameNameSpelling::Shorthand),
        &[
            (
                r#"<div v-bind:foo="foo" />"#,
                &[EXPECTED],
                r#"<div v-bind:foo />"#,
            ),
            (
                r#"<div :foo="foo" />"#,
                &[EXPECTED, PREFER_LONGFORM],
                r#"<div v-bind:foo />"#,
            ),
            (
                r#"<div .foo="foo" />"#,
                &[EXPECTED, PREFER_LONGFORM_PROP],
                r#"<div v-bind:foo.prop />"#,
            ),
        ],
    );
    assert_fixes(
        &linter(VBindStyleOption::Longform, SameNameSpelling::Explicit),
        &[
            (
                r#"<div v-bind:foo-bar />"#,
                &[UNEXPECTED],
                r#"<div v-bind:foo-bar="fooBar" />"#,
            ),
            (
                r#"<div :foo />"#,
                &[UNEXPECTED, PREFER_LONGFORM],
                r#"<div v-bind:foo="foo" />"#,
            ),
            (
                r#"<div .foo />"#,
                &[UNEXPECTED, PREFER_LONGFORM_PROP],
                r#"<div v-bind:foo.prop="foo" />"#,
            ),
        ],
    );
}

#[test]
fn modified_and_property_bindings_are_same_name_in_both_spellings() {
    let always = linter(VBindStyleOption::Shorthand, SameNameSpelling::Shorthand);
    let result = always.lint_template(
        r#"<div :foo-bar.camel="fooBar" .baz="baz" v-bind:qux="qux" :[dyn]="dyn" />"#,
        "test.vue",
    );
    assert_eq!(
        result
            .diagnostics
            .iter()
            .map(|diagnostic| (
                diagnostic.message.as_str(),
                diagnostic.start,
                diagnostic.end
            ))
            .collect::<Vec<_>>(),
        [
            (EXPECTED, 5, 28),
            (EXPECTED, 29, 39),
            (EXPECTED, 40, 56),
            (PREFER_SHORTHAND, 40, 56),
        ]
    );

    let never = linter(VBindStyleOption::Shorthand, SameNameSpelling::Explicit);
    let result = never.lint_template(r#"<div :foo-bar.camel .baz v-bind:qux />"#, "test.vue");
    assert_eq!(
        result
            .diagnostics
            .iter()
            .map(|diagnostic| (
                diagnostic.message.as_str(),
                diagnostic.start,
                diagnostic.end
            ))
            .collect::<Vec<_>>(),
        [
            (UNEXPECTED, 5, 19),
            (UNEXPECTED, 20, 24),
            (UNEXPECTED, 25, 35),
            (PREFER_SHORTHAND, 25, 35),
        ]
    );
}
