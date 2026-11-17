use super::super::NoNegatedCondition;
use super::reported;

#[test]
fn negated_condition_reports_the_whole_conditional_or_if_node() {
    let found = reported(
        Box::new(NoNegatedCondition),
        concat!(
            "<p>{{ a !== b ? a : b }}</p>",
            "<p :title=\"(!a) ? 'x' : 'y'\" />",
            "<p :title=\"!a\n  ? 'multi'\n  : 'line'\" />",
            "<button @click=\"if (a != b) run(a); else run(b)\" />",
            "<button @click=\"() => { if (!a) { run(a) } else { run(b) } }\" />",
        ),
    );
    assert_eq!(
        found,
        [
            "a !== b ? a : b",
            "(!a) ? 'x' : 'y'",
            "!a\n  ? 'multi'\n  : 'line'",
            "if (a != b) run(a); else run(b)",
            "if (!a) { run(a) } else { run(b) }",
        ]
    );
}

#[test]
fn negated_condition_skips_else_if_chains_and_type_wrappers() {
    let found = reported(
        Box::new(NoNegatedCondition),
        concat!(
            "<button @click=\"if (!a) run(a); else if (b) run(b)\" />",
            "<button @click=\"if (!a) run(a)\" />",
            "<p>{{ a ? b : c }}</p>",
            "<p>{{ (!a as boolean) ? b : c }}</p>",
            "<p>{{ !(a as boolean) ? b : c }}</p>",
        ),
    );
    assert_eq!(found, ["!(a as boolean) ? b : c"]);
}
