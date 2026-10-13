use super::NoDeepDestructureInProps;
use crate::rules::script::{
    ScriptLintResult, ScriptLinter, ScriptRule, SfcScriptContext, script_source_type,
};
use oxc_allocator::Allocator;
use oxc_parser::Parser;

struct SetupLinter(NoDeepDestructureInProps);

impl SetupLinter {
    fn lint(&self, source: &str, offset: usize) -> ScriptLintResult {
        self.lint_context(source, offset, true, true, true)
    }

    fn lint_context(
        &self,
        source: &str,
        offset: usize,
        is_sfc: bool,
        is_script_setup: bool,
        sole_script_block: bool,
    ) -> ScriptLintResult {
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, source, script_source_type()).parse();
        assert!(!parsed.panicked && parsed.diagnostics.is_empty());
        let mut result = ScriptLintResult::default();
        self.0.check_program_with_sfc(
            &parsed.program,
            source,
            offset,
            SfcScriptContext {
                is_sfc,
                is_script_setup,
                sole_script_block,
                ..Default::default()
            },
            &mut result,
        );
        result
    }
}

fn create_linter() -> SetupLinter {
    SetupLinter(NoDeepDestructureInProps::default())
}

fn assert_empty(result: ScriptLintResult) {
    assert!(result.diagnostics.is_empty());
    assert_eq!(result.error_count, 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_simple_destructure() {
    let linter = create_linter();
    let result = linter.lint(
        "const { name, count = 0 } = defineProps<{ name: string }>()",
        0,
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_valid_no_destructure() {
    let linter = create_linter();
    let result = linter.lint("const props = defineProps<{ name: string }>()", 0);
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_invalid_deep_destructure() {
    let linter = create_linter();
    let result = linter.lint(
        "const { user: { name } } = defineProps<{ user: User }>()",
        0,
    );
    assert_eq!(result.warning_count, 1);
    insta::with_settings!({ snapshot_path => "../snapshots" }, {
        insta::assert_debug_snapshot!(result.diagnostics);
    });
}

#[test]
fn test_invalid_very_deep_destructure() {
    let linter = create_linter();
    let result = linter.lint(
        "const /* props */ { config: { settings: { theme } } } = defineProps()",
        0,
    );
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_invalid_deep_destructure_with_defaults() {
    let linter = create_linter();
    let result = linter.lint(
        "const { user: { name } } = withDefaults(defineProps<{ user: User }>(), {})",
        0,
    );
    assert_eq!(result.warning_count, 1);
}

#[test]
fn test_valid_with_higher_max_depth() {
    let linter = SetupLinter(NoDeepDestructureInProps { max_depth: 2 });
    let result = linter.lint(
        "const { user: { name } } = defineProps<{ user: User }>()",
        0,
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_define_props_string_not_matched() {
    let linter = create_linter();
    let result = linter.lint(
        r#"const text = "const { user: { name } } = defineProps()""#,
        0,
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn test_unrelated_deep_destructure_not_matched() {
    let linter = create_linter();
    let result = linter.lint(
        "const { user: { name } } = createProps<{ user: User }>()",
        0,
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn standalone_and_direct_generic_checks_have_no_macro_owner() {
    let source = "const { user: { name } } = defineProps<{ user: User }>()";
    let rule = NoDeepDestructureInProps::default();
    let mut result = ScriptLintResult::default();
    rule.check(source, 0, &mut result);
    assert_empty(result);
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(rule));
    assert_empty(linter.lint(source, 0));
}

#[test]
fn sfc_setup_and_single_block_evidence_are_required() {
    let linter = create_linter();
    let source = "const { user: { name } } = defineProps<{ user: User }>()";
    for (is_sfc, is_setup, sole) in [
        (false, true, true),
        (true, false, true),
        (true, true, false),
    ] {
        assert_empty(linter.lint_context(source, 0, is_sfc, is_setup, sole));
    }
}

#[test]
fn nested_arrays_count_with_defaults_and_custom_depth() {
    let source = "const { items: [first = 1] } = defineProps<{ items: number[] }>()";
    let result = create_linter().lint(source, 7);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.warning_count, 1);
    assert_eq!(result.error_count, 0);
    assert_eq!(result.diagnostics[0].start, 13);
    assert_eq!(result.diagnostics[0].end, 35);
    assert_empty(SetupLinter(NoDeepDestructureInProps { max_depth: 2 }).lint(source, 7));
    let deeper = "const { items: [{ name }] } = defineProps<{ items: User[] }>()";
    assert_eq!(
        SetupLinter(NoDeepDestructureInProps { max_depth: 2 })
            .lint(deeper, 0)
            .diagnostics
            .len(),
        1
    );
}

#[test]
fn local_foreign_hoisted_and_nested_calls_are_not_macro_producers() {
    let linter = create_linter();
    for source in [
        "function defineProps() { return { user: { name: 'Ada' } }; } const { user: { name } } = defineProps()",
        "import { defineProps } from './api'; const { user: { name } } = defineProps()",
        "if (false) { var defineProps = () => ({ user: { name: 'Ada' } }); } const { user: { name } } = defineProps()",
        "function ordinary(defineProps: any) { const { user: { name } } = defineProps(); }",
        "const withDefaults = (props: any) => props; const { user: { name } } = withDefaults(defineProps<User>(), {})",
        "const { user: { name } } = (defineProps as any)()",
        "const { user: { name } } = (withDefaults as any)(defineProps<User>(), {})",
    ] {
        assert_empty(linter.lint(source, 0));
    }
}
