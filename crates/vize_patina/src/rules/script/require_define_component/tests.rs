use super::*;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

fn lint(source: &str, context: SfcScriptContext<'_>) -> ScriptLintResult {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    assert!(!parsed.panicked && parsed.diagnostics.is_empty());
    let mut result = ScriptLintResult::default();
    RequireDefineComponent.check_program_with_sfc(&parsed.program, source, 0, context, &mut result);
    result
}

fn ts_sfc() -> SfcScriptContext<'static> {
    SfcScriptContext {
        is_sfc: true,
        typescript: true,
        ..Default::default()
    }
}

#[test]
fn rejects_unwrapped_typescript_default_exports() {
    for source in [
        "export default { props: { count: Number } }",
        "const component = { setup() {} }; export default component",
        "export default class Component {}",
        "export default ({} satisfies Record<string, unknown>)",
    ] {
        let result = lint(source, ts_sfc());
        assert_eq!(result.error_count, 1, "source: {source}");
    }
}

#[test]
fn accepts_vue_and_nuxt_component_helpers() {
    for source in [
        "export default defineComponent({})",
        "export default defineNuxtComponent({})",
        "export default (defineComponent({}) as Component)",
        "export default (defineComponent({}) satisfies Component)",
    ] {
        let result = lint(source, ts_sfc());
        assert_eq!(result.error_count, 0, "source: {source}");
    }
}

#[test]
fn ignores_javascript_setup_and_standalone_modules() {
    let source = "export default {}";
    for context in [
        SfcScriptContext {
            is_sfc: true,
            ..Default::default()
        },
        SfcScriptContext {
            is_sfc: true,
            is_script_setup: true,
            typescript: true,
            ..Default::default()
        },
        SfcScriptContext {
            typescript: true,
            ..Default::default()
        },
    ] {
        assert_eq!(lint(source, context).error_count, 0);
    }
}

#[test]
fn reports_the_default_export_span() {
    let source = "import type { X } from './x'\nexport default {}";
    let result = lint(source, ts_sfc());
    assert_eq!(result.error_count, 1);
    assert_eq!(
        &source[result.diagnostics[0].start as usize..result.diagnostics[0].end as usize],
        "export default {}"
    );
}

#[test]
fn happy_path_enforces_the_rule_on_typescript_sfcs() {
    let source = "<script lang=\"ts\">export default { props: { count: Number } }</script>";
    let result =
        crate::Linter::with_preset(crate::LintPreset::HappyPath).lint_sfc(source, "Counter.vue");
    assert_eq!(
        result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == META.name)
            .count(),
        1
    );
}
