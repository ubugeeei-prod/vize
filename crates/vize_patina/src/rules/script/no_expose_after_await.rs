//! script/no-expose-after-await
//!
//! Disallow `expose()` and `defineExpose()` after an `await`.
//!
//! `expose` publishes the public instance of the component that is current
//! while setup runs. After an `await` there is no current instance, so the
//! component exposes nothing. Mirrors eslint-plugin-vue's
//! `vue/no-expose-after-await`: it checks `expose` from the `setup()` context
//! (`context.expose()` or a destructured `expose`) and a top-level
//! `defineExpose()` in `<script setup>`.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! await loadData()
//! defineExpose({ reload })
//! ```
//!
//! ### Valid
//! ```ts
//! defineExpose({ reload })
//! await loadData()
//! ```

use super::{
    ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext,
    after_await::{self, Target},
};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::Program;
use vize_l0::cstr;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-expose-after-await",
    description: "Disallow expose() and defineExpose() after an await",
    default_severity: Severity::Error,
};

/// Disallow expose() and defineExpose() after an await.
pub struct NoExposeAfterAwait;

impl ScriptRule for NoExposeAfterAwait {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        self.check_program_with_sfc(program, source, offset, SfcScriptContext::default(), result);
    }

    fn check_program_with_sfc<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        sfc: SfcScriptContext<'_>,
        result: &mut ScriptLintResult,
    ) {
        for finding in after_await::find(program, sfc, Target::Expose) {
            let start = offset as u32 + finding.span.start;
            let end = offset as u32 + finding.span.end;
            result.add_diagnostic(
                LintDiagnostic::error(META.name, cstr!("`{}` is forbidden after an `await` expression.", finding.name), start, end)
                    .with_label("called after `await`", start, end)
                    .with_help("After an `await`, there is no current component instance, so nothing is exposed. Call it before the first `await`."),
            );
        }
    }
}

#[cfg(test)]
mod tests;
