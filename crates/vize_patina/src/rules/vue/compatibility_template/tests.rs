use super::*;
use crate::linter::{LintResult, Linter};
use crate::rule::Rule;
use crate::rule::RuleRegistry;

fn lint(rule: Box<dyn Rule>, template: &str) -> LintResult {
    let mut registry = RuleRegistry::new();
    registry.register(rule);
    Linter::with_registry(registry).lint_template(template, "Component.vue")
}

fn lint_sfc(rule: Box<dyn Rule>, source: &str) -> LintResult {
    let mut registry = RuleRegistry::new();
    registry.register(rule);
    Linter::with_registry(registry).lint_sfc(source, "Component.vue")
}

/// The authored text under each diagnostic, in report order.
fn reported<'a>(rule: Box<dyn Rule>, template: &'a str) -> Vec<&'a str> {
    let mut diagnostics = lint(rule, template).diagnostics;
    diagnostics.sort_by_key(|diagnostic| diagnostic.start);
    diagnostics
        .iter()
        .filter_map(|diagnostic| template.get(diagnostic.start as usize..diagnostic.end as usize))
        .collect()
}

#[test]
fn core_rules_read_retained_expression_ast() {
    let eqeqeq = lint(Box::new(Eqeqeq), "<p>{{ value == null }}</p>");
    assert_eq!(eqeqeq.warning_count, 1);
    let sparse = lint(Box::new(NoSparseArrays), "<p :data=\"[1,,2]\" />");
    assert_eq!(sparse.warning_count, 1);
    let console = lint(Box::new(NoConsole), "<p>{{ console.log(value) }}</p>");
    assert_eq!(console.warning_count, 1);
}

#[test]
fn camelcase_ignores_names_owned_by_an_outer_scope() {
    let result = lint(
        Box::new(Camelcase),
        "<p>{{ MAX_ITEMS }} {{ ITEM_LIMITS.perPage }} {{ user_name }}</p>",
    );

    assert_eq!(result.warning_count, 0, "{:?}", result.diagnostics);
}

#[test]
fn no_implicit_coercion_allows_known_numeric_unary_plus_operands() {
    let result = lint(
        Box::new(NoImplicitCoercion),
        concat!(
            "<button @click=\"move(+1)\" />",
            "<input :step=\"+Number(raw)\" />",
            "<p>{{ +parseInt(raw, 10) }}</p>",
            "<p>{{ +parseFloat(raw) }}</p>",
            "<p>{{ +raw }}</p>",
        ),
    );

    assert_eq!(result.warning_count, 1, "{:?}", result.diagnostics);
}

#[test]
fn template_specific_rules_report_compatibility_cases() {
    let literals = lint(
        Box::new(NoLiteralsInTemplate),
        "<Widget :options=\"{ eager: true }\" />",
    );
    assert_eq!(literals.warning_count, 1);
    let restricted = lint(Box::new(NoRestrictedVBind), "<div :v-test=\"value\" />");
    assert_eq!(restricted.warning_count, 1);
    let model = lint(
        Box::new(PreferVModel),
        "<Widget :model-value=\"value\" @update:model-value=\"value = $event\" />",
    );
    assert_eq!(model.warning_count, 1);
}

#[test]
fn repeated_conditional_components_require_keys() {
    let result = lint(
        Box::new(VIfElseKey),
        "<section><Widget v-if=\"first\" /><Widget v-else /></section>",
    );
    assert_eq!(result.warning_count, 2);
}

#[test]
fn core_rules_visit_every_template_javascript_goal() {
    let source = concat!(
        "<button @[console['event']]=\"run(); console['log'](value == null)\" />",
        "<p v-for=\"{ id = console['log'](value == null) } in rows.filter(row => row == value)\" />",
        "<List #item=\"{ id = console['log'](value == null) }\" />",
    );

    let equality = lint(Box::new(Eqeqeq), source);
    assert_eq!(equality.warning_count, 4, "{:?}", equality.diagnostics);
    let console = lint(Box::new(NoConsole), source);
    assert_eq!(console.warning_count, 4, "{:?}", console.diagnostics);
}

#[test]
fn document_core_rules_visit_style_v_bind_expressions() {
    let source = concat!(
        "<template><p class=\"x\" /></template>",
        "<style scoped>.x { color: v-bind('a == b ? foo[\"bar\"] : 9007199254740993'); ",
        "width: v-bind('console.log(value)'); height: v-bind('({ value: value })'); ",
        "margin: v-bind('({ \"quoted\": value })'); }</style>",
    );

    for rule in [
        Box::new(Eqeqeq) as Box<dyn Rule>,
        Box::new(DotNotation),
        Box::new(NoLossOfPrecision),
    ] {
        let result = lint_sfc(rule, source);
        assert_eq!(result.warning_count, 1, "{:?}", result.diagnostics);
    }
    for rule in [
        Box::new(NoConsole) as Box<dyn Rule>,
        Box::new(ObjectShorthand),
        Box::new(QuoteProps),
    ] {
        let result = lint_sfc(rule, source);
        assert_eq!(result.warning_count, 1, "{:?}", result.diagnostics);
    }
}

#[test]
fn dot_notation_uses_the_core_rules_ascii_identifier_policy() {
    let result = lint(
        Box::new(DotNotation),
        "<p>{{ foo['bar'] }} {{ foo['class'] }} {{ foo['default'] }} {{ foo['gröẞe'] }}</p>",
    );

    assert_eq!(result.warning_count, 3, "{:?}", result.diagnostics);
}

#[test]
fn constant_and_negated_conditions_cover_directives_and_handlers() {
    let constants = lint(
        Box::new(NoConstantCondition),
        concat!(
            "<p v-if=\"true\" />",
            "<p v-show=\"'x'\" />",
            "<p>{{ [] ? a : b }}</p>",
            "<button @click=\"() => { if (true) run() }\" />",
        ),
    );
    assert_eq!(constants.warning_count, 4, "{:?}", constants.diagnostics);

    let negated = lint(
        Box::new(NoNegatedCondition),
        concat!(
            "<p>{{ a !== b ? a : b }}</p>",
            "<button @click=\"() => { if (!a) run(a); else run(b) }\" />",
        ),
    );
    assert_eq!(negated.warning_count, 2, "{:?}", negated.diagnostics);
}

#[test]
fn logical_conditions_only_report_when_truthiness_is_fixed() {
    let result = lint(
        Box::new(NoConstantCondition),
        concat!(
            "<p v-if=\"loading && true\" />",
            "<p v-if=\"loading || false\" />",
            "<p v-if=\"null ?? loading\" />",
            "<p v-if=\"loading || ``\" />",
            "<p v-if=\"loading || true\" />",
            "<p v-if=\"loading && false\" />",
            "<p v-if=\"loading && ``\" />",
        ),
    );

    assert_eq!(result.warning_count, 3, "{:?}", result.diagnostics);
}

#[test]
fn empty_patterns_include_assignment_targets() {
    let result = lint(
        Box::new(NoEmptyPattern),
        "<button @click=\"({}) => run(); ({} = value); [] = rows\" />",
    );

    assert_eq!(result.warning_count, 3, "{:?}", result.diagnostics);
}

#[test]
fn implicit_coercion_matches_string_and_numeric_boundaries() {
    let result = lint(
        Box::new(NoImplicitCoercion),
        concat!(
            "<p>{{ !!a }} {{ ~text.indexOf('x') }} {{ 1 * a }} {{ a - 0 }} {{ - -text }}</p>",
            "<p>{{ `` + a }} {{ '' + 'a' }} {{ '' + String(a) }}</p>",
            "<button @click=\"text += ''\" />",
        ),
    );

    assert_eq!(result.warning_count, 7, "{:?}", result.diagnostics);
}

#[test]
fn concatenation_rules_report_the_same_chain_boundaries_as_eslint() {
    let useless = lint(
        Box::new(NoUselessConcat),
        concat!(
            "<p>{{ text + 'b' + 'c' }}</p>",
            "<p>{{ 'a' + ('b' + text) }}</p>",
            "<p>{{ 'a' + 'b' + 'c' }}</p>",
            "<p :title=\"'a' +\n'b'\" />",
        ),
    );
    assert_eq!(useless.warning_count, 4, "{:?}", useless.diagnostics);

    let preferred = lint(
        Box::new(PreferTemplate),
        concat!(
            "<p>{{ 'a' + text }}</p>",
            "<p>{{ 'a' + 'b' }}</p>",
            "<p>{{ `a${text}` + text }}</p>",
            "<p>{{ 'a' + text + 'c' }}</p>",
            "<p>{{ 'x' + f('a' + b) }}</p>",
        ),
    );
    assert_eq!(preferred.warning_count, 5, "{:?}", preferred.diagnostics);
}

#[test]
fn numeric_precision_compares_the_written_integer_with_the_stored_value() {
    let result = lint(
        Box::new(NoLossOfPrecision),
        concat!(
            "<p>{{ 9007199254740991 }} {{ 1731200000000000 }}</p>",
            "<p>{{ 3.141592653589793 }} {{ 0.30000000000000004 }}</p>",
            "<p>{{ 1.7976931348623157e308 }}</p>",
            "<p>{{ 0x20000000000001 }} {{ 1e400 }} {{ 9007199254740993 }}</p>",
        ),
    );

    assert_eq!(result.warning_count, 3, "{:?}", result.diagnostics);
}

#[test]
fn object_key_rules_avoid_semantics_changing_suggestions() {
    let shorthand = lint(
        Box::new(ObjectShorthand),
        concat!(
            "<p>{{ fn({ a: a, go: function () {}, [key]: function () {} }) }}</p>",
            "<p>{{ fn({ [key]: key, ['a']: a, __proto__: __proto__, a: a! }) }}</p>",
            "<p>{{ fn({ get value() {}, set value(next) {}, recurse: function recurse() {}, ",
            "plain: function () {} }) }}</p>",
        ),
    );
    assert_eq!(shorthand.warning_count, 4, "{:?}", shorthand.diagnostics);

    let quotes = lint(
        Box::new(QuoteProps),
        concat!(
            "<p>{{ fn({ 'a': a, 'class': a, '0': a, ['b']: a, 'go'() {} }) }}</p>",
            "<p>{{ rows.map(({ 'id': id }) => id) }}</p>",
            "<p>{{ fn({ '01': a, '-1': a, '+1': a, '.5': a, '5.': a, ",
            "'1e3': a, '1.0': a, '9007199254740992': a, '9007199254740991': a }) }}</p>",
        ),
    );
    assert_eq!(quotes.warning_count, 6, "{:?}", quotes.diagnostics);
}

#[test]
fn location_rules_report_the_operator_or_each_hole() {
    let equality = reported(
        Box::new(Eqeqeq),
        "<p>{{ (a) /* == */ != (b) }} {{ a == b == c }}</p>",
    );
    assert_eq!(equality, ["!=", "==", "=="]);
    let concat = reported(
        Box::new(NoUselessConcat),
        "<p>{{ 'a' +\n'b' + 'c' }} {{ 'x' /* + */ + ('y' + z) }}</p>",
    );
    assert_eq!(concat, ["+", "+"]);
    let sparse = reported(
        Box::new(NoSparseArrays),
        "<p>{{ [a, ,\n b, /* , */ ,] }} {{ [,,] }} {{ [(a), , ...b] }}</p>",
    );
    assert_eq!(sparse, [",", ",", ",", ",", ","]);
}

#[test]
fn literal_keys_match_the_core_rule_values() {
    let quotes = reported(
        Box::new(QuoteProps),
        "<p>{{ use({ '1.5': a, '0.25': a, '1e+21': a, '1e3': a, '.5': a, 'café': a }) }}</p>",
    );
    assert_eq!(quotes, ["'1.5'", "'0.25'", "'1e+21'", "'café'"]);
    let dots = reported(
        Box::new(DotNotation),
        "<p>{{ map[true] }} {{ map[false] }} {{ map[null] }} {{ map[(\"a\")] }} {{ map[1] }}</p>",
    );
    assert_eq!(dots, ["true", "false", "null", "\"a\""]);
    let shorthand = lint(
        Box::new(ObjectShorthand),
        "<p>{{ use({ a: (a), run: (function () {}), __proto__: function () {} }) }}</p>",
    );
    assert_eq!(shorthand.warning_count, 2, "{:?}", shorthand.diagnostics);
}

#[test]
fn implicit_coercion_skips_fractions_chains_and_numeric_negations() {
    let result = reported(
        Box::new(NoImplicitCoercion),
        concat!(
            "<p>{{ a * 1 / 4 }} {{ 2 * a * 1 }} {{ - -1 }} {{ - -Number(a) }} {{ (a * 1) / 4 }}</p>",
            "<p>{{ ~a?.indexOf('x') }} {{ ~a.lastIndexOf('x') }}</p>",
        ),
    );
    assert_eq!(result, ["a * 1", "~a?.indexOf('x')", "~a.lastIndexOf('x')"]);
}

#[test]
fn numeric_precision_compares_significant_digits_in_every_base() {
    let result = lint(
        Box::new(NoLossOfPrecision),
        concat!(
            "<p>{{ 0.12345678901234567890 }} {{ 900719.9254740994 }} {{ 9.007199254740993e15 }}</p>",
            "<p>{{ 0b100000000000000000000000000000000000000000000000000001 }}</p>",
            "<p>{{ 0o400000000000000001 }} {{ 1e-350 }}</p>",
            "<p>{{ 12300000000000000000000000 }} {{ 1_230000000_00000000_00000_000 }}</p>",
        ),
    );
    assert_eq!(result.warning_count, 6, "{:?}", result.diagnostics);
}

#[test]
fn camelcase_reports_owned_declarations_keys_and_assigned_members() {
    let result = lint(
        Box::new(Camelcase),
        concat!(
            "<p>{{ user_name }}</p>",
            "<p v-for=\"row_label in rows\">{{ row_label }}</p>",
            "<p :class=\"{ is_active: total > 0 }\" />",
            "<button @click=\"totals.sub_total = 1\" />",
            "<button @click=\"const next_total = total + 1; total = next_total\" />",
        ),
    );

    assert_eq!(result.warning_count, 4, "{:?}", result.diagnostics);
}
