use super::{
    has_promise_like_return, has_unsafe_template_type, push_warning,
    should_warn_for_emit_validator, should_warn_for_prop_access, with_corsa_session, LintResult,
    Linter, RULE_NO_FLOATING_PROMISES, RULE_NO_UNSAFE_TEMPLATE_BINDING, RULE_REQUIRE_TYPED_EMITS,
    RULE_REQUIRE_TYPED_PROPS,
};
use crate::diagnostic::LintDiagnostic;
use std::path::Path;
use vize_armature::Parser as TemplateParser;
use vize_carton::{profile, FxHashSet};
use vize_croquis::{
    script_parser,
    virtual_ts::{generate_virtual_ts_with_croquis, VirtualTsConfig},
    Analyzer, AnalyzerOptions,
};

use super::{
    markers::{push_promise_marker, QueryKind},
    parsing::collect_floating_candidates,
    rule_queries::{collect_emit_queries, collect_prop_queries, push_macro_warning, MacroWarning},
    template_queries::{collect_template_queries, TemplateQueryKind},
};

pub(super) fn lint_with_descriptor<'a>(
    linter: &Linter,
    source: &str,
    filename: &str,
    descriptor: vize_atelier_sfc::SfcDescriptor<'a>,
) -> LintResult {
    let mut result = profile!(
        "patina.type_aware.script_rules",
        super::super::script_rules::lint_with_descriptor(linter, filename, &descriptor)
    );

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

    let allocator =
        vize_carton::Allocator::with_capacity((source.len() * 4).max(linter.initial_capacity));
    let template_ast = descriptor.template.as_ref().map(|template| {
        let parser = TemplateParser::new(allocator.as_bump(), &template.content);
        let (root, _) = profile!("patina.type_aware.template_parse", parser.parse());
        (root, template.loc.start as u32)
    });

    let analysis = profile!("patina.type_aware.croquis", {
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        if let Some(script_setup) = descriptor.script_setup.as_ref() {
            let generic = script_setup
                .attrs
                .get("generic")
                .map(|value| value.as_ref());
            analyzer.analyze_script_setup_with_generic(script_setup.content.as_ref(), generic);
        } else if let Some(script) = descriptor.script.as_ref() {
            analyzer.analyze_script_plain(script.content.as_ref());
        }
        if let Some((template_ast, _)) = template_ast.as_ref() {
            analyzer.analyze_template(template_ast);
        }
        analyzer.finish()
    });

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
    );
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

    if linter.registry.has_rule(RULE_NO_FLOATING_PROMISES)
        && linter.is_rule_enabled(RULE_NO_FLOATING_PROMISES)
    {
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

    let template_queries = profile!("patina.type_aware.collect_template_queries", {
        if linter.registry.has_rule(RULE_NO_UNSAFE_TEMPLATE_BINDING)
            && linter.is_rule_enabled(RULE_NO_UNSAFE_TEMPLATE_BINDING)
        {
            template_ast.as_ref().map_or_else(Vec::new, |(root, _)| {
                collect_template_queries(&virtual_ts, root, template_offset)
            })
        } else {
            Vec::new()
        }
    });

    if macro_queries.is_empty() && template_queries.is_empty() {
        return result;
    }

    let mut should_warn_for_props = false;
    let mut should_warn_for_emits = false;
    let mut warned_template_owners = FxHashSet::default();
    let _ = profile!(
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
                        if let Some(probe) = probe.as_ref() {
                            if has_promise_like_return(probe)
                                || corsa::utils::is_promise_like_type_texts(
                                    &probe.type_texts,
                                    &probe.property_names,
                                )
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
            Ok(())
        })
    );

    push_macro_warning(
        &mut result,
        &macro_queries,
        MacroWarning {
            kind: QueryKind::PropType,
            base_offset: script_block.loc.start as u32,
            rule_name: RULE_REQUIRE_TYPED_PROPS,
            message: "Prop should have a type definition",
            help:
                "Use `defineProps<Props>()` or a runtime prop object with concrete constructor types.",
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
