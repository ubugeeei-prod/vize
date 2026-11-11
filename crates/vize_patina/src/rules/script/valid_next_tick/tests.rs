use super::ValidNextTick;
use crate::rules::script::ScriptLinter;

fn create_linter() -> ScriptLinter {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(ValidNextTick));
    linter
}

#[test]
fn test_valid_awaited() {
    let result = create_linter().lint(
        "import { nextTick } from 'vue'\nasync function f() { await nextTick() }",
        0,
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_then_chained() {
    let result = create_linter().lint(
        "import { nextTick } from 'vue'\nnextTick().then(() => focusInput())",
        0,
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_callback_argument() {
    let result = create_linter().lint(
        "import { nextTick } from 'vue'\nnextTick(() => focusInput())",
        0,
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_instance_callback() {
    let result = create_linter().lint("this.$nextTick(() => focusInput())", 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_assigned_result() {
    // Assigning the promise is a use; this is a declaration, not a bare
    // expression statement.
    let result = create_linter().lint("import { nextTick } from 'vue'\nconst p = nextTick()", 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_returned_result() {
    let result = create_linter().lint(
        "import { nextTick } from 'vue'\nfunction f() { return nextTick() }",
        0,
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_arrow_expression_return() {
    let result = create_linter().lint(
        "import { nextTick as vueNextTick } from 'vue';\nexport const useTick = () => { const nextTick = () => vueNextTick(); return { nextTick }; };",
        0,
    );
    assert_eq!(result.warning_count, 0, "{:#?}", result.diagnostics);
}

#[test]
fn test_valid_parenthesized_arrow_expression_return() {
    let result = create_linter().lint(
        "import { nextTick } from 'vue'; const tick = () => (nextTick());",
        0,
    );
    assert_eq!(result.warning_count, 0, "{:#?}", result.diagnostics);
}

#[test]
fn test_invalid_arrow_block_bare_call() {
    let result = create_linter().lint(
        "import { nextTick } from 'vue'; const tick = () => { nextTick(); };",
        0,
    );
    assert_eq!(result.warning_count, 1, "{:#?}", result.diagnostics);
}

#[test]
fn test_invalid_arrow_expression_void_call() {
    let result = create_linter().lint(
        "import { nextTick } from 'vue'; const tick = () => void nextTick();",
        0,
    );
    assert_eq!(result.warning_count, 1, "{:#?}", result.diagnostics);
}

#[test]
fn test_nested_arrow_restores_statement_context() {
    let result = create_linter().lint(
        "import { nextTick } from 'vue'; const tick = () => { const inner = () => nextTick(); nextTick(); return inner; };",
        0,
    );
    assert_eq!(result.warning_count, 1, "{:#?}", result.diagnostics);
}

#[test]
fn test_invalid_bare_call() {
    let result = create_linter().lint("import { nextTick } from 'vue'\nnextTick()", 0);
    assert_eq!(result.warning_count, 1);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn test_invalid_bare_instance_call() {
    let result = create_linter().lint("this.$nextTick()", 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_unrelated_next_tick_receiver_is_not_flagged() {
    let result = create_linter().lint("vm.$nextTick()", 0);
    assert_eq!(result.warning_count, 0, "{:#?}", result.diagnostics);
}

#[test]
fn test_invalid_aliased_bare_call() {
    let result = create_linter().lint("import { nextTick as tick } from 'vue'\ntick()", 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_invalid_namespace_call_before_import() {
    let result = create_linter().lint("Vue.nextTick(); import * as Vue from 'vue';", 0);
    assert_eq!(result.warning_count, 1, "{:#?}", result.diagnostics);
}

#[test]
fn test_invalid_void_bare_call() {
    let result = create_linter().lint("import { nextTick } from 'vue'\nvoid nextTick()", 0);
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_unrelated_identifier_not_flagged() {
    // `nextTick` not imported from vue and not `$nextTick`: a local function
    // named `tick` is unrelated.
    let result = create_linter().lint("tick()", 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_same_named_non_vue_import_not_flagged() {
    let result = create_linter().lint("import { nextTick } from './scheduler'; nextTick();", 0);
    assert_eq!(result.warning_count, 0, "{:#?}", result.diagnostics);
}

fn messages(source: &str) -> Vec<(u32, std::string::String)> {
    create_linter()
        .lint(source, 0)
        .diagnostics
        .into_iter()
        .map(|diagnostic| (diagnostic.start, diagnostic.message.to_string()))
        .collect()
}

#[test]
fn test_reports_eslint_plugin_vue_shapes() {
    let source = "import { nextTick } from \"vue\";\nasync function f(done) {\n  nextTick().catch(done);\n  nextTick().finally(done);\n  nextTick(done, done);\n  await nextTick(done);\n  nextTick(done).then(done);\n  const reference = nextTick;\n  await Promise.all([nextTick, reference]);\n}";
    let found: Vec<_> = messages(source)
        .into_iter()
        .map(|(_, message)| message)
        .collect();
    assert_eq!(
        found,
        [
            super::NOT_AWAITED,
            super::NOT_AWAITED,
            super::TOO_MANY_ARGUMENTS,
            super::AWAITED_CALLBACK,
            super::AWAITED_CALLBACK,
            super::NOT_CALLED,
        ]
    );
}

#[test]
fn test_promise_consumers_and_value_passing_are_valid() {
    let source = "import { nextTick } from \"vue\";\nasync function f(done) {\n  const p = nextTick();\n  done(nextTick);\n  done({ nextTick });\n  await Promise.all([nextTick(), p]);\n  nextTick().then(done).catch(done);\n  const pick = done ? nextTick : done;\n  return [p, pick];\n}";
    assert!(messages(source).is_empty(), "{:?}", messages(source));
}

#[test]
fn test_property_keys_and_conditional_tests_are_not_forwarded() {
    let source = "import { nextTick } from \"vue\";\nfunction f(done) {\n  done({ nextTick, tick: (nextTick), pick: done ? nextTick : done });\n  done({ [nextTick]: done });\n  done({ ready: nextTick ? done : done });\n}";
    let key = source.find("[nextTick]").map(|index| index as u32 + 1);
    let test = source.find("nextTick ? done").map(|index| index as u32);
    let expected: Vec<_> = [key, test]
        .into_iter()
        .flatten()
        .map(|start| (start, std::string::String::from(super::NOT_CALLED)))
        .collect();
    assert_eq!(expected.len(), 2);
    assert_eq!(messages(source), expected);
}

#[test]
fn test_shadowed_parameter_is_not_next_tick() {
    let source =
        "import { nextTick } from \"vue\";\nfunction f(nextTick) { nextTick(); [nextTick]; }";
    assert!(messages(source).is_empty(), "{:?}", messages(source));
}

#[test]
fn test_reports_at_the_next_tick_identifier() {
    let source = "import { nextTick } from \"vue\";\nasync function f() { await nextTick(() => {}); this.$nextTick(); }";
    let starts: Vec<_> = messages(source)
        .into_iter()
        .map(|(start, _)| start)
        .collect();
    let first = source.find("nextTick(() =>").map(|index| index as u32);
    let second = source.find("$nextTick").map(|index| index as u32);
    assert_eq!(
        starts,
        [first, second].into_iter().flatten().collect::<Vec<_>>()
    );
}

#[test]
fn test_offset_applied() {
    let source = "this.$nextTick()";
    let result = create_linter().lint(source, 50);
    assert_eq!(result.warning_count, 1);
    assert_eq!(result.diagnostics[0].start, 55);
}
