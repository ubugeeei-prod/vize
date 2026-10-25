//! Prepare once, publish once per project, then evaluate owned checker queries.

mod projects;
mod sfc;
#[cfg(test)]
mod tests;

use super::super::{
    config::{LintResult, Linter},
    corsa_session::CorsaTypeAwareSession,
    native_type_aware::{self, PreparedSfc, TypeAwareFailure, plain_module::PreparedPlainModule},
};
use vize_l0::{String, ToCompactString};

/// Owned lint work; parsing arenas are released before checker publication.
pub struct PreparedLint {
    filename: String,
    work: Work,
}

enum Work {
    Complete(LintResult),
    Sfc(Box<SfcWork>),
    Plain {
        plan: PreparedPlainModule,
        source: Option<String>,
    },
}

// Keep the existing single SFC allocation while making every prepared-file
// slot small. The plan and its finish data share the same owned lifetime.
struct SfcWork {
    plan: PreparedSfc,
    tail: sfc::SfcTail,
}

impl PreparedLint {
    /// Wrap a completed result that needs no checker work.
    pub fn complete(result: LintResult) -> Self {
        Self {
            filename: result.filename.clone(),
            work: Work::Complete(result),
        }
    }

    fn virtual_content(&self) -> Option<&str> {
        match &self.work {
            Work::Complete(_) => None,
            Work::Sfc(work) => work.plan.virtual_content(),
            Work::Plain { plan, .. } => plan.virtual_content(),
        }
    }

    fn evaluate(
        &mut self,
        linter: &Linter,
        session: &mut CorsaTypeAwareSession,
    ) -> Result<(), String> {
        session.select_virtual_document(&self.filename)?;
        match &mut self.work {
            Work::Complete(_) => Ok(()),
            Work::Sfc(work) => work.plan.evaluate(linter, session),
            Work::Plain { plan, .. } => plan.evaluate(linter, session),
        }
    }

    fn finish(self, linter: &Linter, failure: Option<TypeAwareFailure>) -> LintResult {
        match self.work {
            Work::Complete(result) => result,
            Work::Sfc(work) => {
                let SfcWork { plan, tail } = *work;
                tail.finish(linter, plan.finish(linter, failure))
            }
            Work::Plain { plan, source } => {
                let result = plan.finish(linter, failure);
                if let Some(source) = source {
                    linter.finish_script_result(&source, result)
                } else {
                    linter.apply_diagnostic_policy(result)
                }
            }
        }
    }
}

impl Linter {
    /// Whether this configuration enables at least one native type-aware rule.
    pub fn has_type_aware_rules(&self) -> bool {
        native_type_aware::has_active_type_aware_rules(self)
    }

    /// Prepare plain script rules without starting or refreshing a checker.
    pub fn prepare_script(&self, source: &str, filename: &str) -> PreparedLint {
        if !native_type_aware::has_active_type_aware_rules(self) {
            return PreparedLint::complete(self.lint_script(source, filename));
        }
        let mut result = super::sfc::empty_lint_result(filename);
        super::super::script_rules::append_builtin_script_rules_for_source(
            self,
            source,
            0,
            &mut result,
        );
        PreparedLint {
            filename: filename.to_compact_string(),
            work: Work::Plain {
                plan: native_type_aware::plain_module::prepare_plain_module(
                    self, source, filename, result,
                ),
                source: Some(source.to_compact_string()),
            },
        }
    }

    /// Prepare JSX markup and script rules without checker I/O.
    pub fn prepare_jsx(
        &self,
        source: &str,
        filename: &str,
        lang: vize_atelier_jsx::JsxLang,
    ) -> PreparedLint {
        if !native_type_aware::has_active_type_aware_rules(self) {
            return PreparedLint::complete(self.lint_jsx(source, filename, lang));
        }
        let result = self.lint_jsx_without_types(source, filename, lang);
        PreparedLint {
            filename: filename.to_compact_string(),
            work: Work::Plain {
                plan: native_type_aware::plain_module::prepare_plain_module(
                    self, source, filename, result,
                ),
                source: None,
            },
        }
    }

    /// Publish all prepared documents once per effective project/runtime.
    /// Results retain the input order, including files with no type queries.
    pub fn lint_prepared_batch(inputs: Vec<(&Self, PreparedLint)>) -> Vec<LintResult> {
        projects::finish(inputs)
    }
}
