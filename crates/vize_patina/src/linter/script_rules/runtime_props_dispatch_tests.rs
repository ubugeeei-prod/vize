use super::super::Linter;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rules::script::{
    NoRequiredPropWithDefault, ScriptLintResult, ScriptLinter, ScriptRule, ScriptRuleMeta,
    SfcScriptContext,
};
use oxc_ast::ast::Program;

struct Namesake;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-required-prop-with-default",
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
        assert!(sfc.is_sfc && sfc.is_script_setup && sfc.sole_script_block);
        result.add_diagnostic(LintDiagnostic::warn(
            META.name,
            "configured namesake kept its SFC dispatch",
            offset as u32,
            offset as u32 + 1,
        ));
    }
}

#[test]
fn configured_namesake_cannot_gain_runtime_props_capability() {
    let rule = Namesake;
    assert!(rule.as_setup_props_defaults().is_none());
    let mut linter = Linter::new();
    linter
        .script_rule_overrides
        .insert(META.name, Box::new(rule));
    let source = "<script setup>defineProps({ value: { required: true, default: 1 } })</script>";
    let result = linter.lint_sfc(source, "Namesake.vue");
    assert_eq!(result.error_count, 0);
    assert_eq!(result.warning_count, 1);
    assert_eq!(result.diagnostics.len(), 1);
    let diagnostic = &result.diagnostics[0];
    assert_eq!(diagnostic.rule_name, META.name);
    assert_eq!(diagnostic.severity, Severity::Warning);
    assert_eq!(
        diagnostic.message,
        "configured namesake kept its SFC dispatch"
    );
    assert_eq!((diagnostic.start, diagnostic.end), (14, 15));
    assert_eq!(diagnostic.help, None);
    assert!(diagnostic.labels.is_empty());
    assert!(diagnostic.fix.is_none());
}

#[test]
fn standalone_runtime_names_preserve_existing_script_rule_dispatch() {
    let rule = NoRequiredPropWithDefault;
    assert!(rule.as_setup_props_defaults().is_some());
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(rule));
    let result = linter.lint("defineProps({ value: { required: true, default: 1 } })", 23);
    assert_eq!(result.error_count, 0);
    assert_eq!(result.warning_count, 0);
    assert!(result.diagnostics.is_empty());
}
