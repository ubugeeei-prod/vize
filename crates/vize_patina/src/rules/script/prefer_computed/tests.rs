use super::PreferComputed;
use crate::rules::script::ScriptLinter;

fn create_linter() -> ScriptLinter {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(PreferComputed));
    linter
}

#[test]
fn test_warn_watch_with_value_assignment() {
    let linter = create_linter();
    let result = linter.lint(
        r#"
const count = ref(0)
const doubled = ref(0)
watch(count, (val) => {
  doubled.value = val * 2
})
"#,
        0,
    );
    assert_eq!(result.warning_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_valid_watch_without_value_assignment() {
    let linter = create_linter();
    let result = linter.lint(
        r#"
const count = ref(0)
watch(count, (val) => {
  console.log('count changed:', val)
})
"#,
        0,
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_no_watch() {
    let linter = create_linter();
    let result = linter.lint(
        r#"
const count = ref(0)
const doubled = computed(() => count.value * 2)
"#,
        0,
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_no_warn_pattern_inside_string_literal() {
    // The old byte scanner flagged `watch(` + `.value =` even inside a
    // string literal. The AST check must not.
    let linter = create_linter();
    let result = linter.lint(
        r#"
const example = "watch(count, (val) => { doubled.value = val * 2 })"
console.log(example)
"#,
        0,
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_no_warn_pattern_inside_comment() {
    // The old byte scanner flagged the pattern inside comments.
    let linter = create_linter();
    let result = linter.lint(
        r#"
// watch(count, (val) => { doubled.value = val * 2 })
const doubled = computed(() => count.value * 2)
"#,
        0,
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_no_warn_value_comparison_in_callback() {
    // `.value ===` contains the `.value =` substring the old scanner
    // matched; a comparison is not derived-state syncing.
    let linter = create_linter();
    let result = linter.lint(
        r#"
watch(count, (val) => {
  if (doubled.value === val) {
    console.log(val)
  }
})
"#,
        0,
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_no_warn_value_assignment_after_watch_call() {
    // The old scanner matched any `.value =` within 200 bytes after
    // `watch(`, even outside the callback.
    let linter = create_linter();
    let result = linter.lint(
        r#"
watch(count, (val) => {
  console.log(val)
})
doubled.value = 5
"#,
        0,
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_warn_aliased_watch_import() {
    // `import { watch as observe }` never produced the `watch(` byte
    // pattern, so the old scanner missed it entirely.
    let linter = create_linter();
    let result = linter.lint(
        r#"
import { watch as observe } from 'vue'
const doubled = ref(0)
observe(count, (val) => {
  doubled.value = val * 2
})
"#,
        0,
    );
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_warn_watch_with_space_before_paren() {
    // `watch (source, cb)` passed the old fast bailout but the finder only
    // searched for `watch(`, so it was silently missed.
    let linter = create_linter();
    let result = linter.lint(
        r#"
const doubled = ref(0)
watch (count, (val) => {
  doubled.value = val * 2
})
"#,
        0,
    );
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_warn_assignment_beyond_200_bytes() {
    let linter = create_linter();
    let source = format!(
        "const doubled = ref(0)\nwatch(count, (val) => {{ doubled.value = {}val * 2 }})",
        " ".repeat(240)
    );
    assert!(source.find("val * 2").unwrap() > 200);
    assert_eq!(linter.lint(&source, 0).warning_count, 1);
}

#[test]
fn test_warn_function_expression_callback() {
    let linter = create_linter();
    let result = linter.lint(
        r#"
const doubled = ref(0)
watch(count, function (val) {
  doubled.value = val * 2
})
"#,
        0,
    );
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_no_warn_unrelated_function_ending_in_watch() {
    // Identifiers merely ending in `watch` (e.g. `unwatch(...)`) matched
    // the old substring finder.
    let linter = create_linter();
    let result = linter.lint(
        r#"
unwatch(count, (val) => {
  doubled.value = val * 2
})
"#,
        0,
    );
    assert_eq!(result.warning_count, 0);
}
#[test]
fn only_pure_single_unconditional_derived_assignments_warn() {
    let linter = create_linter();
    let prefix = "import { ref, watch } from 'vue'; const source = ref(0); const target = ref(0);";
    for (callback, warnings) in [
        ("(next) => { target.value = next * 2 }", 1),
        ("(next) => target.value = next * 2", 1),
        ("function(next) { target.value = next * 2 }", 1),
        ("() => { target.value = source.value * 2 }", 1),
        ("async (next) => { target.value = next * 2 }", 0),
        ("async (next) => { target.value = await fetch(next) }", 0),
        ("() => { target.value = false }", 0),
        ("(next) => { target.value = null }", 0),
        ("(next) => { target.value = next; refresh() }", 0),
        ("(next) => { if (next) target.value = next }", 0),
        ("(next) => { target.value = refresh(next) }", 0),
        ("(next) => { target.value = target.value + next }", 0),
        ("(next, previous) => { target.value = previous + next }", 0),
        ("(next) => { target.value += next }", 0),
        ("(next) => { target.value.value = next }", 0),
        ("(next) => { target.value = (console.log(next), next) }", 0),
    ] {
        let source = format!("{prefix} watch(source, {callback})");
        assert_eq!(
            linter.lint(&source, 0).warning_count,
            warnings,
            "{callback}"
        );
    }
}

#[test]
fn mutable_unknown_and_shadowed_targets_are_clean() {
    let linter = create_linter();
    for source in [
        "const target = ref(0); watch(source, next => target.value = next); target.value = 3",
        "const target = ref(0); watch(source, next => target.value = next); function edit() { target.value++ }",
        "const target = ref([]); watch(source, next => target.value = next); target.value.push(3)",
        "const target = ref(0); watch(source, next => target.value = next); function edit(target) {}",
        "const target = { value: 0 }; watch(source, next => target.value = next)",
        "watch(source, next => unknown.value = next)",
        "import { ref, watch } from 'other'; const target = ref(0); watch(source, next => target.value = next)",
        "const target = ref(0); function watch() {}; watch(source, next => target.value = next)",
    ] {
        assert_eq!(linter.lint(source, 0).warning_count, 0, "{source}");
    }
}
