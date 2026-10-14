use super::super::{LintResult, Linter};
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::preset::LintPreset;
use crate::rules::script::{
    NoDupeKeys, ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext,
};
use oxc_ast::ast::Program;
use serde_json::{Value, json};

struct Namesake;
static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-dupe-keys",
    description: "configured namesake dispatch witness",
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
        source: &str,
        offset: usize,
        sfc: SfcScriptContext<'_>,
        result: &mut ScriptLintResult,
    ) {
        assert!(sfc.is_sfc);
        assert_eq!(
            source,
            "\nconst props = defineProps({foo: String}); const foo = 1;\n"
        );
        result.add_diagnostic(LintDiagnostic::warn(
            META.name,
            "configured duplicate-key namesake callback",
            offset as u32 + 1,
            offset as u32 + 6,
        ));
    }
}

fn whole(result: &LintResult) -> Value {
    json!({"filename": result.filename, "error_count": result.error_count,
        "warning_count": result.warning_count,
        "diagnostics": result.diagnostics.iter().map(|d| json!({
            "rule_name": d.rule_name, "severity": d.severity, "message": d.message,
            "start": d.start, "end": d.end, "help": d.help, "fix": d.fix,
            "labels": d.labels.iter().map(|label| json!({
                "message": label.message, "start": label.start, "end": label.end
            })).collect::<Vec<_>>()
        })).collect::<Vec<_>>()})
}

#[test]
fn configured_namesake_keeps_complete_generic_normal_setup_and_external_packets() {
    let content = "\nconst props = defineProps({foo: String}); const foo = 1;\n";
    for (filename, opening) in [
        ("SetupNamesake.vue", "<script setup lang=\"ts\">"),
        ("NormalNamesake.vue", "<script lang=\"ts\">"),
        (
            "ExternalNamesake.vue",
            "<script setup lang=\"ts\" src=\"./props.ts\">",
        ),
    ] {
        let rule = Namesake;
        assert!(rule.as_setup_dupe_keys().is_none());
        let mut linter = Linter::with_preset(LintPreset::Incremental)
            .with_additional_rules(vec![META.name.into()]);
        linter
            .script_rule_overrides
            .insert(META.name, Box::new(rule));
        let source = format!("{opening}{content}</script>\n");
        assert_eq!(
            whole(&linter.lint_sfc(&source, filename)),
            json!({
                "filename": filename, "error_count": 0, "warning_count": 1,
                "diagnostics": [{"rule_name": META.name, "severity": "warning",
                    "message": "configured duplicate-key namesake callback",
                    "start": opening.len() + 1, "end": opening.len() + 6,
                    "help": null, "fix": null, "labels": []}]
            })
        );
    }
}

struct ForeignCapability(NoDupeKeys);
static FOREIGN_META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-required-prop-with-default",
    description: "foreign duplicate-key capability witness",
    default_severity: Severity::Warning,
};
impl ScriptRule for ForeignCapability {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &FOREIGN_META
    }
    fn uses_ast(&self) -> bool {
        true
    }
    fn as_setup_dupe_keys(&self) -> Option<&NoDupeKeys> {
        Some(&self.0)
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
            FOREIGN_META.name,
            "foreign capability keeps generic callback",
            offset as u32 + 1,
            offset as u32 + 6,
        ));
    }
}

#[test]
fn foreign_typed_capability_cannot_claim_duplicate_rule_dispatch() {
    let mut linter = Linter::with_preset(LintPreset::Incremental)
        .with_additional_rules(vec![FOREIGN_META.name.into()]);
    linter
        .script_rule_overrides
        .insert(FOREIGN_META.name, Box::new(ForeignCapability(NoDupeKeys)));
    let opening = "<script setup lang=\"ts\">";
    let source =
        format!("{opening}\nconst props = defineProps({{foo: String}}); const foo = 1;\n</script>");
    assert_eq!(
        whole(&linter.lint_sfc(&source, "ForeignCapability.vue")),
        json!({
            "filename": "ForeignCapability.vue", "error_count": 0, "warning_count": 1,
            "diagnostics": [{"rule_name": FOREIGN_META.name, "severity": "warning",
                "message": "foreign capability keeps generic callback",
                "start": opening.len() + 1, "end": opening.len() + 6,
                "help": null, "fix": null, "labels": []}]
        })
    );
}
