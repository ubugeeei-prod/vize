use super::prefilter::{
    descriptor_scripts_may_match_ecosystem_rule, has_only_active_ecosystem_script_rules,
};
use super::registry::RULE_RETURN_IN_EMITS_VALIDATOR;
use super::template_context;
use super::{
    LintResult, Linter, active_builtin_script_rule_entries, block_has_active_ast_rule,
    block_has_active_rule, entry_may_match, has_active_builtin_script_rules, merge_script_result,
    resolved_rule, run_builtin_script_rule,
};
use crate::rules::script::{
    EmitsModuleScript, NoDuplicateAttrInheritance, ReturnInEmitsValidator, ScriptLintResult,
    SfcScriptContext, script_source_type,
};
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use vize_atelier_sfc::SfcDescriptor;
use vize_l0::profile;

/// Reuse the template pass's validated AST. A missing or fatally malformed
/// template supplies `None`, so partial trees never become script-rule evidence.
pub(crate) fn append_builtin_script_diagnostics<'a>(
    linter: &Linter,
    descriptor: &SfcDescriptor<'a>,
    result: &mut LintResult,
    template_ast: Option<(&vize_relief::RootNode<'_>, u32)>,
) {
    let registered_scripts = registered_sfc::has_active(linter);
    if (linter.script_rules.is_empty() || !has_active_builtin_script_rules(linter))
        && !registered_scripts
    {
        return;
    }
    // A template-only SFC has no script callback to carry this native rule.
    // Reuse the validated template AST before script-only byte prefilters; no
    // empty program, extra parse or configured namesake is admitted here.
    if descriptor.script.is_none() && descriptor.script_setup.is_none() {
        if template_context::descriptor_needs_template_ast(
            linter,
            descriptor,
            result.filename.as_str(),
        ) && let Some((root, offset)) = template_ast
        {
            let mut lint = ScriptLintResult::default();
            profile!(
                "patina.script_rule.no_duplicate_attr_inheritance",
                NoDuplicateAttrInheritance::check_scriptless_template(root, offset, &mut lint)
            );
            merge_script_result(linter, result, lint);
        }
        return;
    }
    if !registered_scripts
        && has_only_active_ecosystem_script_rules(linter)
        && !descriptor_scripts_may_match_ecosystem_rule(descriptor)
    {
        return;
    }
    let vapor_component = linter.vapor_mode
        || descriptor
            .script
            .as_ref()
            .is_some_and(|script| script.attrs.contains_key("vapor"))
        || descriptor
            .script_setup
            .as_ref()
            .is_some_and(|script| script.attrs.contains_key("vapor"));

    // Parse each block at most once and only when an active AST rule could
    // match it. Byte rules run directly against the source.
    let script_source = descriptor
        .script
        .as_ref()
        .map(|block| (block.content.as_ref(), block.loc.start));
    let script = script_source.filter(|(source, _)| {
        registered_scripts
            || block_has_active_rule(linter, vapor_component, source, result.filename.as_str())
    });
    let script_setup = descriptor
        .script_setup
        .as_ref()
        .map(|block| (block.content.as_ref(), block.loc.start))
        .filter(|(source, _)| {
            block_has_active_rule(linter, vapor_component, source, result.filename.as_str())
        });
    let setup_alloc = Allocator::default();
    let script_setup_parsed = script_setup
        .filter(|(source, _)| {
            block_has_active_ast_rule(linter, vapor_component, source, result.filename.as_str())
        })
        .map(|(source, _)| {
            let parsed = profile!(
                "patina.script_rule.parse",
                Parser::new(&setup_alloc, source, script_source_type()).parse()
            );
            parsed
        });

    // Only the actual active return-validator rule can request normal-script
    // ownership evidence. Other rules retain their existing dispatch and parse
    // conditions. Inspect the shared setup AST before forcing a sibling parse.
    let emits_setup_rule = active_builtin_script_rule_entries(linter, vapor_component)
        .find(|entry| entry.rule_name == RULE_RETURN_IN_EMITS_VALIDATOR)
        .filter(|entry| {
            script_setup.is_some_and(|(source, _)| {
                entry_may_match(linter, entry, source, result.filename.as_str())
            })
        })
        .and_then(|entry| resolved_rule(linter, entry).as_setup_emits_validator());
    // An external block's content cannot establish its loaded bindings. Keep
    // ordinary rule dispatch unchanged, but never infer macro ownership from
    // parsing an empty external sibling or a fallback setup body.
    let inline_emits_blocks = descriptor
        .script
        .as_ref()
        .is_none_or(|block| block.src.is_none())
        && descriptor
            .script_setup
            .as_ref()
            .is_none_or(|block| block.src.is_none());
    let needs_emits_module = inline_emits_blocks
        && emits_setup_rule.is_some()
        && script_setup_parsed.as_ref().is_some_and(|parsed| {
            !parsed.panicked
                && parsed.diagnostics.is_empty()
                && ReturnInEmitsValidator::setup_needs_module(&parsed.program)
        });
    let script_alloc = Allocator::default();
    let script_parsed = script_source
        .filter(|(source, _)| {
            registered_scripts || needs_emits_module || {
                block_has_active_ast_rule(linter, vapor_component, source, result.filename.as_str())
            }
        })
        .map(|(source, _)| {
            profile!(
                "patina.script_rule.parse",
                Parser::new(&script_alloc, source, script_source_type()).parse()
            )
        });
    let emits_module = match (inline_emits_blocks, script_source, script_parsed.as_ref()) {
        (false, _, _) => EmitsModuleScript::Unavailable,
        (true, None, _) => EmitsModuleScript::Absent,
        (true, Some((_, offset)), Some(parsed))
            if !parsed.panicked && parsed.diagnostics.is_empty() =>
        {
            EmitsModuleScript::Parsed {
                program: &parsed.program,
                offset,
            }
        }
        _ => EmitsModuleScript::Unavailable,
    };

    // Share raw template text and the validated AST with rules that correlate
    // script declarations and template evidence.
    let template_ast = template_ast.filter(|_| {
        template_context::descriptor_needs_template_ast(
            linter,
            descriptor,
            result.filename.as_str(),
        )
    });
    let sfc_context = SfcScriptContext {
        is_sfc: true,
        is_script_setup: false,
        template_source: descriptor
            .template
            .as_ref()
            .map(|block| block.content.as_ref()),
        template_root: template_ast.map(|(root, _)| root),
        template_offset: template_ast.map(|(_, offset)| offset),
        // Both blocks are linted separately below, so a whole-file conclusion
        // is only available to a rule when there is a single block to draw it
        // from. Computed from the descriptor rather than filtered bindings: a skipped block
        // still contributes declarations a rule would need to see.
        sole_script_block: descriptor.script.is_some() != descriptor.script_setup.is_some(),
    };

    for entry in active_builtin_script_rule_entries(linter, vapor_component) {
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
                None,
                false,
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
                SfcScriptContext {
                    is_script_setup: true,
                    ..sfc_context
                },
                emits_setup_rule.map(|_| emits_module),
                descriptor.script.is_none()
                    && descriptor
                        .script_setup
                        .as_ref()
                        .is_some_and(|block| block.src.is_none()),
                result,
            );
        }
    }
    if let Some((source, offset)) = script {
        registered_sfc::run(
            linter,
            source,
            offset,
            script_parsed.as_ref(),
            sfc_context,
            result,
        );
    }
}
mod registered_sfc;
