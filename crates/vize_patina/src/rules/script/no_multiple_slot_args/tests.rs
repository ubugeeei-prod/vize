use crate::rules::script::{NoMultipleSlotArgs, ScriptLintResult, ScriptLinter};

fn lint(source: &str) -> ScriptLintResult {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(NoMultipleSlotArgs));
    linter.lint(source, 0)
}

// --- valid ----------------------------------------------------------------

#[test]
fn test_valid_single_object_arg() {
    let result = lint("slots.default({ foo, bar })");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_single_arg() {
    let result = lint("slots.default(slotProps)");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_zero_args() {
    let result = lint("slots.default()");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_optional_call_single_arg() {
    // The idiomatic optional-call form with a single argument is fine.
    let result = lint("slots.default?.(slotProps)");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_dollar_slots_single_arg() {
    let result = lint("$slots.header(props)");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_use_slots_single_arg() {
    let result = lint("useSlots().default(props)");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_non_slot_object_multiple_args() {
    // An unrelated object called with two arguments must not be reported: the
    // rule only fires for recognised slots sources.
    let source = r#"
foo.bar(a, b)
emit('change', payload)
console.log(a, b)
Math.max(a, b)
"#;
    let result = lint(source);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_bare_slots_call_not_a_member() {
    // `slots('x', 'y')` is not a member access on `slots`, so it is not a slot
    // invocation and is left alone.
    let result = lint("slots(a, b)");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_dynamic_computed_slot_name() {
    // A dynamic (non-literal) computed key is treated conservatively as not a
    // statically-known slot access.
    let result = lint("slots[name](a, b)");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_substring_identifier_not_matched() {
    // An object whose name merely contains "slots" must not match.
    let result = lint("mySlots.default(a, b)");
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_this_unrelated_member() {
    // `this.foo.bar(a, b)` is not a slots access.
    let result = lint("this.foo.bar(a, b)");
    assert_eq!(result.warning_count, 0);
}

// --- invalid: multiple positional args ------------------------------------

#[test]
fn test_invalid_slots_default_two_args() {
    let result = lint("slots.default(foo, bar)");
    assert_eq!(result.warning_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_slots_default_three_args() {
    let result = lint("slots.default(a, b, c)");
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_invalid_dollar_slots_two_args() {
    let result = lint("$slots.header(a, b)");
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_invalid_dollar_scoped_slots_two_args() {
    let result = lint("$scopedSlots.item(a, b)");
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_invalid_this_scoped_slots_two_args() {
    // Options API render function form.
    let result = lint("this.$scopedSlots.foo(a, b)");
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_invalid_this_slots_two_args() {
    let result = lint("this.$slots.foo(a, b)");
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_invalid_use_slots_two_args() {
    let result = lint("useSlots().default(a, b)");
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_invalid_computed_string_slot_name() {
    let result = lint("slots['default'](a, b)");
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_invalid_optional_call_two_args() {
    let result = lint("slots.default?.(a, b)");
    assert_eq!(result.warning_count, 1);
}

// --- invalid: spread argument ---------------------------------------------

#[test]
fn test_invalid_spread_only_arg() {
    let result = lint("slots.default(...args)");
    assert_eq!(result.warning_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_spread_first_of_many() {
    let result = lint("slots.default(...args, extra)");
    assert_eq!(result.warning_count, 1);
}

// --- offsets & multiplicity -----------------------------------------------

#[test]
fn test_invalid_multiple_occurrences() {
    let source = r#"
slots.default(a, b)
$slots.header(x, y)
"#;
    let result = lint(source);
    assert_eq!(result.warning_count, 2);
}

#[test]
fn test_reports_call_span() {
    let source = "const vnode = slots.default(a, b)";
    let result = lint(source);
    assert_eq!(result.diagnostics.len(), 1);
    let diag = &result.diagnostics[0];
    let start = source.find("slots.default(a, b)").unwrap();
    assert_eq!(diag.start, start as u32);
    assert_eq!(diag.end, source.len() as u32);
}

#[test]
fn test_offset_applied() {
    // A non-zero block offset must shift the reported span.
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(NoMultipleSlotArgs));
    let result = linter.lint("slots.default(a, b)", 100);
    assert_eq!(result.warning_count, 1);
    assert_eq!(result.diagnostics[0].start, 100);
}

#[test]
fn test_invalid_nested_in_function() {
    // The call is reported wherever it appears, including inside a render fn.
    let source = r#"
export default {
  setup(_, { slots }) {
    return () => slots.default(a, b)
  }
}
"#;
    let result = lint(source);
    assert_eq!(result.warning_count, 1);
}
