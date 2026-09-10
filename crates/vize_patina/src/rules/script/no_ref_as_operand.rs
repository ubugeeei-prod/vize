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
//! scoped conservatively: a binding is tracked only when its initializer is a
//! direct call to one of the known ref factories, and an operand is reported only
//! when it resolves (through lexical scoping) to such a binding. A same-named
//! non-ref binding in an inner scope shadows the ref and is left alone.

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    AssignmentExpression, BinaryExpression, BlockStatement, CatchClause, ConditionalExpression,
    DoWhileStatement, Expression, ForInStatement, ForOfStatement, ForStatement, ForStatementLeft,
    Function, IfStatement, LogicalExpression, Program, TemplateLiteral, UnaryExpression,
    UpdateExpression, WhileStatement,
};
use oxc_ast_visit::{
    Visit,
    walk::{
        walk_assignment_expression, walk_binary_expression, walk_block_statement,
        walk_catch_clause, walk_conditional_expression, walk_do_while_statement,
        walk_for_in_statement, walk_for_of_statement, walk_for_statement, walk_function,
        walk_if_statement, walk_logical_expression, walk_program, walk_template_literal,
        walk_unary_expression, walk_update_expression, walk_while_statement,
    },
};
use oxc_span::Span;
use oxc_syntax::operator::UnaryOperator;
use oxc_syntax::scope::ScopeFlags;
use vize_s0::{CompactString, FxHashMap};

mod scope;

use scope::{
    collect_scope_bindings, merge_scope_bindings, record_binding_names, record_declaration,
};

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
    scopes: Vec<FxHashMap<CompactString, bool>>,
}

impl<'a> Visit<'a> for RefOperandVisitor<'_> {
    fn visit_program(&mut self, it: &Program<'a>) {
        self.scopes.push(collect_scope_bindings(&it.body));
        walk_program(self, it);
        self.scopes.pop();
    }

    fn visit_function(&mut self, it: &Function<'a>, flags: ScopeFlags) {
        // Parameters shadow outer bindings of the same name (a param is never a
        // ref binding for our purposes), as do the function body's own locals.
        let mut frame = FxHashMap::default();
        for param in &it.params.items {
            record_binding_names(&param.pattern, false, &mut frame);
        }
        if let Some(rest) = &it.params.rest {
            record_binding_names(&rest.rest.argument, false, &mut frame);
        }
        if let Some(body) = it.body.as_ref() {
            merge_scope_bindings(&body.statements, &mut frame);
        }
        self.scopes.push(frame);
        walk_function(self, it, flags);
        self.scopes.pop();
    }

    fn visit_arrow_function_expression(&mut self, it: &oxc_ast::ast::ArrowFunctionExpression<'a>) {
        let mut frame = FxHashMap::default();
        for param in &it.params.items {
            record_binding_names(&param.pattern, false, &mut frame);
        }
        if let Some(rest) = &it.params.rest {
            record_binding_names(&rest.rest.argument, false, &mut frame);
        }
        merge_scope_bindings(&it.body.statements, &mut frame);
        self.scopes.push(frame);
        oxc_ast_visit::walk::walk_arrow_function_expression(self, it);
        self.scopes.pop();
    }

    fn visit_block_statement(&mut self, it: &BlockStatement<'a>) {
        self.scopes.push(collect_scope_bindings(&it.body));
        walk_block_statement(self, it);
        self.scopes.pop();
    }

    fn visit_catch_clause(&mut self, it: &CatchClause<'a>) {
        let mut frame = FxHashMap::default();
        if let Some(param) = &it.param {
            record_binding_names(&param.pattern, false, &mut frame);
        }
        self.scopes.push(frame);
        walk_catch_clause(self, it);
        self.scopes.pop();
    }

    fn visit_for_statement(&mut self, it: &ForStatement<'a>) {
        // A `for (let x = ...; ...)` header introduces a scope around the test,
        // update, and body. Collect any bindings declared in the init.
        let mut frame = FxHashMap::default();
        if let Some(oxc_ast::ast::ForStatementInit::VariableDeclaration(declaration)) = &it.init {
            record_declaration(declaration, &mut frame);
        }
        self.scopes.push(frame);
        if let Some(test) = &it.test {
            self.check_operand(test);
        }
        walk_for_statement(self, it);
        self.scopes.pop();
    }

    fn visit_for_in_statement(&mut self, it: &ForInStatement<'a>) {
        let mut frame = FxHashMap::default();
        if let ForStatementLeft::VariableDeclaration(declaration) = &it.left {
            record_declaration(declaration, &mut frame);
        }
        self.scopes.push(frame);
        walk_for_in_statement(self, it);
        self.scopes.pop();
    }

    fn visit_for_of_statement(&mut self, it: &ForOfStatement<'a>) {
        let mut frame = FxHashMap::default();
        if let ForStatementLeft::VariableDeclaration(declaration) = &it.left {
            record_declaration(declaration, &mut frame);
        }
        self.scopes.push(frame);
        walk_for_of_statement(self, it);
        self.scopes.pop();
    }

    // --- Operator contexts ---

    fn visit_update_expression(&mut self, it: &UpdateExpression<'a>) {
        // `count++`, `--count`: the target is read and written as a value.
        if let oxc_ast::ast::SimpleAssignmentTarget::AssignmentTargetIdentifier(id) = &it.argument
            && self.is_ref(&id.name)
        {
            self.report(id.span, &id.name);
        }
        walk_update_expression(self, it);
    }

    fn visit_assignment_expression(&mut self, it: &AssignmentExpression<'a>) {
        // A compound assignment (`count += 1`, `count ??= x`) reads the target
        // as a value first. A plain `count = x` only rebinds and is left alone.
        if !it.operator.is_assign()
            && let oxc_ast::ast::AssignmentTarget::AssignmentTargetIdentifier(id) = &it.left
            && self.is_ref(&id.name)
        {
            self.report(id.span, &id.name);
        }
        walk_assignment_expression(self, it);
    }

    fn visit_unary_expression(&mut self, it: &UnaryExpression<'a>) {
        // `!count`, `-count`, `+count`, `~count` read the value. `typeof`,
        // `void`, and `delete` inspect the binding itself and are not flagged.
        if !matches!(
            it.operator,
            UnaryOperator::Typeof | UnaryOperator::Void | UnaryOperator::Delete
        ) {
            self.check_operand(&it.argument);
        }
        walk_unary_expression(self, it);
    }

    fn visit_binary_expression(&mut self, it: &BinaryExpression<'a>) {
        self.check_operand(&it.left);
        self.check_operand(&it.right);
        walk_binary_expression(self, it);
    }

    fn visit_logical_expression(&mut self, it: &LogicalExpression<'a>) {
        self.check_operand(&it.left);
        self.check_operand(&it.right);
        walk_logical_expression(self, it);
    }

    fn visit_conditional_expression(&mut self, it: &ConditionalExpression<'a>) {
        self.check_operand(&it.test);
        walk_conditional_expression(self, it);
    }

    fn visit_template_literal(&mut self, it: &TemplateLiteral<'a>) {
        for expression in &it.expressions {
            self.check_operand(expression);
        }
        walk_template_literal(self, it);
    }

    fn visit_if_statement(&mut self, it: &IfStatement<'a>) {
        self.check_operand(&it.test);
        walk_if_statement(self, it);
    }

    fn visit_while_statement(&mut self, it: &WhileStatement<'a>) {
        self.check_operand(&it.test);
        walk_while_statement(self, it);
    }

    fn visit_do_while_statement(&mut self, it: &DoWhileStatement<'a>) {
        self.check_operand(&it.test);
        walk_do_while_statement(self, it);
    }
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
            if let Some(&is_ref) = frame.get(name) {
                return is_ref;
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
