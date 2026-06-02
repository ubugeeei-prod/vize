use super::{
    LintResult, Linter, RULE_NO_FLOATING_PROMISES, RULE_NO_REACTIVITY_LOSS,
    RULE_NO_UNSAFE_TEMPLATE_BINDING, RULE_REQUIRE_TYPED_EMITS, RULE_REQUIRE_TYPED_PROPS,
    has_promise_like_return, has_unsafe_template_type, push_warning,
    should_warn_for_emit_validator, should_warn_for_prop_access, should_warn_for_reactivity_loss,
    with_corsa_session,
};
use crate::diagnostic::LintDiagnostic;
use std::path::Path;
use vize_armature::Parser as TemplateParser;
use vize_carton::{FxHashSet, profile};
use vize_croquis::{
    Croquis, script_parser,
    virtual_ts::{VirtualTsConfig, generate_virtual_ts_with_croquis},
};

use super::{
    markers::{QueryKind, push_promise_marker},
    parsing::{collect_floating_candidates, is_runtime_array_macro},
    reactivity_loss::collect_reactivity_loss_queries,
    rule_queries::{MacroWarning, collect_emit_queries, collect_prop_queries, push_macro_warning},
    template_queries::{TemplateQueryKind, collect_template_query_sets},
};

pub(super) fn lint_with_descriptor<'a>(
    linter: &Linter,
    source: &str,
    filename: &str,
    descriptor: &vize_atelier_sfc::SfcDescriptor<'a>,
) -> LintResult {
    let allocator =
        vize_carton::Allocator::with_capacity((source.len() * 4).max(linter.initial_capacity));
    let template_ast = descriptor.template.as_ref().map(|template| {
        let parser = TemplateParser::new(allocator.as_bump(), &template.content);
        let (root, _) = profile!("patina.type_aware.template_parse", parser.parse());
        (root, template.loc.start as u32)
    });

    let analysis = profile!("patina.type_aware.croquis", {
        super::super::engine::analyze_descriptor_for_lint(
            descriptor,
            template_ast.as_ref().map(|(root, _)| root),
        )
    });

    let mut result = if let (Some((root, _)), Some(template)) =
        (template_ast.as_ref(), descriptor.template.as_ref())
    {
        profile!(
            "patina.type_aware.template_rules",
            linter.lint_sfc_template_root(
                filename,
                template,
                &allocator,
                root,
                Some(descriptor),
                Some(&analysis),
            )
        )
    } else {
        LintResult {
            filename: filename.into(),
            diagnostics: Vec::new(),
            error_count: 0,
            warning_count: 0,
        }
    };
    super::super::script_rules::append_builtin_script_diagnostics(linter, descriptor, &mut result);

    let Some(script_block) = descriptor
        .script_setup
        .as_ref()
        .or(descriptor.script.as_ref())
    else {
        return result;
    };

    let script_content = script_block.content.as_ref();
    if script_content.is_empty() {
        return result;
    }

    // Plan every type-aware probe before generating virtual TS. Most files have
    // enough static macro information to report or skip immediately; if no active
    // rule needs Corsa, returning here avoids virtual project creation and the
    // expensive type-probe round trip entirely.
    let needs_prop_probe = profile!("patina.type_aware.plan_prop_queries", {
        collect_prop_static_warning_or_probe_need(linter, &analysis, &mut result, script_block)
    });
    let needs_emit_probe = profile!("patina.type_aware.plan_emit_queries", {
        collect_emit_static_warning_or_probe_need(linter, &analysis, &mut result, script_block)
    });
    let include_template_queries = is_type_rule_active(linter, RULE_NO_UNSAFE_TEMPLATE_BINDING);
    let include_template_promise_queries = is_type_rule_active(linter, RULE_NO_FLOATING_PROMISES);
    let include_reactivity_queries = is_type_rule_active(linter, RULE_NO_REACTIVITY_LOSS);
    if !needs_prop_probe
        && !needs_emit_probe
        && !include_template_queries
        && !include_template_promise_queries
        && !include_reactivity_queries
    {
        return result;
    }

    let template_offset = template_ast
        .as_ref()
        .map(|(_, offset)| *offset)
        .unwrap_or(0);
    let from_file = Path::new(filename).parent();
    let parse_result = profile!("patina.type_aware.script_parse", {
        if let Some(script_setup) = descriptor.script_setup.as_ref() {
            let generic = script_setup
                .attrs
                .get("generic")
                .map(|value| value.as_ref());
            script_parser::parse_script_setup_with_generic(script_content, generic)
        } else {
            script_parser::parse_script(script_content)
        }
    });
    let config = VirtualTsConfig {
        script_offset: script_block.loc.start as u32,
        template_offset,
        ..Default::default()
    };
    let mut virtual_ts = profile!(
        "patina.type_aware.virtual_ts",
        generate_virtual_ts_with_croquis(
            script_content,
            &parse_result,
            template_ast.as_ref().map(|(root, _)| root),
            &config,
            None,
            from_file,
        )
    );

    let mut macro_queries = Vec::new();
    if needs_prop_probe {
        profile!(
            "patina.type_aware.collect_prop_queries",
            collect_prop_queries(
                linter,
                &analysis,
                &mut result,
                script_block,
                &mut virtual_ts,
                &mut macro_queries,
            )
        )
    }
    if needs_emit_probe {
        profile!(
            "patina.type_aware.collect_emit_queries",
            collect_emit_queries(
                linter,
                &analysis,
                &mut result,
                script_block,
                &mut virtual_ts,
                &mut macro_queries,
            )
        );
    }

    if include_template_promise_queries {
        for candidate in profile!(
            "patina.type_aware.collect_floating_candidates",
            collect_floating_candidates(script_content)
        ) {
            push_promise_marker(
                &mut virtual_ts,
                script_content,
                candidate.start,
                candidate.end,
                &mut macro_queries,
            );
        }
    }

    let reactivity_loss_queries = profile!("patina.type_aware.collect_reactivity_loss_queries", {
        collect_reactivity_loss_queries(
            linter,
            &mut result,
            &parse_result,
            script_content,
            script_block.loc.start as u32,
            &mut virtual_ts,
        )
    });

    // Template type rules share one AST walk. The collector fans out into
    // ordinary unsafe-binding queries and Promise-return queries through optional
    // sinks, so enabling both rules does not double the traversal or reparses of
    // individual template expressions.
    let (template_queries, template_promise_queries) =
        profile!("patina.type_aware.collect_template_query_sets", {
            if include_template_queries || include_template_promise_queries {
                template_ast.as_ref().map_or_else(
                    || (Vec::new(), Vec::new()),
                    |(root, _)| {
                        collect_template_query_sets(
                            &virtual_ts,
                            root,
                            template_offset,
                            include_template_queries,
                            include_template_promise_queries,
                        )
                    },
                )
            } else {
                (Vec::new(), Vec::new())
            }
        });

    if macro_queries.is_empty()
        && template_queries.is_empty()
        && template_promise_queries.is_empty()
        && reactivity_loss_queries.is_empty()
    {
        return result;
    }

    let mut should_warn_for_props = false;
    let mut should_warn_for_emits = false;
    let mut warned_template_owners = FxHashSet::default();
    let mut warned_reactivity_loss_owners = FxHashSet::default();
    let corsa_result = profile!(
        "patina.type_aware.corsa_session",
        with_corsa_session(linter, filename, |session| {
            profile!(
                "patina.type_aware.corsa.open_virtual_project",
                session.open_virtual_project(&virtual_ts.content)
            )?;
            for query in &macro_queries {
                let probe = profile!(
                    "patina.type_aware.corsa.probe_macro",
                    session.probe_type_at_offset(
                        &virtual_ts.content,
                        query.generated_offset,
                        false,
                        matches!(query.kind, QueryKind::EmitValidator | QueryKind::Promise),
                    )
                )?;

                match query.kind {
                    QueryKind::PropType => {
                        should_warn_for_props |= should_warn_for_prop_access(probe.as_ref());
                    }
                    QueryKind::EmitValidator => {
                        should_warn_for_emits |= should_warn_for_emit_validator(probe.as_ref());
                    }
                    QueryKind::Promise => {
                        if let Some(probe) = probe.as_ref()
                            && (has_promise_like_return(probe)
                                || corsa::utils::is_promise_like_type_texts(
                                    &probe.type_texts,
                                    &probe.property_names,
                                ))
                        {
                            push_warning(
                                &mut result,
                                LintDiagnostic::warn(
                                    RULE_NO_FLOATING_PROMISES,
                                    "Floating Promise must be awaited, returned, or explicitly ignored with `void`",
                                    script_block.loc.start as u32 + query.source_start,
                                    script_block.loc.start as u32 + query.source_end,
                                )
                                .with_help(
                                    "Add `await`, return the Promise, or prefix it with `void` when the fire-and-forget behavior is intentional.",
                                ),
                            );
                        }
                    }
                }
            }

            for query in &template_queries {
                let probe = profile!(
                    "patina.type_aware.corsa.probe_template",
                    session.probe_type_at_offset(
                        &virtual_ts.content,
                        query.generated_offset,
                        false,
                        false,
                    )
                )?;
                if !has_unsafe_template_type(probe.as_ref()) {
                    continue;
                }

                let owner_key = query.owner_key();
                if matches!(query.kind, TemplateQueryKind::Expression)
                    && warned_template_owners.contains(&owner_key)
                {
                    continue;
                }

                push_warning(&mut result, query.diagnostic());
                if matches!(query.kind, TemplateQueryKind::CallCallee) {
                    warned_template_owners.insert(owner_key);
                }
            }

            for query in &template_promise_queries {
                let probe = profile!(
                    "patina.type_aware.corsa.probe_template_promise",
                    session.probe_type_at_offset(
                        &virtual_ts.content,
                        query.generated_offset,
                        false,
                        true,
                    )
                )?;
                let Some(probe) = probe.as_ref() else {
                    continue;
                };
                if has_promise_like_return(probe)
                    || corsa::utils::is_promise_like_type_texts(
                        &probe.type_texts,
                        &probe.property_names,
                    )
                {
                    push_warning(&mut result, query.diagnostic());
                }
            }

            for query in &reactivity_loss_queries {
                let probe = profile!(
                    "patina.type_aware.corsa.probe_reactivity_loss",
                    session.probe_type_at_offset(
                        &virtual_ts.content,
                        query.generated_offset,
                        false,
                        false,
                    )
                )?;
                if !should_warn_for_reactivity_loss(probe.as_ref()) {
                    continue;
                }

                let owner_key = query.owner_key();
                if warned_reactivity_loss_owners.insert(owner_key) {
                    push_warning(&mut result, query.diagnostic(script_block.loc.start as u32));
                }
            }
            Ok(())
        })
    );
    if let Err(error) = corsa_result {
        push_warning(
            &mut result,
            LintDiagnostic::warn("type/corsa-runtime", error, 0, 0).with_help(
                "Type-aware lint rules were skipped because the Corsa runtime could not be started. Configure `typeChecker.corsaPath` or install `@typescript/native-preview`.",
            ),
        );
    }

    push_macro_warning(
        &mut result,
        &macro_queries,
        MacroWarning {
            kind: QueryKind::PropType,
            base_offset: script_block.loc.start as u32,
            rule_name: RULE_REQUIRE_TYPED_PROPS,
            message: "Prop should have a type definition",
            help: "Use `defineProps<Props>()` or a runtime prop object with concrete constructor types.",
            should_warn: should_warn_for_props,
        },
    );
    push_macro_warning(
        &mut result,
        &macro_queries,
        MacroWarning {
            kind: QueryKind::EmitValidator,
            base_offset: script_block.loc.start as u32,
            rule_name: RULE_REQUIRE_TYPED_EMITS,
            message: "Emit should have a type definition",
            help: "Use `defineEmits<...>()` or a validator object with typed payload parameters.",
            should_warn: should_warn_for_emits,
        },
    );

    result
}

#[inline]
fn is_type_rule_active(linter: &Linter, rule_name: &str) -> bool {
    linter.registry.has_rule(rule_name) && linter.is_rule_enabled(rule_name)
}

fn collect_prop_static_warning_or_probe_need(
    linter: &Linter,
    analysis: &Croquis,
    result: &mut LintResult,
    script_block: &vize_atelier_sfc::SfcScriptBlock<'_>,
) -> bool {
    if !is_type_rule_active(linter, RULE_REQUIRE_TYPED_PROPS) {
        return false;
    }

    let Some(call) = analysis.macros.define_props() else {
        return false;
    };
    if call.type_args.is_some() {
        return false;
    }

    if is_runtime_array_macro(call.runtime_args.as_ref().map(|args| args.as_str())) {
        push_warning(
            result,
            LintDiagnostic::warn(
                RULE_REQUIRE_TYPED_PROPS,
                "Prop should have a type definition",
                script_block.loc.start as u32 + call.start,
                script_block.loc.start as u32 + call.end,
            )
            .with_help(
                "Use `defineProps<Props>()` or a runtime prop object with concrete constructor types.",
            ),
        );
        return false;
    }

    analysis
        .macros
        .props()
        .iter()
        .any(|prop| prop.prop_type.is_none())
}

fn collect_emit_static_warning_or_probe_need(
    linter: &Linter,
    analysis: &Croquis,
    result: &mut LintResult,
    script_block: &vize_atelier_sfc::SfcScriptBlock<'_>,
) -> bool {
    if !is_type_rule_active(linter, RULE_REQUIRE_TYPED_EMITS) {
        return false;
    }

    let Some(call) = analysis.macros.define_emits() else {
        return false;
    };
    if call.type_args.is_some() {
        return false;
    }

    if is_runtime_array_macro(call.runtime_args.as_ref().map(|args| args.as_str())) {
        push_warning(
            result,
            LintDiagnostic::warn(
                RULE_REQUIRE_TYPED_EMITS,
                "Emit should have a type definition",
                script_block.loc.start as u32 + call.start,
                script_block.loc.start as u32 + call.end,
            )
            .with_help(
                "Use `defineEmits<...>()` or a validator object with typed payload parameters.",
            ),
        );
        return false;
    }

    analysis
        .macros
        .emits()
        .iter()
        .any(|emit| emit.payload_type.is_none())
}
