mod css;
mod planning;
mod prepared;
mod probes;

#[cfg(test)]
mod tests;

use super::super::engine::{SfcTemplateLintInput, TemplateAnalysis};
use super::document::project_type_aware;
use super::{
    LintResult, Linter, RULE_NO_FLOATING_PROMISES, RULE_NO_REACTIVITY_LOSS,
    RULE_NO_UNSAFE_TEMPLATE_BINDING, RULE_STRICT_BOOLEAN, strict_boolean, typescript_eslint,
    with_corsa_session,
};
use super::{
    markers::push_promise_marker,
    parsing::collect_floating_candidates,
    reactivity_loss::collect_reactivity_loss_queries,
    rule_queries::{collect_emit_queries, collect_prop_queries},
    template_queries::collect_template_query_sets,
};
use planning::{
    collect_emit_static_warning_or_probe_need, collect_prop_static_warning_or_probe_need,
    is_type_rule_active,
};
pub(in crate::linter) use prepared::PreparedSfc;
use prepared::SfcQueryPlan;
use vize_croquis::script_parser;
use vize_l0::{ToCompactString, profile};
/// Single-file compatibility path. Batch callers publish all prepared documents
/// before refreshing the checker, then call `evaluate` without reopening them.
pub(super) fn lint_with_descriptor<'a>(
    linter: &Linter,
    source: &str,
    filename: &str,
    descriptor: &vize_atelier_sfc::SfcDescriptor<'a>,
    derived: bool,
) -> LintResult {
    let mut prepared: super::PreparedSfc =
        super::prepare_with_descriptor(linter, source, filename, descriptor, derived);
    let failure = if prepared.virtual_content().is_some() {
        profile!(
            "patina.type_aware.corsa_session",
            with_corsa_session(linter, filename, |session| {
                if let Some(content) = prepared.virtual_content() {
                    profile!(
                        "patina.type_aware.corsa.open_virtual_project",
                        session.open_virtual_project(content, filename)
                    )?;
                }
                prepared.evaluate(linter, session)
            })
        )
        .err()
    } else {
        None
    };
    prepared.finish(linter, failure)
}

pub(in crate::linter) fn prepare_with_descriptor<'a>(
    linter: &Linter,
    source: &str,
    filename: &str,
    descriptor: &vize_atelier_sfc::SfcDescriptor<'a>,
    derived: bool,
) -> PreparedSfc {
    let allocator =
        vize_l0::Allocator::with_capacity((source.len() * 4).max(linter.initial_capacity));
    let script_allocator = oxc_allocator::Allocator::default();
    let scripts = crate::linter::sfc_scripts::lint_scripts(&script_allocator, descriptor);
    let template_ast = descriptor.template.as_ref().map(|template| {
        let l2 = crate::markup::L2Template::lower(&allocator, &template.content);
        let has_fatal_parse_errors = linter.has_fatal_template_parse_errors(&l2);
        (l2, template.loc.start as u32, has_fatal_parse_errors)
    });
    let template_has_fatal_parse_errors = template_ast
        .as_ref()
        .is_some_and(|(_, _, has_fatal)| *has_fatal);
    // Materialize CSS while the template is alive, but retain its position after
    // all type findings when the owned plan is evaluated later.
    let finish = |result, plan| {
        PreparedSfc::new(
            result,
            plan,
            css::finish(
                linter,
                descriptor,
                LintResult {
                    filename: filename.to_compact_string(),
                    diagnostics: Vec::new(),
                    error_count: 0,
                    warning_count: 0,
                },
                template_ast
                    .as_ref()
                    .and_then(|(l2, _, fatal)| (!*fatal).then_some(l2.surface())),
            ),
        )
    };
    let analysis = profile!("patina.type_aware.croquis", {
        super::super::engine::analyze_descriptor_for_lint_with_template(
            &scripts,
            template_ast
                .as_ref()
                .and_then(|(l2, _, has_fatal)| (!*has_fatal).then_some(l2)),
            linter.has_resolved_macro_type_demand().then_some(filename),
            linter.has_unused_bindings_demand(),
            derived,
        )
    });
    let mut result = if let (Some((l2, _, has_fatal)), Some(template)) =
        (template_ast.as_ref(), descriptor.template.as_ref())
    {
        let (lint_result, scripts) = profile!(
            "patina.type_aware.template_rules",
            linter.lint_sfc_template_root(
                SfcTemplateLintInput {
                    filename,
                    template,
                    allocator: &allocator,
                    l2,
                    descriptor,
                    analysis: if *has_fatal {
                        TemplateAnalysis::Disabled
                    } else {
                        TemplateAnalysis::Precomputed(&analysis)
                    },
                },
                &scripts,
            )
        );
        let mut parse_result =
            Linter::l1_template_parse_lint_result(filename, &template.content, l2);
        linter.append_compatibility_template_parse_diagnostics(
            &mut parse_result,
            &template.content,
            l2,
        );
        super::super::engine::offset_result(&mut parse_result, template.loc.start as u32);
        linter.append_style_parse_diagnostics(&mut parse_result, &allocator, &descriptor.styles);
        let mut result = Linter::merge_lint_results(parse_result, lint_result);
        result.error_count += scripts.error_count;
        result.warning_count += scripts.warning_count;
        result.diagnostics.extend(scripts.diagnostics);
        result
    } else {
        let mut result = LintResult {
            filename: filename.into(),
            diagnostics: Vec::new(),
            error_count: 0,
            warning_count: 0,
        };
        super::super::script_rules::append_builtin_script_diagnostics(
            linter,
            descriptor,
            &scripts,
            &mut result,
            None,
        );
        result
    };
    let script_block = descriptor
        .script_setup
        .as_ref()
        .or(descriptor.script.as_ref());
    let include_boolean_queries = is_type_rule_active(linter, RULE_STRICT_BOOLEAN);
    let include_typescript = typescript_eslint::is_active(linter);
    let script_content = script_block.map_or("", |block| block.content.as_ref());
    let script_offset = script_block.map_or(0, |block| block.loc.start as u32);
    if script_content.is_empty() && !include_boolean_queries {
        return finish(result, None);
    }

    let needs_prop_probe = profile!("patina.type_aware.plan_prop_queries", {
        collect_prop_static_warning_or_probe_need(linter, &analysis, &mut result, descriptor)
    });
    let needs_emit_probe = profile!("patina.type_aware.plan_emit_queries", {
        collect_emit_static_warning_or_probe_need(linter, &analysis, &mut result, descriptor)
    });
    let include_template_queries = !template_has_fatal_parse_errors
        && is_type_rule_active(linter, RULE_NO_UNSAFE_TEMPLATE_BINDING);
    let include_template_promise_queries =
        !template_has_fatal_parse_errors && is_type_rule_active(linter, RULE_NO_FLOATING_PROMISES);
    let include_reactivity_queries = is_type_rule_active(linter, RULE_NO_REACTIVITY_LOSS);
    if !needs_prop_probe
        && !needs_emit_probe
        && !include_template_queries
        && !include_template_promise_queries
        && !include_reactivity_queries
        && !include_boolean_queries
        && !include_typescript
    {
        return finish(result, None);
    }

    let template_offset = template_ast
        .as_ref()
        .map(|(_, offset, _)| *offset)
        .unwrap_or(0);
    let parse_result = profile!("patina.type_aware.script_parse", {
        if let Some(script_setup) = descriptor.script_setup.as_ref() {
            let generic = script_setup
                .attrs
                .get("generic")
                .map(|value| value.as_ref());
            let lang = script_setup.lang.as_deref().map(str::trim);
            let jsx = matches!(lang, Some("tsx" | "jsx"));
            script_parser::parse_script_setup_with_generic_and_jsx(script_content, generic, jsx)
        } else {
            super::script_options::parse_plain_script_for_type_aware(script_content)
        }
    });
    let mut virtual_ts = profile!(
        "patina.type_aware.virtual_ts",
        project_type_aware(
            descriptor,
            script_content,
            template_ast
                .as_ref()
                .and_then(|(l2, _, has_fatal)| { (!*has_fatal).then_some(l2.lint_template()) }),
            script_offset,
            template_offset,
            filename,
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
                descriptor,
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
                descriptor,
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
            script_offset,
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
                    |(l2, _, has_fatal)| {
                        if *has_fatal {
                            (Vec::new(), Vec::new())
                        } else {
                            collect_template_query_sets(
                                &virtual_ts,
                                l2.surface(),
                                template_offset,
                                include_template_queries,
                                include_template_promise_queries,
                            )
                        }
                    },
                )
            } else {
                (Vec::new(), Vec::new())
            }
        });

    let boolean_plan = include_boolean_queries.then(|| {
        strict_boolean::plan(
            descriptor,
            template_ast
                .as_ref()
                .and_then(|(l2, offset, fatal)| (!*fatal).then_some((l2.surface(), *offset))),
            &mut virtual_ts,
        )
    });

    let typescript_plan = include_typescript
        .then(|| typescript_eslint::Plan::from_descriptor(descriptor, virtual_ts.mapping.clone()));
    if macro_queries.is_empty()
        && template_queries.is_empty()
        && template_promise_queries.is_empty()
        && reactivity_loss_queries.is_empty()
        && boolean_plan
            .as_ref()
            .is_none_or(strict_boolean::Plan::is_empty)
        && typescript_plan
            .as_ref()
            .is_none_or(typescript_eslint::Plan::is_empty)
    {
        return finish(result, None);
    }

    finish(
        result,
        Some(SfcQueryPlan {
            content: virtual_ts.content,
            source: source.to_compact_string(),
            script_offset,
            macros: macro_queries,
            templates: template_queries,
            promises: template_promise_queries,
            reactivities: reactivity_loss_queries,
            boolean: boolean_plan,
            typescript: typescript_plan,
        }),
    )
}
