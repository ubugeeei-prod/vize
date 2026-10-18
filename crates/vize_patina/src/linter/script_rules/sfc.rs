use super::prefilter::{
    descriptor_scripts_may_match_ecosystem_rule, has_only_active_ecosystem_script_rules,
};
use super::registry::{RULE_NO_DUPE_KEYS, RULE_RETURN_IN_EMITS_VALIDATOR};
use super::template_context;
use super::{
    LintResult, Linter, active_builtin_script_rule_entries, block_has_active_ast_rule,
    block_has_active_rule, entry_may_match, has_active_builtin_script_rules, merge_script_result,
    resolved_rule, run_builtin_script_rule,
};
use crate::rules::script::{
    EmitsModuleScript, NoDupeKeys, NoDupeKeysModuleScript, NoDuplicateAttrInheritance,
    ReturnInEmitsValidator, ScriptLintResult, SfcScriptContext, script_source_type,
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
    template_syntax: Option<&vize_armature::parser::TemplateSyntaxFacts<'_>>,
) {
    append_builtin_script_diagnostics_with_template_context(
        linter,
        descriptor,
        result,
        template_ast,
        template_syntax,
        None,
    );
}

pub(crate) fn append_builtin_script_diagnostics_with_template_context<'a>(
    linter: &Linter,
    descriptor: &SfcDescriptor<'a>,
    result: &mut LintResult,
    template_ast: Option<(&vize_relief::RootNode<'_>, u32)>,
    template_syntax: Option<&vize_armature::parser::TemplateSyntaxFacts<'_>>,
    mut template_context: Option<&mut crate::context::LintContext<'_>>,
) {
    let registered_scripts = registered_sfc::has_active(linter);
    if (linter.script_rules.is_empty() || !has_active_builtin_script_rules(linter))
        && !registered_scripts
    {
        return;
    }
    let template_references = template_ast
        .zip(template_syntax)
        .filter(|((root, offset), _)| {
            descriptor.template.as_ref().is_some_and(|template| {
                *offset == template.loc.start as u32
                    && core::ptr::eq(root.source, template.content.as_ref())
            })
        })
        .map(|((root, offset), syntax)| {
            (
                vize_croquis::drawer::collect_template_references(root, syntax),
                offset,
            )
        });
    // Resolve actual configured capabilities once. Script-bearing SFCs defer
    // emission until the existing clean script parses can positively witness a
    // component binding; scriptless/early returns still run the capability.
    let mut append_template =
        |result: &mut LintResult, bindings: &crate::rules::script::TemplateComponentBindings| {
            let Some((references, offset)) = &template_references else {
                return;
            };
            for entry in active_builtin_script_rule_entries(linter, true) {
                if let Some(rule) = resolved_rule(linter, entry).template_check() {
                    let mut lint = ScriptLintResult::default();
                    rule.check_template(references, bindings, *offset, &mut lint);
                    if let Some(ctx) = template_context.as_deref_mut() {
                        if linter.report_unused_disable_directives {
                            ctx.register_executed_template_rule(entry.rule_name);
                        }
                        for diagnostic in lint.diagnostics {
                            ctx.report_template_in_sfc(diagnostic);
                        }
                    } else {
                        merge_script_result(linter, result, lint);
                    }
                }
            }
        };
    // A template-only SFC has no script callback to carry this native rule.
    // Reuse the validated template AST before script-only byte prefilters; no
    // empty program, extra parse or configured namesake is admitted here.
    if descriptor.script.is_none() && descriptor.script_setup.is_none() {
        append_template(result, &Default::default());
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
        append_template(
            result,
            &crate::rules::script::TemplateComponentBindings::from_parsed_sfc(
                descriptor, None, None,
            ),
        );
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
                Parser::new(
                    &setup_alloc,
                    source,
                    sfc_script_source_type(
                        descriptor
                            .script_setup
                            .as_ref()
                            .and_then(|block| block.lang.as_deref())
                    )
                )
                .parse()
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
    let dupe_setup_rule = active_builtin_script_rule_entries(linter, vapor_component)
        .find(|entry| entry.rule_name == RULE_NO_DUPE_KEYS)
        .filter(|entry| {
            script_setup.is_some_and(|(source, _)| {
                entry_may_match(linter, entry, source, result.filename.as_str())
            })
        })
        .and_then(|entry| resolved_rule(linter, entry).as_setup_dupe_keys());
    let needs_dupe_module = inline_emits_blocks
        && dupe_setup_rule.is_some()
        && script_setup_parsed.as_ref().is_some_and(|parsed| {
            !parsed.panicked
                && parsed.diagnostics.is_empty()
                && NoDupeKeys::setup_needs_module(&parsed.program)
        });
    let script_alloc = Allocator::default();
    let script_parsed = script_source
        .filter(|(source, _)| {
            registered_scripts || needs_emits_module || needs_dupe_module || {
                block_has_active_ast_rule(linter, vapor_component, source, result.filename.as_str())
            }
        })
        .map(|(source, _)| {
            profile!(
                "patina.script_rule.parse",
                Parser::new(
                    &script_alloc,
                    source,
                    sfc_script_source_type(
                        descriptor
                            .script
                            .as_ref()
                            .and_then(|block| block.lang.as_deref())
                    )
                )
                .parse()
            )
        });
    let needs_component_bindings = template_references.as_ref().is_some_and(|(references, _)| {
        references.references().iter().any(|reference| {
            reference.owner == vize_croquis::drawer::TemplateReferenceOwner::ComponentContext
                && matches!(reference.name.as_str(), "$listeners" | "$scopedSlots")
        })
    });
    let component_bindings = if needs_component_bindings {
        crate::rules::script::TemplateComponentBindings::from_parsed_sfc(
            descriptor,
            script_setup_parsed.as_ref(),
            script_parsed.as_ref(),
        )
    } else {
        Default::default()
    };
    append_template(result, &component_bindings);
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

    let dupe_module = match (inline_emits_blocks, script_source, script_parsed.as_ref()) {
        (false, _, _) => NoDupeKeysModuleScript::Unavailable,
        (true, None, _) => NoDupeKeysModuleScript::Absent,
        (true, Some((_, offset)), Some(parsed))
            if !parsed.panicked && parsed.diagnostics.is_empty() =>
        {
            NoDupeKeysModuleScript::Parsed {
                program: &parsed.program,
                offset,
            }
        }
        _ => NoDupeKeysModuleScript::Unavailable,
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
        sibling_program: None,
    };

    let setup_program = script_setup_parsed
        .as_ref()
        .filter(|parsed| !parsed.panicked && parsed.diagnostics.is_empty())
        .map(|parsed| &parsed.program);
    let normal_program = script_parsed
        .as_ref()
        .filter(|parsed| !parsed.panicked && parsed.diagnostics.is_empty())
        .map(|parsed| &parsed.program);
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
                SfcScriptContext {
                    sibling_program: setup_program,
                    ..sfc_context
                },
                None,
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
                    sibling_program: normal_program,
                    ..sfc_context
                },
                emits_setup_rule.map(|_| emits_module),
                dupe_setup_rule.map(|_| dupe_module),
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

// JSX render functions are valid in JavaScript SFC blocks. Keep TypeScript
// blocks in TS mode so generic arrows and angle-bracket assertions retain
// their grammar; only an explicit TSX block opts those files into JSX.
fn sfc_script_source_type(lang: Option<&str>) -> oxc_span::SourceType {
    script_source_type().with_jsx(matches!(
        lang,
        None | Some("js" | "javascript" | "jsx" | "tsx")
    ))
}

#[cfg(test)]
mod tests {
    use crate::rules::script::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
    use crate::{LintDiagnostic, LintPreset, Linter, Severity};
    use serde_json::json;

    struct ScriptOnly;
    static META: ScriptRuleMeta = ScriptRuleMeta {
        name: "script/no-deprecated-dollar-scopedslots-api",
        description: "configured script-only capability witness",
        default_severity: Severity::Warning,
    };
    impl ScriptRule for ScriptOnly {
        fn meta(&self) -> &'static ScriptRuleMeta {
            &META
        }
        fn check(&self, source: &str, offset: usize, result: &mut ScriptLintResult) {
            assert_eq!(source, "const marker = 1;");
            result.add_diagnostic(LintDiagnostic::warn(
                META.name,
                "script-only override",
                offset as u32,
                offset as u32 + 5,
            ));
        }
    }

    #[test]
    fn a_script_only_override_never_inherits_template_pragma_usage() {
        assert!(ScriptOnly.template_check().is_none());
        let source = "<script>const marker = 1;</script>\n<template>\n<!-- eslint-disable-next-line vue/no-deprecated-dollar-scopedslots-api -->\n<div>{{ $scopedSlots }}</div>\n</template>\n";
        let mut linter = Linter::with_preset(LintPreset::Incremental)
            .with_enabled_rules(Some(vec![META.name.into()]))
            .with_report_unused_disable_directives(true);
        linter
            .script_rule_overrides
            .insert(META.name.into(), Box::new(ScriptOnly));
        let result = linter.lint_sfc(source, "Override.vue");
        let diagnostics = result.diagnostics.iter().map(|d| json!({
            "rule_name":d.rule_name, "severity":d.severity, "message":d.message,
            "start":d.start, "end":d.end, "help":d.help, "fix":d.fix,
            "labels":d.labels.iter().map(|l| json!({"message":l.message,"start":l.start,"end":l.end})).collect::<Vec<_>>()
        })).collect::<Vec<_>>();
        assert_eq!(
            json!({"filename":result.filename,"error_count":result.error_count,"warning_count":result.warning_count,"diagnostics":diagnostics}),
            json!({
                "filename":"Override.vue","error_count":0,"warning_count":1,
                "diagnostics":[{"rule_name":META.name,"severity":"warning","message":"script-only override","start":8,"end":13,"help":null,"fix":null,"labels":[]}]
            })
        );
    }
}
