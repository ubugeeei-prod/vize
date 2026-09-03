use super::{LintResult, Linter, severity::append_with_rule_overrides};
use crate::rules::script::{ScriptLintResult, SfcScriptContext, script_source_type};
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use vize_atelier_sfc::{SfcDescriptor, SfcParseOptions, parse_sfc};
use vize_s0::profile;

mod html_scripts;
mod prefilter;
mod registry;
mod template_ast;
mod template_context;

use html_scripts::extract_inline_scripts;
use prefilter::{
    descriptor_scripts_may_match_ecosystem_rule, has_only_active_ecosystem_script_rules,
    script_rule_applies_to_filename, script_rule_may_match,
};
pub use registry::BuiltinScriptRuleMeta;
use registry::{ALL_BUILTIN_SCRIPT_RULE_NAMES, BUILTIN_SCRIPT_RULES, BuiltinScriptRuleEntry};

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
        .filter(|(source, _)| block_has_active_rule(linter, source, result.filename.as_str()));
    let script_alloc = Allocator::default();
    let script_parsed = script
        .filter(|(source, _)| block_has_active_ast_rule(linter, source, result.filename.as_str()))
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
        .filter(|(source, _)| block_has_active_rule(linter, source, result.filename.as_str()));
    let setup_alloc = Allocator::default();
    let script_setup_parsed = script_setup
        .filter(|(source, _)| block_has_active_ast_rule(linter, source, result.filename.as_str()))
        .map(|(source, _)| {
            let parsed = profile!(
                "patina.script_rule.parse",
                Parser::new(&setup_alloc, source, script_source_type()).parse()
            );
            parsed
        });

    // Cross-block context shared by every rule invocation for this SFC: rules
    // that correlate script declarations with template usage read the raw
    // `<template>` source (`script/no-unused-emit-declarations`, where an
    // over-match only suppresses) or its parsed AST (rules that *create* a
    // finding from template evidence, where an over-match would be a false
    // positive). The AST is parsed at most once.
    let template_allocator = vize_s0::Allocator::default();
    let template_ast = template_context::descriptor_needs_template_ast(
        linter,
        descriptor,
        result.filename.as_str(),
    )
    .then(|| template_ast::parse_for_script_rules(linter, descriptor, &template_allocator))
    .flatten();
    let sfc_context = SfcScriptContext {
        template_source: descriptor
            .template
            .as_ref()
            .map(|block| block.content.as_ref()),
        template_root: template_ast.as_ref().map(|ast| &ast.root),
        template_offset: template_ast.as_ref().map(|ast| ast.offset),
        // Both blocks are linted separately below, so a whole-file conclusion
        // is only available to a rule when there is a single block to draw it
        // from. Computed from the descriptor rather than from the filtered
        // `script` / `script_setup` bindings: a block skipped by the prefilter
        // still contributes declarations a rule would need to see.
        sole_script_block: descriptor.script.is_some() != descriptor.script_setup.is_some(),
    };

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
                sfc_context,
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
                sfc_context,
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
fn entry_may_match(
    linter: &Linter,
    entry: &BuiltinScriptRuleEntry,
    source: &str,
    filename: &str,
) -> bool {
    script_rule_applies_to_filename(entry.rule_name, filename)
        && (linter.script_rule_overrides.contains_key(entry.rule_name)
            || script_rule_may_match(entry.rule_name, source))
}

/// Whether any enabled built-in script rule could match `source`.
///
/// Mirrors the per-rule `is_rule_enabled` + `script_rules.contains` +
/// `script_rule_may_match` gate so a block matching no rule is never parsed.
fn block_has_active_rule(linter: &Linter, source: &str, filename: &str) -> bool {
    active_builtin_script_rule_entries(linter)
        .any(|entry| entry_may_match(linter, entry, source, filename))
}

/// Whether any enabled AST-based built-in script rule could match `source`.
fn block_has_active_ast_rule(linter: &Linter, source: &str, filename: &str) -> bool {
    active_builtin_script_rule_entries(linter).any(|entry| {
        resolved_rule(linter, entry).uses_ast() && entry_may_match(linter, entry, source, filename)
    })
}

/// Run a single built-in script rule against a script block.
///
/// AST rules consume the shared parse when available and receive the
/// cross-block `sfc` context (empty outside SFC linting). Byte rules run
/// their source-level `check`, preserving the same rule-major ordering.
#[allow(clippy::too_many_arguments)]
fn run_builtin_script_rule(
    linter: &Linter,
    entry: &BuiltinScriptRuleEntry,
    rule: &dyn crate::rules::script::ScriptRule,
    source: &str,
    offset: usize,
    parsed: Option<&oxc_parser::ParserReturn<'_>>,
    sfc: SfcScriptContext<'_>,
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
        profile!(
            entry.profile_name,
            rule.check_program_with_sfc(&parsed.program, source, offset, sfc, &mut lint)
        );
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
    if !block_has_active_rule(linter, source, result.filename.as_str()) {
        return;
    }

    let allocator = Allocator::default();
    let parsed = block_has_active_ast_rule(linter, source, result.filename.as_str()).then(|| {
        profile!(
            "patina.script_rule.parse",
            Parser::new(&allocator, source, script_source_type()).parse()
        )
    });

    for entry in active_builtin_script_rule_entries(linter) {
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
            result,
        );
    }
}
