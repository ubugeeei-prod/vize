//! Owned checker work, independent of the source descriptor and template arena.

use super::super::{
    CorsaTypeAwareSession, LintResult, Linter, RULE_REQUIRE_TYPED_EMITS, RULE_REQUIRE_TYPED_PROPS,
    TypeAwareFailure,
    markers::{MacroQuery, QueryKind},
    reactivity_loss::ReactivityLossQuery,
    rule_queries::{MacroWarning, push_macro_warning},
    strict_boolean,
    template_queries::{TemplatePromiseQuery, TemplateQuery},
};
use super::probes::{self, ProbeBatch};
use vize_l0::String;

pub(super) struct SfcQueryPlan {
    pub content: String,
    pub source: String,
    pub script_offset: u32,
    pub macros: Vec<MacroQuery>,
    pub templates: Vec<TemplateQuery>,
    pub promises: Vec<TemplatePromiseQuery>,
    pub reactivities: Vec<ReactivityLossQuery>,
    pub boolean: Option<strict_boolean::Plan>,
}

/// A complete preparation retains no descriptor, AST or allocator references.
pub(in crate::linter) struct PreparedSfc {
    result: LintResult,
    plan: Option<SfcQueryPlan>,
    css: LintResult,
    should_warn_for_props: bool,
    should_warn_for_emits: bool,
}

impl PreparedSfc {
    pub(super) fn new(result: LintResult, plan: Option<SfcQueryPlan>, css: LintResult) -> Self {
        Self {
            result,
            plan,
            css,
            should_warn_for_props: false,
            should_warn_for_emits: false,
        }
    }

    pub(in crate::linter) fn virtual_content(&self) -> Option<&str> {
        self.plan.as_ref().map(|plan| plan.content.as_str())
    }

    /// The caller selects this document in an already published checker snapshot.
    /// This method neither opens documents nor refreshes the project.
    pub(in crate::linter) fn evaluate(
        &mut self,
        linter: &Linter,
        session: &mut CorsaTypeAwareSession,
    ) -> Result<(), String> {
        let Some(plan) = self.plan.as_ref() else {
            return Ok(());
        };
        probes::evaluate(
            session,
            ProbeBatch {
                content: &plan.content,
                source: &plan.source,
                script_offset: plan.script_offset,
                macros: &plan.macros,
                templates: &plan.templates,
                promises: &plan.promises,
                reactivities: &plan.reactivities,
            },
            &mut self.result,
            &mut self.should_warn_for_props,
            &mut self.should_warn_for_emits,
        )?;
        if let Some(boolean) = &plan.boolean {
            strict_boolean::evaluate(
                session,
                &plan.content,
                boolean,
                linter.strict_boolean_options,
                &mut self.result,
            )?;
        }
        Ok(())
    }

    /// Retain partial probe findings on failure, followed by macro and CSS findings.
    /// The SFC engine owns severity and document-level postprocessing.
    pub(in crate::linter) fn finish(
        mut self,
        _linter: &Linter,
        failure: Option<TypeAwareFailure>,
    ) -> LintResult {
        if let Some(plan) = self.plan {
            if let Some(failure) = failure {
                super::super::push_type_aware_failure(&mut self.result, failure);
            }
            push_macro_warning(
                &mut self.result,
                &plan.macros,
                MacroWarning {
                    kind: QueryKind::PropType,
                    base_offset: plan.script_offset,
                    rule_name: RULE_REQUIRE_TYPED_PROPS,
                    message: "Prop should have a type definition",
                    help: "Use `defineProps<Props>()` or a runtime prop object with concrete constructor types.",
                    should_warn: self.should_warn_for_props,
                },
            );
            push_macro_warning(
                &mut self.result,
                &plan.macros,
                MacroWarning {
                    kind: QueryKind::EmitValidator,
                    base_offset: plan.script_offset,
                    rule_name: RULE_REQUIRE_TYPED_EMITS,
                    message: "Emit should have a type definition",
                    help: "Use `defineEmits<...>()` or a validator object with typed payload parameters.",
                    should_warn: self.should_warn_for_emits,
                },
            );
        }
        self.result.error_count += self.css.error_count;
        self.result.warning_count += self.css.warning_count;
        self.result.diagnostics.extend(self.css.diagnostics);
        self.result
    }
}
