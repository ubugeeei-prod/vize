//! script/no-ref-as-operand
//!
//! Require a ref-bound variable to be accessed through `.value` when it is used
//! as an *operand*. A composable like `ref()`, `computed()`, `shallowRef()`,
//! `toRef()`, or `customRef()` returns a ref object whose underlying value lives
//! behind `.value`; using the binding itself in an arithmetic, logical, unary,
//! update, or comparison position (or as a condition) operates on the ref object
//! rather than its value, which is almost always a bug.
//!
//! ```js
//! let count = ref(0)
//! count++                  // BAD: operates on the ref object, not the number
//! console.log(count + 1)   // BAD
//! if (count) {}            // BAD: a ref object is always truthy
//!
//! count.value++            // GOOD
//! console.log(count.value + 1)  // GOOD
//! watch(count, () => {})   // GOOD: passing the ref itself is not an operand
//! ```
//!
//! Port of [`vue/no-ref-as-operand`](https://eslint.vuejs.org/rules/no-ref-as-operand.html),
//! imports are resolved through their lexical binding, including aliases and
//! namespace members from Vue composition modules. Unrelated functions are not
//! inferred to return refs from their names, and an operand is reported only
//! when it resolves (through lexical scoping) to such a binding. A same-named
//! non-ref binding in an inner scope shadows the ref and is left alone.

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{Expression, Program};
use oxc_ast_visit::Visit;
use oxc_span::Span;
use vize_l0::CompactString;

mod scope;

use scope::{Binding, Frame};

mod visitor;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-ref-as-operand",
    description: "Require ref-bound variables to be accessed via `.value` when used as an operand",
    default_severity: Severity::Error,
};

/// Require ref-bound variables to be unwrapped via `.value` when used as an operand.
pub struct NoRefAsOperand;

impl ScriptRule for NoRefAsOperand {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    #[inline]
    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        let mut visitor = RefOperandVisitor {
            offset,
            result,
            scopes: Vec::new(),
        };
        visitor.visit_program(program);
    }
}

/// Tracks, per lexical scope, which names are bound to a ref factory result.
///
/// A scope frame is pushed on entry to the program, every function/arrow body,
/// block/catch body, and every `for` header; bindings introduced by variable
/// declarations or parameters in that
/// frame map a name to whether its initializer is a ref factory call. Operand
/// identifiers resolve against the stack innermost-first, so a same-named
/// non-ref binding closer to the use shadows an outer ref (no false positive).
struct RefOperandVisitor<'rule> {
    offset: usize,
    result: &'rule mut ScriptLintResult,
    scopes: Vec<Frame>,
}

impl RefOperandVisitor<'_> {
    /// Report a direct identifier operand if it resolves to a ref binding. Used
    /// for both operator operands and condition positions (`if`, `while`,
    /// ternary test, ...), where a ref object is always truthy.
    fn check_operand(&mut self, expression: &Expression<'_>) {
        if let Expression::Identifier(id) = expression
            && self.is_ref(&id.name)
        {
            self.report(id.span, &id.name);
        }
    }

    /// Resolve `name` against the scope stack, innermost-first. Returns `true`
    /// only when the nearest binding of that name is a ref factory result.
    fn is_ref(&self, name: &str) -> bool {
        for frame in self.scopes.iter().rev() {
            if let Some(&binding) = frame.get(name) {
                return binding == Binding::Ref;
            }
        }
        false
    }

    fn report(&mut self, span: Span, name: &str) {
        let start = self.offset as u32 + span.start;
        let end = self.offset as u32 + span.end;
        let mut message = CompactString::with_capacity(name.len() + 48);
        message.push('\'');
        message.push_str(name);
        message.push_str("' is a ref and must be unwrapped with `.value` here.");
        let diagnostic = LintDiagnostic::error(META.name, message, start, end)
            .with_label("ref used directly as an operand", start, end)
            .with_help(
                "A ref holds its value behind `.value`. Read it as `<name>.value` in this \
                 position; passing the ref itself (e.g. to `watch`) does not need `.value`.",
            );
        self.result.add_diagnostic(diagnostic);
    }
}

#[cfg(test)]
mod tests;
