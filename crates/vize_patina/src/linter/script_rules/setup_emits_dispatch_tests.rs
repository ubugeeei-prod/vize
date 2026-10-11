use super::super::Linter;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rules::script::{ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext};
use oxc_ast::ast::Program;

struct Namesake;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/return-in-emits-validator",
    description: "configured namesake dispatch control",
    default_severity: Severity::Warning,
};

impl ScriptRule for Namesake {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    fn uses_ast(&self) -> bool {
        true
    }

    fn check_program_with_sfc<'a>(
        &self,
        _program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        sfc: SfcScriptContext<'_>,
        result: &mut ScriptLintResult,
    ) {
        assert!(sfc.is_sfc && sfc.is_script_setup);
        result.add_diagnostic(LintDiagnostic::warn(
            META.name,
            "configured namesake kept its SFC dispatch",
            offset as u32,
            offset as u32 + 1,
        ));
    }
}

#[test]
fn configured_namesake_keeps_existing_dispatch_with_unavailable_sibling() {
    let rule = Namesake;
    assert!(rule.as_setup_emits_validator().is_none());
    let mut linter = Linter::new();
    linter
        .script_rule_overrides
        .insert(META.name, Box::new(rule));
    let source = "<script lang=\"ts\">const = ;</script>\n<script setup>const emit = defineEmits({ submit(payload) { void payload } })</script>";
    let result = linter.lint_sfc(source, "Namesake.vue");
    assert_eq!(result.error_count, 0);
    assert_eq!(result.warning_count, 1);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].message,
        "configured namesake kept its SFC dispatch"
    );
    let start = source.find("const emit").unwrap() as u32;
    assert_eq!(result.diagnostics[0].start, start);
    assert_eq!(result.diagnostics[0].end, start + 1);
}
