//! vue/no-setup-props-reactivity-loss
//!
//! Destructuring `setup`'s props parameter, or reading a prop in that function's
//! root, copies the current value and drops later updates. A read inside another
//! function stays reactive. `<script setup>` treats `defineProps()` the same way:
//! the destructure itself is kept, and a later root copy is reported.
//!
//! ## Examples
//!
//! ### Invalid
//! ```js
//! export default {
//!   setup({ count }) {},
//!   setup(props) { const { count } = props },
//! }
//! const { count } = defineProps()
//! const copy = count
//! ```
//!
//! ### Valid
//! ```js
//! export default {
//!   setup(props) { watch(() => props.count) },
//! }
//! const { count } = defineProps()
//! const double = computed(() => count)
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::Severity;
use oxc_ast::ast::Program;
use oxc_ast_visit::Visit;

mod visitor;

use visitor::Visitor;

pub(super) const RULE: &str = "vue/no-setup-props-reactivity-loss";
pub(super) const DESTRUCTURE: &str =
    "Destructuring the `props` will cause the value to lose reactivity.";
pub(super) const GET_SETUP: &str = "Getting a value from the `props` in root scope of `setup()` will cause the value to lose reactivity.";
pub(super) const GET_SCRIPT_SETUP: &str = "Getting a value from the `props` in root scope of `<script setup>` will cause the value to lose reactivity.";

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: RULE,
    description: "Disallow setup prop reads that lose reactivity",
    default_severity: Severity::Warning,
};

pub struct NoSetupPropsReactivityLoss;

impl ScriptRule for NoSetupPropsReactivityLoss {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    fn uses_ast(&self) -> bool {
        true
    }

    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        let mut visitor = Visitor {
            offset,
            result,
            fn_depth: 0,
            scope_id: 0,
            force_vue: false,
            frames: Vec::new(),
            objects: Vec::new(),
        };
        visitor.visit_program(program);
    }
}

#[cfg(test)]
mod tests;
