use super::NoUseComputedPropertyLikeMethod;
use crate::rules::script::ScriptLinter;

/// Lint `source` with only this rule enabled and return the error count.
fn count(source: &str) -> usize {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(NoUseComputedPropertyLikeMethod));
    linter.lint(source, 0).error_count
}

// Each case is `(source, expected_error_count)`.
const CASES: &[(&str, usize)] = &[
    // Valid: read the computed value (no call parentheses).
    (
        "export default { computed: { c() { return 1 } }, methods: { m() { return this.c } } }",
        0,
    ),
    // Valid: no Options API `computed` to resolve (Composition API).
    (
        "import { computed } from 'vue'\nconst d = computed(() => c.value)",
        0,
    ),
    // Valid: `c` is not declared as a computed, so calling it is allowed.
    ("export default { methods: { m() { return this.c() } } }", 0),
    // Valid: the called name (`other`) is not a computed; `c` is never called.
    (
        "export default { computed: { c() { return 1 } }, methods: { m() { return this.x() } } }",
        0,
    ),
    // Valid: receiver is `other`, not `this`.
    (
        "export default { computed: { c() { return 1 } }, methods: { m() { return other.c() } } }",
        0,
    ),
    // Valid: a non-arrow nested function rebinds `this`.
    (
        "export default { computed: { c() { return 1 } }, methods: { m() { return [1].map(function () { return this.c() }) } } }",
        0,
    ),
    // Valid: spread computed members are not statically known.
    (
        "export default { computed: { ...mapGetters(['c']) }, methods: { m() { return this.c() } } }",
        0,
    ),
    // Invalid: computed called from a method.
    (
        "export default { computed: { c() { return 1 } }, methods: { m() { return this.c() } } }",
        1,
    ),
    // Invalid: computed called from another computed getter.
    (
        "export default { computed: { a() { return 1 }, b() { return this.a() } } }",
        1,
    ),
    // Invalid: string-keyed computed, static call.
    (
        "export default { computed: { \"c\"() { return 1 } }, methods: { m() { return this.c() } } }",
        1,
    ),
    // Invalid: resolved through `defineComponent(...)`.
    (
        "import { defineComponent } from 'vue'\nexport default defineComponent({ computed: { c() { return 1 } }, methods: { m() { return this.c() } } })",
        1,
    ),
    // Invalid: resolved through an identifier binding.
    (
        "const o = { computed: { c() { return 1 } }, methods: { m() { return this.c() } } }\nexport default o",
        1,
    ),
    // Invalid: arrow callback keeps the lexical (component) `this`.
    (
        "export default { computed: { c() { return 1 } }, methods: { m() { return [1].map(() => this.c()) } } }",
        1,
    ),
    // Invalid: two distinct computed calls each report.
    (
        "export default { computed: { a() { return 1 }, b() { return 2 } }, methods: { m() { return this.a() + this.b() } } }",
        2,
    ),
];

#[test]
fn test_cases() {
    for (source, expected) in CASES {
        assert_eq!(count(source), *expected, "source: {source}");
    }
}

#[test]
fn test_diagnostic_shape() {
    let source = "export default { computed: { fullName() { return this.a } }, \
             methods: { greet() { return this.fullName() } } }";
    let result = {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(NoUseComputedPropertyLikeMethod));
        linter.lint(source, 0)
    };
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}
