use super::{LintResult, Linter};
use crate::rules::script::{ScriptLintResult, script_source_type};
use memchr::memmem;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use vize_atelier_sfc::{SfcDescriptor, SfcParseOptions, parse_sfc};
use vize_carton::profile;

mod html_scripts;
mod registry;

use html_scripts::extract_inline_scripts;
pub use registry::BuiltinScriptRuleMeta;
use registry::{
    ALL_BUILTIN_SCRIPT_RULE_NAMES, BUILTIN_SCRIPT_RULES, BuiltinScriptRuleEntry,
    RULE_NO_RESTRICTED_GLOBALS, RULE_NO_RESTRICTED_MEMBERS, RULE_PINIA_PREFER_STORE_TO_REFS,
    RULE_PREFER_COMPUTED, RULE_VUE_ROUTER_PREFER_NAMED_PUSH, RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT,
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

#[cfg(test)]
#[inline]
pub(crate) const fn opt_in_script_rule_names() -> &'static [&'static str] {
    OPT_IN_SCRIPT_RULE_NAMES
}

#[inline]
pub(crate) fn has_active_builtin_script_rules(linter: &Linter) -> bool {
    active_builtin_script_rule_entries(linter).next().is_some()
}

fn active_builtin_script_rule_entries(
    linter: &Linter,
) -> impl Iterator<Item = &'static BuiltinScriptRuleEntry> + '_ {
    linter
        .script_rules
        .iter()
        .copied()
        .filter(|rule_name| linter.is_rule_enabled(rule_name))
        .filter_map(builtin_script_rule_entry)
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

pub(crate) fn lint_with_descriptor<'a>(
    linter: &Linter,
    filename: &str,
    descriptor: &SfcDescriptor<'a>,
) -> LintResult {
    let mut result = profile!(
        "patina.sfc.descriptor.template_lint",
        linter.lint_sfc_template_with_descriptor(filename, descriptor)
    );

    append_builtin_script_diagnostics(linter, descriptor, &mut result);
    result
}

pub(crate) fn append_builtin_script_diagnostics<'a>(
    linter: &Linter,
    descriptor: &SfcDescriptor<'a>,
    result: &mut LintResult,
) {
    if linter.script_rules.is_empty() || !has_active_builtin_script_rules(linter) {
        return;
    }
    if has_only_active_ecosystem_script_rules(linter)
        && !descriptor_scripts_may_match_ecosystem_rule(descriptor)
    {
        return;
    }

    // Parse each block at most once and only when an active AST rule could
    // match it. Byte rules run directly against the source.
    let script = descriptor
        .script
        .as_ref()
        .map(|block| (block.content.as_ref(), block.loc.start))
        .filter(|(source, _)| block_has_active_rule(linter, source));
    let script_alloc = Allocator::default();
    let script_parsed = script
        .filter(|(source, _)| block_has_active_ast_rule(linter, source))
        .map(|(source, _)| {
            let parsed = profile!(
                "patina.script_rule.parse",
                Parser::new(&script_alloc, source, script_source_type()).parse()
            );
            parsed
        });

    let script_setup = descriptor
        .script_setup
        .as_ref()
        .map(|block| (block.content.as_ref(), block.loc.start))
        .filter(|(source, _)| block_has_active_rule(linter, source));
    let setup_alloc = Allocator::default();
    let script_setup_parsed = script_setup
        .filter(|(source, _)| block_has_active_ast_rule(linter, source))
        .map(|(source, _)| {
            let parsed = profile!(
                "patina.script_rule.parse",
                Parser::new(&setup_alloc, source, script_source_type()).parse()
            );
            parsed
        });

    for entry in active_builtin_script_rule_entries(linter) {
        let rule = resolved_rule(linter, entry);
        if let Some((source, offset)) = script {
            run_builtin_script_rule(
                linter,
                entry,
                rule,
                source,
                offset,
                script_parsed.as_ref(),
                result,
            );
        }
        if let Some((source, offset)) = script_setup.filter(|_| rule.runs_on_script_setup()) {
            run_builtin_script_rule(
                linter,
                entry,
                rule,
                source,
                offset,
                script_setup_parsed.as_ref(),
                result,
            );
        }
    }
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

/// Whether `entry` could match `source`. A configured override bypasses the
/// byte prefilter (its deny list may reference identifiers the default prefilter
/// does not know about), so the block is always parsed for overridden rules.
#[inline]
fn entry_may_match(linter: &Linter, entry: &BuiltinScriptRuleEntry, source: &str) -> bool {
    linter.script_rule_overrides.contains_key(entry.rule_name)
        || script_rule_may_match(entry.rule_name, source)
}

/// Whether any enabled built-in script rule could match `source`.
///
/// Mirrors the per-rule `is_rule_enabled` + `script_rules.contains` +
/// `script_rule_may_match` gate so a block matching no rule is never parsed.
fn block_has_active_rule(linter: &Linter, source: &str) -> bool {
    active_builtin_script_rule_entries(linter).any(|entry| entry_may_match(linter, entry, source))
}

/// Whether any enabled AST-based built-in script rule could match `source`.
fn block_has_active_ast_rule(linter: &Linter, source: &str) -> bool {
    active_builtin_script_rule_entries(linter).any(|entry| {
        resolved_rule(linter, entry).uses_ast() && entry_may_match(linter, entry, source)
    })
}

/// Run a single built-in script rule against a script block.
///
/// AST rules consume the shared parse when available. Byte rules run their
/// source-level `check`, preserving the same rule-major ordering.
#[allow(clippy::too_many_arguments)]
fn run_builtin_script_rule(
    linter: &Linter,
    entry: &BuiltinScriptRuleEntry,
    rule: &dyn crate::rules::script::ScriptRule,
    source: &str,
    offset: usize,
    parsed: Option<&oxc_parser::ParserReturn<'_>>,
    result: &mut LintResult,
) {
    if !entry_may_match(linter, entry, source) {
        return;
    }
    let mut lint = ScriptLintResult::default();
    if rule.uses_ast() {
        let Some(parsed) = parsed else {
            return;
        };
        if parsed.panicked || !parsed.errors.is_empty() {
            return;
        }
        profile!(
            entry.profile_name,
            rule.check_program(&parsed.program, source, offset, &mut lint)
        );
    } else {
        profile!(entry.profile_name, rule.check(source, offset, &mut lint));
    }
    merge_script_result(result, lint);
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

fn merge_script_result(result: &mut LintResult, script_result: ScriptLintResult) {
    result.error_count += script_result.error_count;
    result.warning_count += script_result.warning_count;
    result.diagnostics.extend(script_result.diagnostics);
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
    if !block_has_active_rule(linter, source) {
        return;
    }

    let allocator = Allocator::default();
    let parsed = block_has_active_ast_rule(linter, source).then(|| {
        profile!(
            "patina.script_rule.parse",
            Parser::new(&allocator, source, script_source_type()).parse()
        )
    });

    for entry in active_builtin_script_rule_entries(linter) {
        let rule = resolved_rule(linter, entry);
        run_builtin_script_rule(linter, entry, rule, source, offset, parsed.as_ref(), result);
    }
}

fn script_rule_may_match(rule_name: &str, source: &str) -> bool {
    let bytes = source.as_bytes();
    match rule_name {
        RULE_PINIA_PREFER_STORE_TO_REFS => memmem::find(bytes, b"Store").is_some(),
        RULE_VUE_ROUTER_PREFER_NAMED_PUSH => {
            (memmem::find(bytes, b".push").is_some() || memmem::find(bytes, b".replace").is_some())
                && (memmem::find(bytes, b"'/").is_some() || memmem::find(bytes, b"\"/").is_some())
                && (memmem::find(bytes, b"router").is_some()
                    || memmem::find(bytes, b"Router").is_some())
        }
        RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT => {
            memmem::find(bytes, b"toMatchSnapshot").is_some()
                && memmem::find(bytes, b".html").is_some()
        }
        RULE_PREFER_COMPUTED => memmem::find(bytes, b"watch").is_some(),
        RULE_NO_RESTRICTED_GLOBALS => {
            memmem::find(bytes, b"process").is_some()
                || memmem::find(bytes, b"localStorage").is_some()
                || memmem::find(bytes, b"sessionStorage").is_some()
        }
        // The static singleton has an empty deny list and never fires; configured
        // instances bypass this prefilter via `entry_may_match`.
        RULE_NO_RESTRICTED_MEMBERS => false,
        _ => true,
    }
}

fn descriptor_scripts_may_match_ecosystem_rule(descriptor: &SfcDescriptor<'_>) -> bool {
    descriptor
        .script
        .as_ref()
        .is_some_and(|script| source_may_match_ecosystem_rule(script.content.as_ref()))
        || descriptor
            .script_setup
            .as_ref()
            .is_some_and(|script| source_may_match_ecosystem_rule(script.content.as_ref()))
}

fn is_ecosystem_script_rule(rule_name: &str) -> bool {
    matches!(
        rule_name,
        RULE_PINIA_PREFER_STORE_TO_REFS
            | RULE_VUE_ROUTER_PREFER_NAMED_PUSH
            | RULE_VUE_TEST_UTILS_NO_HTML_SNAPSHOT
    )
}

fn has_only_active_ecosystem_script_rules(linter: &Linter) -> bool {
    active_builtin_script_rule_entries(linter)
        .all(|entry| is_ecosystem_script_rule(entry.rule_name))
}

fn source_may_match_ecosystem_rule(source: &str) -> bool {
    let bytes = source.as_bytes();
    memmem::find(bytes, b"Store").is_some()
        || ((memmem::find(bytes, b".push").is_some() || memmem::find(bytes, b".replace").is_some())
            && (memmem::find(bytes, b"'/").is_some() || memmem::find(bytes, b"\"/").is_some())
            && (memmem::find(bytes, b"router").is_some()
                || memmem::find(bytes, b"Router").is_some()))
        || (memmem::find(bytes, b"toMatchSnapshot").is_some()
            && memmem::find(bytes, b".html").is_some())
}
