use super::{LintResult, Linter, severity::append_with_rule_overrides};
use crate::rules::script::{
    EmitsModuleScript, NoDupeKeysModuleScript, ScriptLintResult, ScriptRule, SfcScriptContext,
    script_source_type,
};
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use vize_atelier_sfc::{SfcDescriptor, SfcParseOptions, parse_sfc};
use vize_l0::profile;

mod html_scripts;
pub(crate) mod native;
mod prefilter;
mod registry;
mod sfc;
pub(crate) use sfc::{
    append_builtin_script_diagnostics, append_builtin_script_diagnostics_with_template_context,
};
mod template_context;

#[cfg(test)]
mod deep_props_dispatch_tests;
#[cfg(test)]
mod dupe_keys_dispatch_tests;
#[cfg(test)]
mod runtime_props_dispatch_tests;
#[cfg(test)]
mod setup_emits_dispatch_tests;

use html_scripts::extract_inline_scripts;
use prefilter::{
    script_rule_applies_to_filename, script_rule_applies_to_module, script_rule_may_match,
};
pub use registry::BuiltinScriptRuleMeta;
use registry::{
    ALL_BUILTIN_SCRIPT_RULE_NAMES, BUILTIN_SCRIPT_RULES, BuiltinScriptRuleEntry, RULE_NO_DUPE_KEYS,
    RULE_NO_DUPLICATE_ATTR_INHERITANCE, RULE_NO_GET_CURRENT_INSTANCE,
    RULE_NO_REQUIRED_PROP_WITH_DEFAULT, RULE_RETURN_IN_EMITS_VALIDATOR,
};

#[cfg(test)]
use registry::OPT_IN_SCRIPT_RULE_NAMES;

pub fn builtin_script_rules() -> Vec<BuiltinScriptRuleMeta> {
    BUILTIN_SCRIPT_RULES
        .iter()
        .map(|entry| entry.meta())
        .collect()
}

#[inline]
pub(crate) const fn all_builtin_script_rule_names() -> &'static [&'static str] {
    ALL_BUILTIN_SCRIPT_RULE_NAMES
}

pub(crate) fn builtin_script_rule_names_for_preset(preset: &str) -> Vec<&'static str> {
    BUILTIN_SCRIPT_RULES
        .iter()
        .filter(|entry| entry.presets.contains(&preset))
        .map(|entry| entry.rule_name)
        .collect()
}

#[cfg(test)]
#[inline]
pub(crate) const fn opt_in_script_rule_names() -> &'static [&'static str] {
    OPT_IN_SCRIPT_RULE_NAMES
}

#[inline]
pub(crate) fn has_active_builtin_script_rules(linter: &Linter) -> bool {
    active_builtin_script_rule_entries(linter, true)
        .next()
        .is_some()
}

/// Select original producer roles once, before template parsing. Ordinary
/// compiles and configurations without a template capability allocate no packet.
pub(crate) fn parse_template_for_script_rules<'a>(
    linter: &Linter,
    parser: vize_armature::Parser<'a>,
) -> (
    vize_relief::RootNode<'a>,
    Vec<vize_relief::errors::CompilerError>,
    Option<vize_armature::parser::TemplateSyntaxFacts<'a>>,
) {
    if active_builtin_script_rule_entries(linter, true)
        .any(|entry| resolved_rule(linter, entry).template_check().is_some())
    {
        let (root, errors, facts) = parser.parse_with_template_syntax_facts();
        (root, errors, Some(facts))
    } else {
        let (root, errors) = parser.parse();
        (root, errors, None)
    }
}

fn active_builtin_script_rule_entries(
    linter: &Linter,
    vapor_component: bool,
) -> impl Iterator<Item = &'static BuiltinScriptRuleEntry> + '_ {
    linter
        .script_rules
        .iter()
        .copied()
        .filter(|rule_name| linter.is_rule_enabled(rule_name))
        .filter(move |rule_name| {
            *rule_name != RULE_NO_GET_CURRENT_INSTANCE
                || vapor_component
                || linter.explicit_no_get_current_instance
        })
        .filter_map(builtin_script_rule_entry)
}

/// Only the static native builtin can gain scriptless template execution.
/// Configured script overrides and registered namesakes retain their existing callbacks.
fn has_scriptless_attrs_inheritance(linter: &Linter, filename: &str) -> bool {
    !linter
        .script_rule_overrides
        .contains_key(RULE_NO_DUPLICATE_ATTR_INHERITANCE)
        && !linter.registry.has_rule(RULE_NO_DUPLICATE_ATTR_INHERITANCE)
        && script_rule_applies_to_filename(RULE_NO_DUPLICATE_ATTR_INHERITANCE, filename)
        && active_builtin_script_rule_entries(linter, true)
            .any(|entry| entry.rule_name == RULE_NO_DUPLICATE_ATTR_INHERITANCE)
}

fn builtin_script_rule_entry(rule_name: &str) -> Option<&'static BuiltinScriptRuleEntry> {
    BUILTIN_SCRIPT_RULES
        .iter()
        .find(|entry| entry.rule_name == rule_name)
}

#[inline]
pub(crate) fn parse_sfc_for_lint<'a>(
    source: &'a str,
    filename: &str,
) -> Result<SfcDescriptor<'a>, vize_atelier_sfc::SfcError> {
    profile!(
        "patina.sfc.parse_for_lint",
        parse_sfc(
            source,
            SfcParseOptions {
                filename: filename.into(),
                ..Default::default()
            }
        )
    )
}

/// Resolve the rule instance to run for `entry`: a project-configured override
/// when present, otherwise the static registry singleton.
#[inline]
fn resolved_rule<'a>(
    linter: &'a Linter,
    entry: &'a BuiltinScriptRuleEntry,
) -> &'a dyn crate::rules::script::ScriptRule {
    match linter.script_rule_overrides.get(entry.rule_name) {
        Some(rule) => rule.as_ref(),
        None => entry.rule,
    }
}

/// A configured override bypasses the byte prefilter: its deny list may name
/// identifiers unknown to that prefilter, so overridden rules always parse the block.
#[inline]
fn entry_may_match(
    linter: &Linter,
    entry: &BuiltinScriptRuleEntry,
    source: &str,
    filename: &str,
) -> bool {
    script_rule_applies_to_filename(entry.rule_name, filename)
        && script_rule_applies_to_module(entry.rule_name, source, filename)
        && (linter.script_rule_overrides.contains_key(entry.rule_name)
            || script_rule_may_match(entry.rule_name, source))
}

/// Whether any enabled built-in script rule could match `source`.
///
/// Mirrors the per-rule `is_rule_enabled` + `script_rules.contains` +
/// `script_rule_may_match` gate so a block matching no rule is never parsed.
fn block_has_active_rule(
    linter: &Linter,
    vapor_component: bool,
    source: &str,
    filename: &str,
) -> bool {
    active_builtin_script_rule_entries(linter, vapor_component)
        .any(|entry| entry_may_match(linter, entry, source, filename))
}

/// Whether any enabled AST-based built-in script rule could match `source`.
fn block_has_active_ast_rule(
    linter: &Linter,
    vapor_component: bool,
    source: &str,
    filename: &str,
) -> bool {
    active_builtin_script_rule_entries(linter, vapor_component).any(|entry| {
        resolved_rule(linter, entry).uses_ast() && entry_may_match(linter, entry, source, filename)
    })
}

/// Run a single built-in script rule against a script block.
///
/// AST rules consume the shared parse when available and receive the
/// cross-block `sfc` context (empty outside SFC linting). Byte rules run
/// their source-level `check`, preserving the same rule-major ordering.
#[expect(clippy::too_many_arguments, reason = "explicit per-block state")]
fn run_builtin_script_rule(
    linter: &Linter,
    entry: &BuiltinScriptRuleEntry,
    rule: &dyn crate::rules::script::ScriptRule,
    source: &str,
    offset: usize,
    parsed: Option<&oxc_parser::ParserReturn<'_>>,
    sfc: SfcScriptContext<'_>,
    emits_module: Option<EmitsModuleScript<'_>>,
    dupe_module: Option<NoDupeKeysModuleScript<'_>>,
    runtime_props_eligible: bool,
    result: &mut LintResult,
) {
    if !entry_may_match(linter, entry, source, result.filename.as_str()) {
        return;
    }
    let mut lint = ScriptLintResult::default();
    if rule.uses_ast() {
        let Some(parsed) = parsed else {
            return;
        };
        if parsed.panicked || !parsed.diagnostics.is_empty() {
            return;
        }
        if let Some(module) =
            emits_module.filter(|_| entry.rule_name == RULE_RETURN_IN_EMITS_VALIDATOR)
        {
            if let Some(rule) = rule.as_setup_emits_validator() {
                profile!(
                    entry.profile_name,
                    rule.check_setup_with_module(
                        &parsed.program,
                        source,
                        offset,
                        module,
                        &mut lint
                    )
                );
            }
        } else if let Some(module) = dupe_module.filter(|_| entry.rule_name == RULE_NO_DUPE_KEYS)
            && let Some(dupe_rule) = rule.as_setup_dupe_keys()
        {
            profile!(
                entry.profile_name,
                dupe_rule.check_setup_with_module(
                    &parsed.program,
                    source,
                    offset,
                    module,
                    &mut lint
                )
            );
        } else if let Some(props_rule) = rule.as_setup_props_destructure()
            && entry.rule_name == props_rule.meta().name
        {
            if runtime_props_eligible {
                profile!(
                    entry.profile_name,
                    props_rule.check_setup(&parsed.program, source, offset, &mut lint)
                );
            }
        } else if runtime_props_eligible
            && entry.rule_name == RULE_NO_REQUIRED_PROP_WITH_DEFAULT
            && let Some(props_rule) = rule.as_setup_props_defaults()
        {
            profile!(entry.profile_name, {
                rule.check_program_with_sfc(&parsed.program, source, offset, sfc, &mut lint);
                props_rule.check_setup_runtime(&parsed.program, offset, &mut lint);
            });
        } else {
            profile!(
                entry.profile_name,
                rule.check_program_with_sfc(&parsed.program, source, offset, sfc, &mut lint)
            );
        }
    } else {
        profile!(entry.profile_name, rule.check(source, offset, &mut lint));
    }
    merge_script_result(linter, result, lint);
}

pub(crate) fn append_builtin_script_diagnostics_from_html(
    linter: &Linter,
    source: &str,
    result: &mut LintResult,
) {
    if linter.script_rules.is_empty() || !has_active_builtin_script_rules(linter) {
        return;
    }

    for (script, offset) in extract_inline_scripts(source) {
        append_builtin_script_rules_for_source(linter, script, offset, result);
    }
}

fn merge_script_result(linter: &Linter, result: &mut LintResult, script_result: ScriptLintResult) {
    let overrides = &linter.severity_overrides;
    append_with_rule_overrides(result, script_result.diagnostics, overrides);
}

/// Run every enabled built-in script rule against a single script block.
///
/// Mirrors the previous per-rule flow exactly: each rule is gated on
/// `is_rule_enabled` + `script_rules.contains` and on its `script_rule_may_match`
/// byte prefilter, runs into its own [`ScriptLintResult`], and is merged in the
/// original rule order. Active AST rules share one oxc parse; byte rules run
/// directly against the source.
pub(crate) fn append_builtin_script_rules_for_source(
    linter: &Linter,
    source: &str,
    offset: usize,
    result: &mut LintResult,
) {
    // Skip work entirely when no enabled rule could match this block.
    if !block_has_active_rule(linter, linter.vapor_mode, source, result.filename.as_str()) {
        return;
    }

    let allocator = Allocator::default();
    let parsed =
        block_has_active_ast_rule(linter, linter.vapor_mode, source, result.filename.as_str())
            .then(|| {
                profile!(
                    "patina.script_rule.parse",
                    Parser::new(&allocator, source, script_source_type()).parse()
                )
            });

    for entry in active_builtin_script_rule_entries(linter, linter.vapor_mode) {
        let rule = resolved_rule(linter, entry);
        // Inline HTML scripts have no SFC template, so the context is empty.
        run_builtin_script_rule(
            linter,
            entry,
            rule,
            source,
            offset,
            parsed.as_ref(),
            SfcScriptContext::default(),
            None,
            None,
            false,
            result,
        );
    }
}
