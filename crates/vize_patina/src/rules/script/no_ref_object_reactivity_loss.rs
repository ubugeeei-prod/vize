//! vue/no-ref-object-reactivity-loss
//!
//! Reading `.value` from a ref, or destructuring that property, copies the
//! current value and drops later updates. A read inside another function keeps
//! the ref reactive. Writing `.value` updates the ref and is left alone.
//!
//! ## Examples
//!
//! ### Invalid
//! ```js
//! import { ref } from 'vue'
//! const count = ref(0)
//! const value = count.value
//! const { value: copy } = count
//! ```
//!
//! ### Valid
//! ```js
//! import { ref } from 'vue'
//! const count = ref(0)
//! const value = computed(() => count.value)
//! fn(count)
//! count.value = 1
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::Severity;
use oxc_ast::ast::Program;
use oxc_ast_visit::Visit;

mod visitor;

use visitor::Visitor;

pub(super) const RULE: &str = "vue/no-ref-object-reactivity-loss";
pub(super) const MESSAGE: &str = "Getting a value from the ref object in the same scope will cause the value to lose reactivity.";
pub(super) const REF_FACTORIES: &[&str] = &["ref", "computed", "shallowRef", "toRef", "customRef"];

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: RULE,
    description: "Disallow copying a ref value in the same scope",
    default_severity: Severity::Warning,
};

pub struct NoRefObjectReactivityLoss;

impl ScriptRule for NoRefObjectReactivityLoss {
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
            writing: false,
            frames: Vec::new(),
        };
        visitor.visit_program(program);
    }
}

#[cfg(test)]
mod tests;
