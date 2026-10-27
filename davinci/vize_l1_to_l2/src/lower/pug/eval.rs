//! pug's build-time JavaScript, evaluated without an engine.
//!
//! pug compiles a template into one function and runs it with no data,
//! so whatever its code computes from literals and from names the
//! template itself binds (`- var`, `each`) is fixed when Vue compiles.
//! This module evaluates exactly that subset over the oxc AST and refuses
//! the rest with a reason: names the template does not bind (pug reads
//! them from its options, never from the component), calls, assignments,
//! and comparisons that would depend on object identity.

use alloc::vec::Vec as StdVec;

use oxc_ast::ast::{
    ArrayExpressionElement, BigIntLiteral, BinaryOperator, BindingPattern, ChainExpression,
    Expression, ImportExpression, LogicalExpression, LogicalOperator, NumericLiteral,
    ObjectPropertyKind, PrivateInExpression, PropertyKey, PropertyKind, Statement, UnaryOperator,
    VariableDeclarationKind,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_l0::{Allocator, String, cstr};

use super::literal::string_literal;
use super::ops::{binary_value, is_array_index, property};
use super::scope::{Kind, Scope};
use super::value::{Value, number_string};

/// Why a piece of pug code has no fixed value.
pub(super) type Refusal = String;

/// A value, and whether it read any template binding (pug only folds
/// binding-free values at compile time; the rest take its runtime path).
#[derive(Debug)]
pub(super) struct Evaluated {
    pub value: Value,
    pub constant: bool,
}

/// Evaluate one pug expression (`= …`, an attribute value, `#{…}`, a
/// condition or an `each` subject).
pub(super) fn expression(scope: &Scope, source: &str) -> Result<Evaluated, Refusal> {
    with_expression(source, |expression, wrapped| {
        let mut evaluator = Evaluator {
            scope,
            source: wrapped,
            constant: true,
        };
        let value = evaluator.eval(expression)?;
        Ok(Evaluated {
            value,
            constant: evaluator.constant,
        })
    })
}

/// Whether `source` is an expression pug's lexer accepts — it checks
/// every expression, including those in branches it skips.
pub(super) fn parses(source: &str) -> bool {
    with_expression(source, |_, _| Ok(())).is_ok()
}

/// Parse `source` as `pug-lexer` checks expressions (`is-expression`:
/// acorn 7 at ES2019, line comments rejected) and run `then` on it.
fn with_expression<T>(
    source: &str,
    then: impl FnOnce(&Expression<'_>, &str) -> Result<T, Refusal>,
) -> Result<T, Refusal> {
    let allocator = Allocator::default();
    let wrapped = cstr!("({source}\n)");
    let parsed = Parser::new(allocator.as_oxc(), &wrapped, SourceType::script()).parse();
    let invalid = !parsed.diagnostics.is_empty()
        || parsed.panicked
        || parsed
            .program
            .comments
            .iter()
            .any(|comment| comment.is_line());
    let [Statement::ExpressionStatement(statement)] = parsed.program.body.as_slice() else {
        return Err(syntax_error(source));
    };
    let expression = match &statement.expression {
        Expression::ParenthesizedExpression(inner) => &inner.expression,
        other => other,
    };
    let mut syntax = Es2019 { newer: false };
    syntax.visit_expression(expression);
    if invalid || syntax.newer {
        return Err(syntax_error(source));
    }
    then(expression, &wrapped)
}

/// Flags syntax newer than ES2019, which acorn 7 rejects.
struct Es2019 {
    newer: bool,
}

impl<'a> Visit<'a> for Es2019 {
    fn visit_logical_expression(&mut self, it: &LogicalExpression<'a>) {
        self.newer |= it.operator == LogicalOperator::Coalesce;
        walk::walk_logical_expression(self, it);
    }

    fn visit_chain_expression(&mut self, _: &ChainExpression<'a>) {
        self.newer = true;
    }

    fn visit_numeric_literal(&mut self, it: &NumericLiteral<'a>) {
        self.newer |= it.raw.is_some_and(|raw| raw.as_str().contains('_'));
    }

    fn visit_big_int_literal(&mut self, _: &BigIntLiteral<'a>) {
        self.newer = true;
    }

    fn visit_import_expression(&mut self, _: &ImportExpression<'a>) {
        self.newer = true;
    }

    fn visit_private_in_expression(&mut self, _: &PrivateInExpression<'a>) {
        self.newer = true;
    }
}

/// Whether `source` is an object literal (a fresh object pug may
/// modify in place).
pub(super) fn is_object_literal(source: &str) -> bool {
    with_expression(source, |expression, _| {
        Ok(matches!(expression, Expression::ObjectExpression(_)))
    })
    .unwrap_or(false)
}

/// Bind a mixin's parameter list (pug pastes it into
/// `function(…)`) to `args` in the current function frame; a missing
/// argument takes its default.
pub(super) fn bind_parameters(
    scope: &mut Scope,
    params: &str,
    args: &[Value],
) -> Result<(), Refusal> {
    let allocator = Allocator::default();
    let wrapped = cstr!("(function({params}\n){{}})");
    let parsed = Parser::new(allocator.as_oxc(), &wrapped, SourceType::script()).parse();
    let function = match parsed.program.body.as_slice() {
        [Statement::ExpressionStatement(statement)] if parsed.diagnostics.is_empty() => {
            match &statement.expression {
                Expression::ParenthesizedExpression(inner) => match &inner.expression {
                    Expression::FunctionExpression(function) => Some(function),
                    _ => None,
                },
                _ => None,
            }
        }
        _ => None,
    };
    let Some(function) = function else {
        return Err(cstr!(
            "the parameters `{}` are not valid JavaScript",
            params.trim()
        ));
    };
    if function.params.rest.is_some() {
        return Err(String::from(
            "only the last parameter can be a rest parameter",
        ));
    }
    for (index, param) in function.params.items.iter().enumerate() {
        let BindingPattern::BindingIdentifier(name) = &param.pattern else {
            return Err(String::from("only plain parameter names can be bound"));
        };
        let mut value = args.get(index).cloned().unwrap_or(Value::Undefined);
        if let (Value::Undefined, Some(init)) = (&value, &param.initializer) {
            let mut evaluator = Evaluator {
                scope,
                source: &wrapped,
                constant: true,
            };
            value = evaluator.eval(init)?;
        }
        scope.declare_var(name.name.as_str(), value)?;
    }
    Ok(())
}

/// Run unbuffered code (`- …`): variable declarations only.
pub(super) fn statements(scope: &mut Scope, source: &str) -> Result<(), Refusal> {
    let allocator = Allocator::default();
    let parsed = Parser::new(allocator.as_oxc(), source, SourceType::script()).parse();
    if !parsed.diagnostics.is_empty() || parsed.panicked {
        return Err(syntax_error(source));
    }
    for statement in &parsed.program.body {
        match statement {
            Statement::EmptyStatement(_) => {}
            Statement::VariableDeclaration(declaration) => {
                let kind = match declaration.kind {
                    VariableDeclarationKind::Var => Kind::Var,
                    VariableDeclarationKind::Let => Kind::Let,
                    VariableDeclarationKind::Const => Kind::Const,
                    _ => return Err(String::from(UNSUPPORTED_STATEMENT)),
                };
                for declarator in &declaration.declarations {
                    let BindingPattern::BindingIdentifier(name) = &declarator.id else {
                        return Err(String::from("only plain names can be declared"));
                    };
                    let value = match &declarator.init {
                        Some(init) => {
                            let mut evaluator = Evaluator {
                                scope,
                                source,
                                constant: true,
                            };
                            Some(evaluator.eval(init)?)
                        }
                        None => (kind == Kind::Let).then_some(Value::Undefined),
                    };
                    scope.declare(name.name.as_str(), kind, value)?;
                }
            }
            _ => return Err(String::from(UNSUPPORTED_STATEMENT)),
        }
    }
    Ok(())
}

const UNSUPPORTED_STATEMENT: &str = "only variable declarations (`var`, `let`, `const`) can run";

fn syntax_error(source: &str) -> Refusal {
    cstr!("`{}` is not valid JavaScript", source.trim())
}

struct Evaluator<'s, 'c> {
    scope: &'s Scope,
    source: &'c str,
    constant: bool,
}

impl Evaluator<'_, '_> {
    fn text(&self, node: &impl GetSpan) -> &str {
        let span = node.span();
        self.source
            .get(span.start as usize..span.end as usize)
            .unwrap_or_default()
    }

    fn eval(&mut self, expression: &Expression<'_>) -> Result<Value, Refusal> {
        vize_l0::ensure_sufficient_stack(|| self.eval_inner(expression))
    }

    fn eval_inner(&mut self, expression: &Expression<'_>) -> Result<Value, Refusal> {
        Ok(match expression {
            Expression::BooleanLiteral(literal) => Value::Bool(literal.value),
            Expression::NullLiteral(_) => Value::Null,
            Expression::NumericLiteral(literal) => {
                let raw = self.text(&**literal);
                let legacy_octal = raw.len() > 1
                    && raw.starts_with('0')
                    && raw.as_bytes().get(1).is_some_and(u8::is_ascii_digit);
                if legacy_octal {
                    return Err(self.unsupported(&**literal));
                }
                Value::Number(literal.value)
            }
            Expression::StringLiteral(literal) => {
                let raw = self.text(&**literal);
                let quote = raw.chars().next().unwrap_or('"');
                match string_literal(raw, quote) {
                    Some(text) => Value::Str(text),
                    None => return Err(self.unsupported(&**literal)),
                }
            }
            Expression::TemplateLiteral(template) => {
                let mut out = String::default();
                for (index, quasi) in template.quasis.iter().enumerate() {
                    let raw = quasi.value.raw.as_str();
                    let cooked = quasi.value.cooked.as_ref().map(|cooked| cooked.as_str());
                    match cooked {
                        Some(cooked)
                            if !raw.contains(['\n', '\r']) && !cooked.contains('\u{fffd}') =>
                        {
                            out.push_str(cooked);
                        }
                        _ => return Err(self.unsupported(&**template)),
                    }
                    if let Some(part) = template.expressions.get(index) {
                        let value = self.eval(part)?;
                        out.push_str(&self.printable(value, &**template)?.to_js_string());
                    }
                }
                Value::Str(out)
            }
            Expression::Identifier(identifier) => {
                let name = identifier.name.as_str();
                match self.scope.lookup(name) {
                    Some(value) => {
                        self.constant = false;
                        value.clone()
                    }
                    None => match name {
                        "undefined" => Value::Undefined,
                        "NaN" => Value::Number(f64::NAN),
                        "Infinity" => Value::Number(f64::INFINITY),
                        _ => {
                            return Err(cstr!("`{name}` is not defined by the template"));
                        }
                    },
                }
            }
            Expression::ParenthesizedExpression(inner) => self.eval(&inner.expression)?,
            Expression::ArrayExpression(array) => {
                let mut items = StdVec::with_capacity(array.elements.len());
                for element in &array.elements {
                    match element {
                        ArrayExpressionElement::SpreadElement(_)
                        | ArrayExpressionElement::Elision(_) => {
                            return Err(self.unsupported(&**array));
                        }
                        element => match element.as_expression() {
                            Some(item) => items.push(self.eval(item)?),
                            None => return Err(self.unsupported(&**array)),
                        },
                    }
                }
                Value::Array(items)
            }
            Expression::ObjectExpression(object) => {
                let mut entries: StdVec<(String, Value)> = StdVec::new();
                for property in &object.properties {
                    let ObjectPropertyKind::ObjectProperty(property) = property else {
                        return Err(self.unsupported(&**object));
                    };
                    if property.kind != PropertyKind::Init || property.method {
                        return Err(self.unsupported(&**object));
                    }
                    let key = match &property.key {
                        PropertyKey::StaticIdentifier(name) => String::from(name.name.as_str()),
                        PropertyKey::PrivateIdentifier(_) => {
                            return Err(self.unsupported(&**object));
                        }
                        key => match key.as_expression() {
                            Some(key) if property.computed => {
                                let value = self.eval(key)?;
                                self.printable(value, &**object)?.to_js_string()
                            }
                            Some(
                                key
                                @ (Expression::StringLiteral(_) | Expression::NumericLiteral(_)),
                            ) => self.eval(key)?.to_js_string(),
                            _ => return Err(self.unsupported(&**object)),
                        },
                    };
                    // Integer-like keys enumerate before the others and
                    // `__proto__` sets the prototype: neither keeps order.
                    if key == "__proto__" || is_array_index(&key) {
                        return Err(self.unsupported(&**object));
                    }
                    let value = self.eval(&property.value)?;
                    match entries.iter_mut().find(|(bound, _)| *bound == key) {
                        Some((_, slot)) => *slot = value,
                        None => entries.push((key, value)),
                    }
                }
                Value::Object(entries)
            }
            Expression::UnaryExpression(unary) => {
                let operand = self.eval(&unary.argument)?;
                match unary.operator {
                    UnaryOperator::LogicalNot => Value::Bool(!operand.truthy()),
                    UnaryOperator::Typeof => Value::str(operand.type_of()),
                    UnaryOperator::Void => Value::Undefined,
                    UnaryOperator::UnaryNegation => {
                        Value::Number(-self.printable(operand, &**unary)?.to_number())
                    }
                    UnaryOperator::UnaryPlus => {
                        Value::Number(self.printable(operand, &**unary)?.to_number())
                    }
                    _ => return Err(self.unsupported(&**unary)),
                }
            }
            Expression::LogicalExpression(logical) => {
                let left = self.eval(&logical.left)?;
                let short = match logical.operator {
                    LogicalOperator::And => !left.truthy(),
                    LogicalOperator::Or => left.truthy(),
                    LogicalOperator::Coalesce => !left.is_nullish(),
                };
                if short {
                    left
                } else {
                    self.eval(&logical.right)?
                }
            }
            Expression::ConditionalExpression(conditional) => {
                if self.eval(&conditional.test)?.truthy() {
                    self.eval(&conditional.consequent)?
                } else {
                    self.eval(&conditional.alternate)?
                }
            }
            Expression::BinaryExpression(binary) => {
                let left = self.eval(&binary.left)?;
                let right = self.eval(&binary.right)?;
                // Identity and nullish comparisons never print a function.
                let prints = match binary.operator {
                    BinaryOperator::StrictEquality | BinaryOperator::StrictInequality => false,
                    BinaryOperator::Equality | BinaryOperator::Inequality => {
                        !(left.is_nullish()
                            || right.is_nullish()
                            || matches!((&left, &right), (Value::Function(_), Value::Function(_))))
                    }
                    _ => true,
                };
                if prints && (left.has_function() || right.has_function()) {
                    return Err(self.unsupported(&**binary));
                }
                match binary_value(binary.operator, &left, &right) {
                    Some(value) => value,
                    None => return Err(self.unsupported(&**binary)),
                }
            }
            Expression::StaticMemberExpression(member) if !member.optional => {
                let object = self.eval(&member.object)?;
                match property(&object, member.property.name.as_str()) {
                    Some(value) => value,
                    None => return Err(self.unsupported(&**member)),
                }
            }
            Expression::ComputedMemberExpression(member) if !member.optional => {
                let object = self.eval(&member.object)?;
                let key = self.eval(&member.expression)?;
                let key = match key {
                    Value::Str(text) => text,
                    Value::Number(number) => number_string(number),
                    _ => return Err(self.unsupported(&**member)),
                };
                match property(&object, &key) {
                    Some(value) => value,
                    None => return Err(self.unsupported(&**member)),
                }
            }
            other => return Err(self.unsupported(other)),
        })
    }

    /// `value`, unless converting it would print a function's source.
    fn printable(&self, value: Value, node: &impl GetSpan) -> Result<Value, Refusal> {
        if value.has_function() {
            return Err(self.unsupported(node));
        }
        Ok(value)
    }

    fn unsupported(&self, node: &impl GetSpan) -> Refusal {
        cstr!("`{}` needs a JavaScript engine", self.text(node))
    }
}
