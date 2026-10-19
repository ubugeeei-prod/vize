//! script/no-watch-after-await
//!
//! Disallow `watch` and `watchEffect` after an `await` in `setup()`.
//!
//! A watcher created while `setup()` runs is stopped with its component. After
//! an `await` there is no current instance, so the watcher is never stopped
//! automatically and leaks. Mirrors eslint-plugin-vue's
//! `vue/no-watch-after-await`: it checks the `setup()` option of a component,
//! and allows a watcher whose stop handle is kept (assigned, passed on, or
//! stored), since the code can stop it itself.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! import { watch } from 'vue'
//! export default {
//!   async setup(props) {
//!     await loadData()
//!     watch(() => props.id, reload)
//!   },
//! }
//! ```
//!
//! ### Valid
//! ```ts
//! import { watch } from 'vue'
//! export default {
//!   async setup(props) {
//!     watch(() => props.id, reload)
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
    name: "script/no-watch-after-await",
    description: "Disallow watchers registered after an await in setup() without keeping their stop handle",
    default_severity: Severity::Error,
};

/// Disallow watchers registered after an await in setup() without keeping their stop handle.
pub struct NoWatchAfterAwait;

impl ScriptRule for NoWatchAfterAwait {
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
        for finding in after_await::find(program, sfc, Target::Watch) {
            let start = offset as u32 + finding.span.start;
            let end = offset as u32 + finding.span.end;
            result.add_diagnostic(
                LintDiagnostic::error(META.name, CompactString::new("`watch` is forbidden after an `await` expression."), start, end)
                    .with_label("registered after `await`", start, end)
                    .with_help("After an `await`, `setup()` has no current component instance, so this watcher is never stopped with the component. Create it before the first `await`, or keep its stop handle and stop it yourself."),
            );
        }
    }
}

#[cfg(test)]
mod tests;
