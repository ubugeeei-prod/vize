use super::{LintResult, Linter};
use crate::rules::script::{NoGetCurrentInstance, NoNextTick, NoOptionsApi, ScriptRule};
use vize_atelier_sfc::{parse_sfc, SfcDescriptor, SfcParseOptions};
use vize_carton::profile;

pub(crate) const RULE_NO_OPTIONS_API: &str = "script/no-options-api";
pub(crate) const RULE_NO_GET_CURRENT_INSTANCE: &str = "script/no-get-current-instance";
pub(crate) const RULE_NO_NEXT_TICK: &str = "script/no-next-tick";
const OPINIONATED_SCRIPT_PRESETS: &[&str] = &["opinionated", "nuxt"];

pub struct BuiltinScriptRuleMeta {
    pub name: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    pub fixable: bool,
    pub default_severity: crate::Severity,
    pub presets: &'static [&'static str],
}

pub fn builtin_script_rules() -> [BuiltinScriptRuleMeta; 3] {
    let no_options_api = NoOptionsApi;
    let no_options_api_meta = no_options_api.meta();
    let no_get_current_instance = NoGetCurrentInstance;
    let no_get_current_instance_meta = no_get_current_instance.meta();
    let no_next_tick = NoNextTick;
    let no_next_tick_meta = no_next_tick.meta();

    [
        BuiltinScriptRuleMeta {
            name: no_options_api_meta.name,
            description: no_options_api_meta.description,
            category: "Vapor",
            fixable: false,
            default_severity: no_options_api_meta.default_severity,
            presets: OPINIONATED_SCRIPT_PRESETS,
        },
        BuiltinScriptRuleMeta {
            name: no_get_current_instance_meta.name,
            description: no_get_current_instance_meta.description,
            category: "Vapor",
            fixable: false,
            default_severity: no_get_current_instance_meta.default_severity,
            presets: OPINIONATED_SCRIPT_PRESETS,
        },
        BuiltinScriptRuleMeta {
            name: no_next_tick_meta.name,
            description: no_next_tick_meta.description,
            category: "Vapor",
            fixable: false,
            default_severity: no_next_tick_meta.default_severity,
            presets: OPINIONATED_SCRIPT_PRESETS,
        },
    ]
}

#[inline]
pub(crate) fn has_active_builtin_script_rules(linter: &Linter) -> bool {
    linter
        .script_rules
        .iter()
        .copied()
        .any(|rule_name| linter.is_rule_enabled(rule_name))
}

#[inline]
pub(crate) fn parse_sfc_for_lint<'a>(
    source: &'a str,
    filename: &str,
) -> Result<SfcDescriptor<'a>, vize_atelier_sfc::SfcError> {
    profile!(
        "patina.sfc.parse_for_lint",
        parse_sfc(source, sfc_parse_options(filename))
    )
}

pub(crate) fn lint_with_descriptor<'a>(
    linter: &Linter,
    filename: &str,
    descriptor: &SfcDescriptor<'a>,
) -> LintResult {
    let mut result = if let Some(template) = descriptor.template.as_ref() {
        let mut template_result = profile!(
            "patina.sfc.descriptor.template_lint",
            linter.lint_template(&template.content, filename)
        );
        let byte_offset = template.loc.start as u32;
        if byte_offset > 0 {
            for diag in &mut template_result.diagnostics {
                diag.start += byte_offset;
                diag.end += byte_offset;
                for label in &mut diag.labels {
                    label.start += byte_offset;
                    label.end += byte_offset;
                }
            }
        }
        template_result
    } else {
        LintResult {
            filename: filename.into(),
            diagnostics: Vec::new(),
            error_count: 0,
            warning_count: 0,
        }
    };

    append_builtin_script_diagnostics(linter, descriptor, &mut result);
    result
}

pub(crate) fn append_builtin_script_diagnostics<'a>(
    linter: &Linter,
    descriptor: &SfcDescriptor<'a>,
    result: &mut LintResult,
) {
    if linter.script_rules.is_empty() {
        return;
    }

    append_builtin_script_rule(
        linter,
        descriptor,
        result,
        RULE_NO_OPTIONS_API,
        "patina.script_rule.no_options_api",
        NoOptionsApi,
    );
    append_builtin_script_rule(
        linter,
        descriptor,
        result,
        RULE_NO_GET_CURRENT_INSTANCE,
        "patina.script_rule.no_get_current_instance",
        NoGetCurrentInstance,
    );
    append_builtin_script_rule(
        linter,
        descriptor,
        result,
        RULE_NO_NEXT_TICK,
        "patina.script_rule.no_next_tick",
        NoNextTick,
    );
}

fn merge_script_result(
    result: &mut LintResult,
    script_result: crate::rules::script::ScriptLintResult,
) {
    result.error_count += script_result.error_count;
    result.warning_count += script_result.warning_count;
    result.diagnostics.extend(script_result.diagnostics);
}

fn append_builtin_script_rule<'a, R: ScriptRule>(
    linter: &Linter,
    descriptor: &SfcDescriptor<'a>,
    result: &mut LintResult,
    rule_name: &str,
    profile_name: &'static str,
    rule: R,
) {
    if !linter.is_rule_enabled(rule_name) || !linter.script_rules.contains(&rule_name) {
        return;
    }

    if let Some(script) = descriptor.script.as_ref() {
        let mut lint = crate::rules::script::ScriptLintResult::default();
        profile!(
            profile_name,
            rule.check(script.content.as_ref(), script.loc.start, &mut lint)
        );
        merge_script_result(result, lint);
    }
    if let Some(script_setup) = descriptor.script_setup.as_ref() {
        let mut lint = crate::rules::script::ScriptLintResult::default();
        profile!(
            profile_name,
            rule.check(
                script_setup.content.as_ref(),
                script_setup.loc.start,
                &mut lint,
            )
        );
        merge_script_result(result, lint);
    }
}

#[inline]
fn sfc_parse_options(filename: &str) -> SfcParseOptions {
    SfcParseOptions {
        filename: filename.into(),
        ..Default::default()
    }
}
