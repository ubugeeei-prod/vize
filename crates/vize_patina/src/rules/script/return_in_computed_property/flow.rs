//! Reachable getter completions; nested function returns never enter this flow.
use oxc_ast::ast::{Expression, Statement};
use oxc_ast_visit::Visit;

const NORMAL: u8 = 1;
const VALUE: u8 = 2;
const UNDEFINED: u8 = 4;
const THROW: u8 = 8;

#[derive(Clone, Copy)]
struct Jump<'a> {
    label: Option<&'a str>,
    continuing: bool,
}

#[derive(Default)]
struct Flow<'a> {
    exits: u8,
    jumps: Vec<Jump<'a>>,
    saw_value: bool,
}

impl<'a> Flow<'a> {
    fn new(exits: u8) -> Self {
        Self {
            exits,
            jumps: Vec::new(),
            saw_value: exits & VALUE != 0,
        }
    }
    fn union(mut self, other: Self) -> Self {
        self.exits |= other.exits;
        self.saw_value |= other.saw_value;
        self.jumps.extend(other.jumps);
        self
    }
    fn then(mut self, next: impl FnOnce() -> Self) -> Self {
        if self.exits & NORMAL == 0 {
            return self;
        }
        self.exits &= !NORMAL;
        self.union(next())
    }
    fn consume_break(mut self, label: Option<&str>, include_continue: bool) -> Self {
        self.jumps.retain(|jump| {
            let consumed = (jump.label.is_none() || jump.label == label)
                && (!jump.continuing || include_continue);
            if consumed && !jump.continuing {
                self.exits |= NORMAL;
            }
            !consumed
        });
        self
    }
}

/// Every path must leave through `return` or `throw`, and, like the reference
/// rule's `treatUndefinedAsUnspecified` bookkeeping, the getter's last `return`
/// in source order must carry a value. An early bare `return;` guard followed
/// by a valued `return` is therefore accepted.
///
/// `typed` says the function declares a return type that excludes `undefined`.
/// TypeScript then rejects any reachable implicit completion and, unlike this
/// syntactic pass, knows the discriminant's type, so it alone decides whether a
/// `switch` without `default` covers every value. Without a declared type the
/// missing `default` stays a path out of the switch, as in eslint-plugin-vue.
pub(super) fn returns_value(statements: &[Statement<'_>], typed: bool) -> bool {
    let flow = Paths { typed }.sequence(statements);
    if flow.exits & NORMAL != 0 || !flow.jumps.is_empty() {
        return false;
    }
    let mut last = LastReturn(None);
    for statement in statements {
        last.visit_statement(statement);
    }
    last.0 == Some(true)
}

/// Whether the last `return` of the getter itself (not of a nested function)
/// has an argument.
struct LastReturn(Option<bool>);
impl<'a> Visit<'a> for LastReturn {
    fn visit_function(&mut self, _: &oxc_ast::ast::Function<'a>, _: oxc_syntax::scope::ScopeFlags) {
    }
    fn visit_arrow_function_expression(&mut self, _: &oxc_ast::ast::ArrowFunctionExpression<'a>) {}
    fn visit_class(&mut self, _: &oxc_ast::ast::Class<'a>) {}
    fn visit_return_statement(&mut self, ret: &oxc_ast::ast::ReturnStatement<'a>) {
        self.0 = Some(ret.argument.is_some());
        if let Some(argument) = &ret.argument {
            self.visit_expression(argument);
        }
    }
}

/// Analysis settings shared by every statement of one function body.
#[derive(Clone, Copy)]
struct Paths {
    typed: bool,
}

impl Paths {
    fn sequence<'a>(self, statements: &'a [Statement<'a>]) -> Flow<'a> {
        let mut flow = Flow::new(NORMAL);
        for statement in statements {
            flow = flow.then(|| self.statement_flow(statement, None));
        }
        flow
    }

    fn statement_flow<'a>(self, statement: &'a Statement<'a>, label: Option<&'a str>) -> Flow<'a> {
        match statement {
            Statement::ReturnStatement(ret) => Flow::new(if let Some(value) = &ret.argument {
                VALUE | expression_throw(value)
            } else {
                UNDEFINED
            }),
            Statement::ThrowStatement(_) => Flow::new(THROW),
            Statement::BreakStatement(jump) => Flow {
                jumps: vec![Jump {
                    label: jump.label.as_ref().map(|label| label.name.as_str()),
                    continuing: false,
                }],
                ..Flow::default()
            },
            Statement::ContinueStatement(jump) => Flow {
                jumps: vec![Jump {
                    label: jump.label.as_ref().map(|label| label.name.as_str()),
                    continuing: true,
                }],
                ..Flow::default()
            },
            Statement::BlockStatement(block) => self.sequence(&block.body),
            Statement::IfStatement(branch) => {
                let alternate = branch.alternate.as_ref().map_or_else(
                    || Flow::new(NORMAL),
                    |statement| self.statement_flow(statement, None),
                );
                self.statement_flow(&branch.consequent, None)
                    .union(alternate)
                    .union(Flow::new(expression_throw(&branch.test)))
            }
            Statement::SwitchStatement(switch) => {
                let mut suffix = Flow::new(NORMAL);
                let mut entries = Flow::default();
                for case in switch.cases.iter().rev() {
                    suffix = self.sequence(&case.consequent).then(|| suffix);
                    entries = entries.union(Flow {
                        exits: suffix.exits,
                        jumps: suffix.jumps.clone(),
                        saw_value: suffix.saw_value,
                    });
                    if let Some(test) = &case.test {
                        entries.exits |= expression_throw(test);
                    }
                }
                if !self.typed && !switch.cases.iter().any(|case| case.test.is_none()) {
                    entries.exits |= NORMAL;
                }
                entries.exits |= expression_throw(&switch.discriminant);
                entries.consume_break(None, false)
            }
            Statement::WhileStatement(loop_) => self
                .loop_flow(&loop_.body, label, truth(&loop_.test), false)
                .0
                .union(Flow::new(expression_throw(&loop_.test))),
            Statement::DoWhileStatement(loop_) => {
                let (mut body, reaches_test) =
                    self.loop_flow(&loop_.body, label, truth(&loop_.test), true);
                if reaches_test {
                    body.exits |= expression_throw(&loop_.test);
                }
                body
            }
            Statement::ForStatement(loop_) => {
                let (mut body, reaches_update) = self.loop_flow(
                    &loop_.body,
                    label,
                    loop_
                        .test
                        .as_ref()
                        .and_then(truth)
                        .or_else(|| loop_.test.is_none().then_some(true)),
                    false,
                );
                if let Some(init) = &loop_.init {
                    let mut finder = ThrowFinder(false);
                    finder.visit_for_statement_init(init);
                    if finder.0 {
                        body.exits |= THROW;
                    }
                }
                if let Some(test) = &loop_.test {
                    body.exits |= expression_throw(test);
                }
                if reaches_update && let Some(update) = &loop_.update {
                    body.exits |= expression_throw(update);
                }
                body
            }
            Statement::ForInStatement(loop_) => self
                .loop_flow(&loop_.body, label, None, false)
                .0
                .union(Flow::new(expression_throw(&loop_.right))),
            Statement::ForOfStatement(loop_) => self
                .loop_flow(&loop_.body, label, None, false)
                .0
                .union(Flow::new(expression_throw(&loop_.right))),
            Statement::LabeledStatement(labeled) => {
                let label = labeled.label.name.as_str();
                let mut flow = self.statement_flow(&labeled.body, Some(label));
                flow.jumps.retain(|jump| {
                    if jump.label == Some(label) && !jump.continuing {
                        flow.exits |= NORMAL;
                        false
                    } else {
                        true
                    }
                });
                flow
            }
            Statement::TryStatement(try_) => {
                let mut flow = self.sequence(&try_.block.body);
                if let Some(handler) = &try_.handler
                    && flow.exits & THROW != 0
                {
                    flow.exits &= !THROW;
                    flow = flow.union(self.sequence(&handler.body.body));
                }
                if let Some(finalizer) = &try_.finalizer {
                    if flow.exits == 0 && flow.jumps.is_empty() {
                        return flow;
                    }
                    let mut finally = self.sequence(&finalizer.body);
                    finally.saw_value |= flow.saw_value;
                    if finally.exits & NORMAL == 0 {
                        return finally;
                    }
                    finally.exits &= !NORMAL;
                    flow = flow.union(finally);
                }
                flow
            }
            Statement::FunctionDeclaration(_) => Flow::new(NORMAL),
            _ => {
                let mut finder = ThrowFinder(false);
                finder.visit_statement(statement);
                Flow::new(NORMAL | if finder.0 { THROW } else { 0 })
            }
        }
    }

    fn loop_flow<'a>(
        self,
        body: &'a Statement<'a>,
        label: Option<&'a str>,
        condition: Option<bool>,
        runs_once: bool,
    ) -> (Flow<'a>, bool) {
        if condition == Some(false) && !runs_once {
            return (Flow::new(NORMAL), false);
        }
        let mut flow = self.statement_flow(body, None);
        let reaches_test = flow.exits & NORMAL != 0
            || flow
                .jumps
                .iter()
                .any(|jump| jump.continuing && (jump.label.is_none() || jump.label == label));
        flow.exits &= !NORMAL;
        flow = flow.consume_break(label, true);
        if (!runs_once || reaches_test) && condition != Some(true) {
            flow.exits |= NORMAL;
        }
        (flow, reaches_test)
    }
}

fn truth(expression: &Expression<'_>) -> Option<bool> {
    match expression.get_inner_expression() {
        Expression::BooleanLiteral(value) => Some(value.value),
        Expression::NumericLiteral(value) => Some(value.value != 0.0 && !value.value.is_nan()),
        Expression::StringLiteral(value) => Some(!value.value.is_empty()),
        Expression::NullLiteral(_) => Some(false),
        _ => None,
    }
}

fn expression_throw(expression: &Expression<'_>) -> u8 {
    let mut finder = ThrowFinder(false);
    finder.visit_expression(expression);
    if finder.0 { THROW } else { 0 }
}

/// Like the reference code-path model, evaluating references and calls may
/// throw into catch. Merely declaring a nested function does not execute it.
struct ThrowFinder(bool);
impl<'a> Visit<'a> for ThrowFinder {
    fn visit_function(&mut self, _: &oxc_ast::ast::Function<'a>, _: oxc_syntax::scope::ScopeFlags) {
    }
    fn visit_arrow_function_expression(&mut self, _: &oxc_ast::ast::ArrowFunctionExpression<'a>) {}
    fn visit_identifier_reference(&mut self, _: &oxc_ast::ast::IdentifierReference<'a>) {
        self.0 = true;
    }
    fn visit_call_expression(&mut self, _: &oxc_ast::ast::CallExpression<'a>) {
        self.0 = true;
    }
    fn visit_new_expression(&mut self, _: &oxc_ast::ast::NewExpression<'a>) {
        self.0 = true;
    }
    fn visit_await_expression(&mut self, _: &oxc_ast::ast::AwaitExpression<'a>) {
        self.0 = true;
    }
    fn visit_throw_statement(&mut self, _: &oxc_ast::ast::ThrowStatement<'a>) {
        self.0 = true;
    }
}
