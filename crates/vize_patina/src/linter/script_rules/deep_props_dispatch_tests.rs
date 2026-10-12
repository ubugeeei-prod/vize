use super::super::Linter;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::preset::LintPreset;
use crate::rules::script::{
    NoDeepDestructureInProps, ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext,
};
use oxc_ast::ast::Program;
use serde_json::{Value, json};

struct Namesake {
    is_setup: bool,
}

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-deep-destructure-in-props",
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
        source: &str,
        offset: usize,
        sfc: SfcScriptContext<'_>,
        result: &mut ScriptLintResult,
    ) {
        assert!(sfc.is_sfc && sfc.sole_script_block);
        assert_eq!(sfc.is_script_setup, self.is_setup);
        assert_eq!(source, "\nconst { user: { name } } = defineProps();\n");
        result.add_diagnostic(LintDiagnostic::warn(
            META.name,
            "configured namesake retains generic script dispatch",
            offset as u32 + 7,
            offset as u32 + 25,
        ));
    }
}

fn whole(result: &super::super::LintResult) -> Value {
    json!({
        "filename": result.filename,
        "error_count": result.error_count,
        "warning_count": result.warning_count,
        "diagnostics": result.diagnostics.iter().map(|d| json!({
            "rule_name": d.rule_name, "severity": d.severity, "message": d.message,
            "start": d.start, "end": d.end, "help": d.help, "fix": d.fix,
            "labels": d.labels.iter().map(|label| json!({
                "message": label.message, "start": label.start, "end": label.end
            })).collect::<Vec<_>>()
        })).collect::<Vec<_>>()
    })
}

#[test]
fn custom_namesake_retains_generic_engine_dispatch_for_all_three_envelopes() {
    for (filename, source, is_setup, start, end) in [
        (
            "SetupNamesake.vue",
            "<script setup lang=\"ts\">\nconst { user: { name } } = defineProps();\n</script>\n",
            true,
            31,
            49,
        ),
        (
            "NormalNamesake.vue",
            "<script lang=\"ts\">\nconst { user: { name } } = defineProps();\n</script>\n",
            false,
            25,
            43,
        ),
        (
            "ExternalNamesake.vue",
            "<script setup lang=\"ts\" src=\"./props.ts\">\nconst { user: { name } } = defineProps();\n</script>\n",
            true,
            48,
            66,
        ),
    ] {
        let rule = Namesake { is_setup };
        assert!(rule.as_setup_props_destructure().is_none());
        let mut linter = Linter::with_preset(LintPreset::Incremental)
            .with_additional_rules(vec![META.name.into()]);
        linter
            .script_rule_overrides
            .insert(META.name, Box::new(rule));
        let result = linter.lint_sfc(source, filename);
        assert_eq!(
            whole(&result),
            json!({
                "filename": filename, "error_count": 0, "warning_count": 1,
                "diagnostics": [{
                    "rule_name": META.name, "severity": "warning",
                    "message": "configured namesake retains generic script dispatch",
                    "start": start, "end": end, "help": null, "fix": null, "labels": []
                }]
            }),
            "{filename}"
        );
    }
}

struct ForeignCapability(NoDeepDestructureInProps);

static FOREIGN_META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-required-prop-with-default",
    description: "foreign typed capability dispatch control",
    default_severity: Severity::Warning,
};

impl ScriptRule for ForeignCapability {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &FOREIGN_META
    }

    fn uses_ast(&self) -> bool {
        true
    }

    fn as_setup_props_destructure(&self) -> Option<&NoDeepDestructureInProps> {
        Some(&self.0)
    }

    fn check_program_with_sfc<'a>(
        &self,
        _program: &'a Program<'a>,
        source: &str,
        offset: usize,
        sfc: SfcScriptContext<'_>,
        result: &mut ScriptLintResult,
    ) {
        assert!(sfc.is_sfc && sfc.is_script_setup && sfc.sole_script_block);
        assert_eq!(source, "\nconst { user: { name } } = defineProps();\n");
        result.add_diagnostic(LintDiagnostic::warn(
            FOREIGN_META.name,
            "foreign capability retains generic script dispatch",
            offset as u32 + 7,
            offset as u32 + 25,
        ));
    }
}

#[test]
fn foreign_capability_identity_keeps_the_complete_generic_engine_packet() {
    let rule = ForeignCapability(NoDeepDestructureInProps::default());
    assert!(rule.as_setup_props_destructure().is_some());
    let mut linter = Linter::with_preset(LintPreset::Incremental)
        .with_additional_rules(vec![FOREIGN_META.name.into()])
        .with_rule_severity_overrides(vec![(FOREIGN_META.name.into(), Severity::Warning)]);
    linter
        .script_rule_overrides
        .insert(FOREIGN_META.name, Box::new(rule));
    assert_eq!(
        whole(&linter.lint_sfc(
            "<script setup lang=\"ts\">\nconst { user: { name } } = defineProps();\n</script>\n",
            "ForeignCapability.vue"
        )),
        json!({
            "filename": "ForeignCapability.vue", "error_count": 0, "warning_count": 1,
            "diagnostics": [{
                "rule_name": FOREIGN_META.name, "severity": "warning",
                "message": "foreign capability retains generic script dispatch",
                "start": 31, "end": 49, "help": null, "fix": null, "labels": []
            }]
        })
    );
}
