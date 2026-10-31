use super::super::DefineMacrosOrder;
use crate::rules::script::{ScriptLintResult, ScriptLinter, ScriptRule, SfcScriptContext};
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::config::DefineMacrosOrderOptions;

const EXPOSE: &str = "`defineExpose` should be the last statement in `<script setup>`.";

fn check(source: &str, offset: usize, setup: bool) -> ScriptLintResult {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::ts()).parse();
    assert!(parsed.errors.is_empty());
    let rule = DefineMacrosOrder::configured(DefineMacrosOrderOptions {
        define_expose_last: true,
        ..Default::default()
    });
    let mut result = ScriptLintResult::default();
    rule.check_program_with_sfc(
        &parsed.program,
        source,
        offset,
        SfcScriptContext {
            is_sfc: setup,
            is_script_setup: setup,
            ..Default::default()
        },
        &mut result,
    );
    result
}

#[test]
fn last_statement_is_literal_and_last_expose_is_the_selected_occurrence() {
    for tail in [
        "ordinary();",
        "type End = string;",
        "import 'tail';",
        ";",
        "debugger;",
    ] {
        let source = format!("defineExpose({{}});\n{tail}");
        let result = check(&source, 0, true);
        assert_eq!(result.diagnostics.len(), 1, "{tail}");
        assert_eq!(result.diagnostics[0].message, EXPOSE);
        assert!(result.diagnostics[0].fix.is_none());
    }
    for source in [
        "ordinary();\ndefineExpose({});",
        "defineExpose({});\n// trailing comment",
        "defineExpose({});\nordinary();\ndefineExpose({});",
        "withDefaults(defineExpose({}), {});\nordinary();",
        "function nested() { defineExpose({}); }\nordinary();",
        "api.defineExpose({});\nordinary();",
        "declare const ambient = defineExpose({});\nordinary();",
    ] {
        assert!(check(source, 0, true).diagnostics.is_empty(), "{source}");
    }
}

#[test]
fn expose_precedes_the_independent_prefix_finding_with_exact_unicode_crlf_spans() {
    for newline in ["\n", "\r\n"] {
        let source = [
            "// 🦀 authored prefix",
            "defineEmits();",
            "defineProps();",
            "defineExpose({});",
            "ordinary();",
        ]
        .join(newline);
        let result = check(&source, 41, true);
        assert_eq!(result.diagnostics.len(), 2);
        let expose = &result.diagnostics[0];
        let start = 41 + source.find("defineExpose").unwrap() as u32;
        assert_eq!(expose.message, EXPOSE);
        assert_eq!(
            (expose.start, expose.end),
            (start, start + "defineExpose({});".len() as u32)
        );
        assert_eq!(
            expose.help.as_deref(),
            Some("Put `defineExpose` as the last statement in `<script setup>`.")
        );
        assert_eq!(expose.labels.len(), 1);
        assert_eq!(expose.labels[0].message, "compiler macro should be last");
        assert!(expose.fix.is_none());
        assert_eq!(
            result.diagnostics[1].message,
            "`defineProps` should be above `defineEmits`."
        );
        assert!(result.diagnostics[1].fix.is_none());
    }
}

#[test]
fn new_last_check_requires_real_setup_context_and_manual_repair_is_a_fixed_point() {
    let invalid = "defineExpose({ ready: true });\nordinary();";
    assert_eq!(check(invalid, 0, true).diagnostics.len(), 1);
    assert!(check(invalid, 0, false).diagnostics.is_empty());
    let mut standalone = ScriptLinter::new();
    standalone.add_rule(Box::new(DefineMacrosOrder::configured(
        DefineMacrosOrderOptions {
            define_expose_last: true,
            ..Default::default()
        },
    )));
    assert!(standalone.lint(invalid, 0).diagnostics.is_empty());
    let repaired = "ordinary();\ndefineExpose({ ready: true });";
    assert!(check(repaired, 0, true).diagnostics.is_empty());
    assert!(check(repaired, 0, true).diagnostics.is_empty());
}
