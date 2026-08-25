use super::{META, NoDeprecatedDestroyedLifecycle};
use crate::diagnostic::Severity;
use crate::rules::script::{ScriptLintResult, ScriptLinter, ScriptRule, script_source_type};
use crate::{Linter, builtin_script_rules};
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use vize_carton::{String, ToCompactString};

fn lint(source: &str) -> ScriptLintResult {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(NoDeprecatedDestroyedLifecycle));
    linter.lint(source, 0)
}

fn apply_fixes(source: &str, result: &ScriptLintResult) -> String {
    let mut edits = result
        .diagnostics
        .iter()
        .flat_map(|diagnostic| diagnostic.fix.as_ref().unwrap().edits.iter())
        .collect::<Vec<_>>();
    edits.sort_by_key(|edit| std::cmp::Reverse(edit.start));

    let mut fixed = source.to_compact_string();
    for edit in edits {
        fixed.replace_range(edit.start as usize..edit.end as usize, &edit.new_text);
    }
    fixed
}

fn assert_valid_and_idempotent(source: &str, expected: &str) {
    let first = lint(source);
    assert!(!first.diagnostics.is_empty(), "{source}");
    let fixed = apply_fixes(source, &first);
    assert_eq!(fixed, expected, "{source}");

    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &fixed, script_source_type()).parse();
    assert!(!parsed.panicked, "{fixed}");
    assert!(
        parsed.diagnostics.is_empty(),
        "{fixed}: {:?}",
        parsed.diagnostics
    );
    assert!(lint(&fixed).diagnostics.is_empty(), "{fixed}");
}

#[test]
fn metadata_and_registry_match_the_upstream_contract() {
    let meta = NoDeprecatedDestroyedLifecycle.meta();
    assert_eq!(meta.name, META.name);
    assert_eq!(meta.default_severity, Severity::Error);
    assert!(!NoDeprecatedDestroyedLifecycle.runs_on_script_setup());

    let registered = builtin_script_rules()
        .into_iter()
        .find(|rule| rule.name == META.name)
        .unwrap();
    assert!(registered.fixable);
    assert!(registered.presets.is_empty());
}

#[test]
fn reports_both_hooks_with_exact_messages_spans_and_offset_fixes() {
    let source = "export default {\n  destroyed() {},\n  beforeDestroy: teardown\n}";
    let mut result = ScriptLintResult::default();
    NoDeprecatedDestroyedLifecycle.check(source, 23, &mut result);
    assert_eq!(result.error_count, 2);

    for (diagnostic, old, new, message) in [
        (
            &result.diagnostics[0],
            "destroyed",
            "unmounted",
            "The `destroyed` lifecycle hook is deprecated. Use `unmounted` instead.",
        ),
        (
            &result.diagnostics[1],
            "beforeDestroy",
            "beforeUnmount",
            "The `beforeDestroy` lifecycle hook is deprecated. Use `beforeUnmount` instead.",
        ),
    ] {
        let relative = source.find(old).unwrap() as u32;
        assert_eq!(diagnostic.rule_name, META.name);
        assert_eq!(diagnostic.message, message);
        assert_eq!(
            (diagnostic.start, diagnostic.end),
            (23 + relative, 23 + relative + old.len() as u32)
        );
        let edit = &diagnostic.fix.as_ref().unwrap().edits[0];
        assert_eq!((edit.start, edit.end), (diagnostic.start, diagnostic.end));
        assert_eq!(edit.new_text, new);
    }
}

#[test]
fn fixes_every_upstream_property_shape_and_is_idempotent() {
    for (source, expected) in [
        (
            "export default { destroyed() {} }",
            "export default { unmounted() {} }",
        ),
        (
            "export default { destroyed: teardown }",
            "export default { unmounted: teardown }",
        ),
        (
            "export default { 'destroyed': teardown }",
            "export default { unmounted: teardown }",
        ),
        (
            "export default { ['destroyed']: teardown }",
            "export default { ['unmounted']: teardown }",
        ),
        (
            "export default { [`destroyed`]: teardown }",
            "export default { [`unmounted`]: teardown }",
        ),
        (
            "const destroyed = teardown; export default { destroyed }",
            "const destroyed = teardown; export default { unmounted:destroyed }",
        ),
        (
            "const beforeDestroy = teardown; export default { beforeDestroy }",
            "const beforeDestroy = teardown; export default { beforeUnmount:beforeDestroy }",
        ),
    ] {
        assert_valid_and_idempotent(source, expected);
    }
}

#[test]
fn supports_direct_define_component_and_identifier_bound_options() {
    for source in [
        "export default { destroyed() {} }",
        "export default defineComponent({ beforeDestroy() {} })",
        "const options = { destroyed() {} }; export default options",
        "const options = { beforeDestroy() {} }; export default defineComponent(options)",
    ] {
        assert_eq!(lint(source).error_count, 1, "{source}");
    }
}

#[test]
fn ignores_vue_three_hooks_dynamic_keys_nested_objects_and_unrelated_exports() {
    for source in [
        "export default { unmounted() {}, beforeUnmount() {} }",
        "const destroyed = 'destroyed'; export default { [destroyed]() {} }",
        "const suffix = 'ed'; export default { [`destroy${suffix}`]() {} }",
        "export default { methods: { destroyed() {}, beforeDestroy() {} } }",
        "const nested = { destroyed() {} }; export default { methods: nested }",
        "const unrelated = { destroyed() {} }; export default function component() {}",
    ] {
        assert!(lint(source).diagnostics.is_empty(), "{source}");
    }
}

#[test]
fn reports_only_the_first_duplicate_of_each_hook_like_upstream() {
    let source =
        "export default { destroyed() {}, destroyed() {}, beforeDestroy() {}, beforeDestroy() {} }";
    let result = lint(source);
    assert_eq!(result.error_count, 2);
    assert_eq!(
        result.diagnostics[0].start as usize,
        source.find("destroyed").unwrap()
    );
    assert_eq!(
        result.diagnostics[1].start as usize,
        source.find("beforeDestroy").unwrap()
    );
}

#[test]
fn computed_diagnostics_cover_the_authored_literal_but_fix_only_its_contents() {
    let source = "const café = 1; export default { ['destroyed']() {} }";
    let result = lint(source);
    let key_start = source.find("'destroyed'").unwrap() as u32;
    let diagnostic = &result.diagnostics[0];
    assert_eq!(
        (diagnostic.start, diagnostic.end),
        (key_start, key_start + 11)
    );
    let edit = &diagnostic.fix.as_ref().unwrap().edits[0];
    assert_eq!((edit.start, edit.end), (key_start + 1, key_start + 10));
    assert_eq!(edit.new_text, "unmounted");
}

#[test]
fn does_not_run_on_script_setup() {
    let linter = Linter::new().with_enabled_rules(Some(vec![META.name.into()]));
    let setup = linter.lint_sfc(
        "<script setup>export default { destroyed() {} }</script>",
        "Setup.vue",
    );
    assert!(setup.diagnostics.is_empty(), "{:?}", setup.diagnostics);

    let plain = linter.lint_sfc(
        "<script>export default { destroyed() {} }</script>",
        "Plain.vue",
    );
    assert_eq!(plain.error_count, 1);
    assert_eq!(plain.diagnostics[0].rule_name, META.name);
}
