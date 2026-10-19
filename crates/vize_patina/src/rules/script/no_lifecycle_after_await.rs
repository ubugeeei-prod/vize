//! script/no-lifecycle-after-await
//!
//! Disallow lifecycle hooks registered after an `await` in `setup()`.
//!
//! Vue registers `onMounted` and the other lifecycle hooks on the component
//! instance that is current while `setup()` runs. After an `await` there is no
//! current instance, so the hook is never registered. Mirrors
//! eslint-plugin-vue's `vue/no-lifecycle-after-await`: it checks the `setup()`
//! option of a component, not `<script setup>` (whose compiler restores the
//! instance after `await`). A hook given an explicit target instance as its
//! second argument is allowed.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! import { onMounted } from 'vue'
//! export default {
//!   async setup() {
//!     await loadData()
//!     onMounted(() => {})
//!   },
//! }
//! ```
//!
//! ### Valid
//! ```ts
//! import { onMounted } from 'vue'
//! export default {
//!   async setup() {
//!     onMounted(() => {})
//!     await loadData()
//!   },
//! }
//! ```

use super::{
    ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext,
    after_await::{self, Target},
};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::Program;
use vize_l0::CompactString;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-lifecycle-after-await",
    description: "Disallow lifecycle hooks registered after an await in setup()",
    default_severity: Severity::Error,
};

/// Disallow lifecycle hooks registered after an await in setup().
pub struct NoLifecycleAfterAwait;

impl ScriptRule for NoLifecycleAfterAwait {
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
        for finding in after_await::find(program, sfc, Target::Lifecycle) {
            let start = offset as u32 + finding.span.start;
            let end = offset as u32 + finding.span.end;
            result.add_diagnostic(
                LintDiagnostic::error(META.name, CompactString::new("Lifecycle hooks are forbidden after an `await` expression."), start, end)
                    .with_label("registered after `await`", start, end)
                    .with_help("After an `await`, `setup()` has no current component instance, so this hook is never registered. Register it before the first `await`."),
            );
        }
    }
}

#[cfg(test)]
mod tests;
