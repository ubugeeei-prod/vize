use super::{NoExportInScriptSetup, ScriptLintResult, ScriptRule, SfcScriptContext};
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

fn lint_context(source: &str, offset: usize, context: SfcScriptContext<'_>) -> ScriptLintResult {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    assert!(!parsed.panicked && parsed.diagnostics.is_empty());
    let mut result = ScriptLintResult::default();
    NoExportInScriptSetup.check_program_with_sfc(
        &parsed.program,
        source,
        offset,
        context,
        &mut result,
    );
    result
}

pub(super) fn lint_setup(source: &str, offset: usize) -> ScriptLintResult {
    lint_context(
        source,
        offset,
        SfcScriptContext {
            is_sfc: true,
            is_script_setup: true,
            ..Default::default()
        },
    )
}

#[test]
fn exact_block_ownership_preserves_exports_without_await_or_macros() {
    let source = "export const value = 1;\nexport default {};\n";
    let setup = lint_setup(source, 100);
    assert_eq!((setup.error_count, setup.warning_count), (2, 0));
    assert_eq!(
        (setup.diagnostics[0].start, setup.diagnostics[0].end),
        (100, 123)
    );
    assert_eq!(
        (setup.diagnostics[1].start, setup.diagnostics[1].end),
        (124, 142)
    );
    for context in [
        SfcScriptContext::default(),
        SfcScriptContext {
            is_sfc: true,
            ..Default::default()
        },
        SfcScriptContext {
            is_script_setup: true,
            ..Default::default()
        },
    ] {
        let result = lint_context(source, 100, context);
        assert_eq!((result.error_count, result.warning_count), (0, 0));
        assert!(result.diagnostics.is_empty());
    }
}

#[test]
fn top_level_await_and_macro_names_do_not_establish_sfc_ownership() {
    for source in [
        "const value = await load();\nexport { value };\n",
        "function defineProps() {}\ndefineProps();\nexport const value = 1;\n",
        "const defineEmits = () => {};\ndefineEmits();\nexport default {};\n",
    ] {
        for context in [
            SfcScriptContext::default(),
            SfcScriptContext {
                is_sfc: true,
                ..Default::default()
            },
        ] {
            let result = lint_context(source, 0, context);
            assert_eq!((result.error_count, result.warning_count), (0, 0));
            assert!(result.diagnostics.is_empty());
        }
        assert_eq!(lint_setup(source, 0).error_count, 1);
    }
}
