use super::NoRefAsOperand;
use crate::rules::script::{ScriptLintResult, ScriptLinter};

fn lint(source: &str) -> ScriptLintResult {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(NoRefAsOperand));
    linter.lint(source, 0)
}

// --- Valid: proper `.value` access, or passing the ref itself ---

#[test]
fn test_valid_value_access() {
    assert_eq!(lint("const count = ref(0)\ncount.value++").error_count, 0);
    assert_eq!(
        lint("const count = ref(0)\nconsole.log(count.value + 1)").error_count,
        0
    );
    assert_eq!(
        lint("const count = ref(0)\nif (count.value) {}").error_count,
        0
    );
}

#[test]
fn test_valid_passing_ref_itself() {
    // Passing the ref as a call argument (not an operand) is fine.
    assert_eq!(
        lint("const count = ref(0)\nwatch(count, () => {})").error_count,
        0
    );
    assert_eq!(lint("const count = ref(0)\nuseFoo(count)").error_count, 0);
    assert_eq!(
        lint("const count = ref(0)\nreturn { count }").error_count,
        0
    );
}

#[test]
fn test_valid_plain_reassignment() {
    // `count = x` rebinds the variable; only compound assignment reads it.
    assert_eq!(lint("let count = ref(0)\ncount = ref(1)").error_count, 0);
}

#[test]
fn test_valid_non_ref_binding() {
    // Not initialized from a ref factory: never tracked.
    assert_eq!(lint("const count = 0\ncount + 1").error_count, 0);
    assert_eq!(
        lint("const state = reactive({ n: 0 })\nstate.n + 1").error_count,
        0
    );
}

#[test]
fn test_valid_typeof() {
    // `typeof` inspects the binding, not its value.
    assert_eq!(
        lint("const count = ref(0)\nif (typeof count === 'object') {}").error_count,
        0
    );
}

// --- Invalid: update / unary / binary / logical / conditional / condition ---

#[test]
fn test_invalid_update_expression() {
    let result = lint("let count = ref(0)\ncount++");
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_prefix_update() {
    assert_eq!(lint("let count = ref(0)\n--count").error_count, 1);
}

#[test]
fn test_invalid_binary_operand() {
    assert_eq!(
        lint("const count = ref(0)\nconsole.log(count + 1)").error_count,
        1
    );
    assert_eq!(
        lint("const count = ref(0)\nconst x = 1 + count").error_count,
        1
    );
    assert_eq!(
        lint("const count = ref(0)\nconst x = count === 0").error_count,
        1
    );
    assert_eq!(
        lint("const count = ref(0)\nconst x = count < 5").error_count,
        1
    );
}

#[test]
fn test_invalid_unary_operand() {
    assert_eq!(
        lint("const flag = ref(false)\nconst x = !flag").error_count,
        1
    );
    assert_eq!(
        lint("const count = ref(0)\nconst x = -count").error_count,
        1
    );
}

#[test]
fn test_invalid_logical_operand() {
    assert_eq!(
        lint("const flag = ref(false)\nconst x = flag && other").error_count,
        1
    );
    assert_eq!(
        lint("const count = ref(0)\nconst x = other || count").error_count,
        1
    );
    assert_eq!(
        lint("const count = ref(0)\nconst x = count ?? 1").error_count,
        1
    );
}

#[test]
fn test_invalid_conditional_test() {
    assert_eq!(
        lint("const flag = ref(false)\nconst x = flag ? 1 : 2").error_count,
        1
    );
}

#[test]
fn test_invalid_if_condition() {
    let result = lint("const flag = ref(false)\nif (flag) {}");
    assert_eq!(result.error_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_while_condition() {
    assert_eq!(
        lint("const flag = ref(true)\nwhile (flag) {}").error_count,
        1
    );
    assert_eq!(
        lint("const flag = ref(true)\ndo {} while (flag)").error_count,
        1
    );
}

#[test]
fn test_invalid_for_condition() {
    assert_eq!(
        lint("const flag = ref(true)\nfor (;flag;) {}").error_count,
        1
    );
}

#[test]
fn test_invalid_compound_assignment() {
    assert_eq!(lint("let count = ref(0)\ncount += 1").error_count, 1);
    assert_eq!(lint("let count = ref(0)\ncount *= 2").error_count, 1);
}

// --- Factory variants ---

#[test]
fn test_invalid_other_factories() {
    assert_eq!(lint("const c = computed(() => 1)\nc + 1").error_count, 1);
    assert_eq!(lint("const c = shallowRef(0)\nc + 1").error_count, 1);
    assert_eq!(lint("const c = toRef(props, 'x')\nc + 1").error_count, 1);
    assert_eq!(
        lint("const c = customRef(() => ({}))\nc + 1").error_count,
        1
    );
}

#[test]
fn test_factory_with_ts_cast() {
    assert_eq!(
        lint("const count = ref(0) as Ref<number>\ncount + 1").error_count,
        1
    );
}

// --- Scoping / soundness ---

#[test]
fn test_shadowing_param_not_flagged() {
    // The outer `count` is a ref, but the inner `count` parameter shadows it
    // and is not a ref, so `count + 1` inside `f` must not be flagged.
    let source = "const count = ref(0)\nfunction f(count) { return count + 1 }";
    assert_eq!(lint(source).error_count, 0);
}

#[test]
fn test_shadowing_local_not_flagged() {
    let source = "const count = ref(0)\nfunction f() { const count = 1; return count + 1 }";
    assert_eq!(lint(source).error_count, 0);
}

#[test]
fn test_ref_used_inside_nested_function() {
    // A top-level ref used as an operand inside a nested function is flagged.
    let source = "const count = ref(0)\nfunction f() { return count + 1 }";
    assert_eq!(lint(source).error_count, 1);
}

#[test]
fn test_multiple_invalid_reported() {
    let source = "const a = ref(0)\nconst b = ref(0)\nconst x = a + b";
    assert_eq!(lint(source).error_count, 2);
}

#[test]
fn test_no_refs_no_reports() {
    assert_eq!(
        lint("const x = 1\nconst y = x + 1\nif (y) {}").error_count,
        0
    );
}
