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
    AssignmentExpression, BinaryExpression, BindingPattern, ConditionalExpression,
    DoWhileStatement, Expression, ForStatement, Function, IfStatement, LogicalExpression, Program,
    Statement, UnaryExpression, UpdateExpression, VariableDeclaration, WhileStatement,
};
use oxc_ast_visit::{
    Visit,
    walk::{
        walk_assignment_expression, walk_binary_expression, walk_conditional_expression,
        walk_do_while_statement, walk_for_statement, walk_function, walk_if_statement,
        walk_logical_expression, walk_program, walk_unary_expression, walk_update_expression,
        walk_while_statement,
    },
};
use oxc_span::Span;
use oxc_syntax::operator::UnaryOperator;
use oxc_syntax::scope::ScopeFlags;
use vize_carton::{CompactString, FxHashMap};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-ref-as-operand",
    description: "Require ref-bound variables to be accessed via `.value` when used as an operand",
    default_severity: Severity::Error,
};

/// The ref factories whose direct call result is a single ref object that must
/// be unwrapped with `.value`. Mirrors eslint-plugin-vue's tracked composables;
/// notably excludes `reactive`/`toRefs` (their results are not single refs).
const REF_FACTORIES: [&str; 5] = ["ref", "computed", "shallowRef", "toRef", "customRef"];

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
/// and every `for` header; bindings introduced by `var`/`let`/`const` in that
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
        merge_scope_bindings(&it.body.statements, &mut frame);
        self.scopes.push(frame);
        oxc_ast_visit::walk::walk_arrow_function_expression(self, it);
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

/// Collect the top-level bindings of a statement list into a fresh frame.
fn collect_scope_bindings(statements: &[Statement<'_>]) -> FxHashMap<CompactString, bool> {
    let mut frame = FxHashMap::default();
    merge_scope_bindings(statements, &mut frame);
    frame
}

/// Record every variable binding declared directly in `statements` into `frame`,
/// marking each as a ref (initializer is a ref factory call) or not. Nested
/// blocks/functions are not descended into: their bindings belong to their own
/// frame, pushed when the visitor enters them.
fn merge_scope_bindings(statements: &[Statement<'_>], frame: &mut FxHashMap<CompactString, bool>) {
    for statement in statements {
        if let Statement::VariableDeclaration(declaration) = statement {
            record_declaration(declaration, frame);
        }
    }
}

/// Record the bindings of a single variable declaration. A plain
/// `const NAME = factory(...)` marks `NAME` as a ref; any other initializer (or
/// a destructuring pattern) marks the bound names as non-refs so they shadow.
fn record_declaration(
    declaration: &VariableDeclaration<'_>,
    frame: &mut FxHashMap<CompactString, bool>,
) {
    for declarator in &declaration.declarations {
        let is_ref = matches!(&declarator.id, BindingPattern::BindingIdentifier(_))
            && declarator.init.as_ref().is_some_and(is_ref_factory_call);
        record_binding_names(&declarator.id, is_ref, frame);
    }
}

/// Insert the names bound by `pattern` into `frame` with the given ref flag.
/// Only a plain identifier can be a ref binding; destructured names are always
/// recorded as non-refs (so they correctly shadow an outer ref of that name).
fn record_binding_names(
    pattern: &BindingPattern<'_>,
    is_ref: bool,
    frame: &mut FxHashMap<CompactString, bool>,
) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => {
            frame.insert(CompactString::from(id.name.as_str()), is_ref);
        }
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                record_binding_names(&property.value, false, frame);
            }
            if let Some(rest) = &object.rest {
                record_binding_names(&rest.argument, false, frame);
            }
        }
        BindingPattern::ArrayPattern(array) => {
            for element in array.elements.iter().flatten() {
                record_binding_names(element, false, frame);
            }
            if let Some(rest) = &array.rest {
                record_binding_names(&rest.argument, false, frame);
            }
        }
        BindingPattern::AssignmentPattern(assignment) => {
            record_binding_names(&assignment.left, false, frame);
        }
    }
}

/// Whether `expression` is a direct call to a known ref factory, e.g. `ref(0)`
/// or `computed(() => ...)`. Parenthesized and TS-cast wrappers are peeled.
fn is_ref_factory_call(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::CallExpression(call) => {
            matches!(&call.callee, Expression::Identifier(id) if REF_FACTORIES.contains(&id.name.as_str()))
        }
        Expression::ParenthesizedExpression(paren) => is_ref_factory_call(&paren.expression),
        Expression::TSAsExpression(ts) => is_ref_factory_call(&ts.expression),
        Expression::TSSatisfiesExpression(ts) => is_ref_factory_call(&ts.expression),
        Expression::TSNonNullExpression(ts) => is_ref_factory_call(&ts.expression),
        _ => false,
    }
}

#[cfg(test)]
mod tests;
