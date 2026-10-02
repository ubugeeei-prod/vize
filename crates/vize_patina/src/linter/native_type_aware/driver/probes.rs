use super::super::{
    CorsaTypeAwareSession, LintResult, RULE_NO_FLOATING_PROMISES, has_promise_like_return,
    markers::{MacroQuery, QueryKind},
    push_warning,
    reactivity_loss::ReactivityLossQuery,
    should_warn_for_emit_validator, should_warn_for_prop_access, should_warn_for_reactivity_loss,
    template_queries::{TemplatePromiseQuery, TemplateQuery, TemplateQueryKind},
};
use crate::diagnostic::LintDiagnostic;
use vize_l0::{FxHashSet, String, profile};

pub(super) struct ProbeBatch<'a> {
    pub content: &'a str,
    pub source: &'a str,
    pub script_offset: u32,
    pub macros: &'a [MacroQuery],
    pub templates: &'a [TemplateQuery],
    pub promises: &'a [TemplatePromiseQuery],
    pub reactivities: &'a [ReactivityLossQuery],
}

pub(super) fn evaluate(
    session: &mut CorsaTypeAwareSession,
    batch: ProbeBatch<'_>,
    result: &mut LintResult,
    should_warn_for_props: &mut bool,
    should_warn_for_emits: &mut bool,
) -> Result<(), String> {
    let mut warned_template_owners = FxHashSet::default();
    let mut warned_reactivity_loss_owners = FxHashSet::default();
    for query in batch.macros {
        let probe = profile!(
            "patina.type_aware.corsa.probe_macro",
            session.probe_type_at_offset(
                batch.content,
                query.generated_offset,
                false,
                matches!(query.kind, QueryKind::EmitValidator | QueryKind::Promise),
            )
        )?;

        match query.kind {
            QueryKind::PropType => {
                *should_warn_for_props |= should_warn_for_prop_access(probe.as_ref());
            }
            QueryKind::EmitValidator => {
                *should_warn_for_emits |= should_warn_for_emit_validator(probe.as_ref());
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
                                result,
                                LintDiagnostic::warn(
                                    RULE_NO_FLOATING_PROMISES,
                                    "Floating Promise must be awaited, returned, or explicitly ignored with `void`",
                                    batch.script_offset + query.source_start,
                                    batch.script_offset + query.source_end,
                                )
                                .with_help(
                                    "Add `await`, return the Promise, or prefix it with `void` when the fire-and-forget behavior is intentional.",
                                ),
                            );
                }
            }
        }
    }

    for query in batch.templates {
        let probe = profile!(
            "patina.type_aware.corsa.probe_template",
            session.probe_type_at_offset(
                batch.content,
                query.generated_offset,
                false,
                matches!(query.kind, TemplateQueryKind::CallReturn),
            )
        )?;
        if !super::super::expression_bindings::template_binding_is_unsafe(
            batch.source,
            query,
            probe.as_ref(),
        ) {
            continue;
        }

        let owner_key = query.owner_key();
        if matches!(
            query.kind,
            TemplateQueryKind::Expression | TemplateQueryKind::CallReturn
        ) && warned_template_owners.contains(&owner_key)
        {
            continue;
        }

        push_warning(result, query.diagnostic());
        if matches!(query.kind, TemplateQueryKind::CallCallee) {
            warned_template_owners.insert(owner_key);
        }
    }

    for query in batch.promises {
        let probe = profile!(
            "patina.type_aware.corsa.probe_template_promise",
            session.probe_type_at_offset(batch.content, query.generated_offset, false, true,)
        )?;
        let Some(probe) = probe.as_ref() else {
            continue;
        };
        if has_promise_like_return(probe)
            || corsa::utils::is_promise_like_type_texts(&probe.type_texts, &probe.property_names)
        {
            push_warning(result, query.diagnostic());
        }
    }

    for query in batch.reactivities {
        let probe = profile!(
            "patina.type_aware.corsa.probe_reactivity_loss",
            session.probe_type_at_offset(batch.content, query.generated_offset, false, false,)
        )?;
        if !should_warn_for_reactivity_loss(probe.as_ref()) {
            continue;
        }

        let owner_key = query.owner_key();
        if warned_reactivity_loss_owners.insert(owner_key) {
            push_warning(result, query.diagnostic(batch.script_offset));
        }
    }
    Ok(())
}
