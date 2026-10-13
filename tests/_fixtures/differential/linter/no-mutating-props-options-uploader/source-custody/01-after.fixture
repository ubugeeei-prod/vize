//! Borrow ordinary-script callbacks from the already configured rule instances.

use super::super::{LintResult, Linter, merge_script_result};
use crate::rules::script::{ScriptLintResult, ScriptRule, SfcScriptContext};
use oxc_parser::ParserReturn;
use vize_l0::profile;

fn active(linter: &Linter) -> impl Iterator<Item = &dyn ScriptRule> + '_ {
    linter.registry.rules().iter().filter_map(|rule| {
        let name = rule.meta().name;
        if !linter.is_rule_enabled(name) {
            return None;
        }
        let script = rule.as_sfc_script_rule()?;
        (script.uses_ast() && script.meta().name == name).then_some(script)
    })
}

pub(super) fn has_active(linter: &Linter) -> bool {
    active(linter).next().is_some()
}

pub(super) fn run(
    linter: &Linter,
    source: &str,
    offset: usize,
    parsed: Option<&ParserReturn<'_>>,
    sfc: SfcScriptContext<'_>,
    result: &mut LintResult,
) {
    let Some(parsed) = parsed.filter(|parsed| !parsed.panicked && parsed.diagnostics.is_empty())
    else {
        return;
    };
    for rule in active(linter) {
        let mut lint = ScriptLintResult::default();
        profile!(
            "patina.script_rule.registered_sfc",
            rule.check_program_with_sfc(&parsed.program, source, offset, sfc, &mut lint,)
        );
        merge_script_result(linter, result, lint);
    }
}
