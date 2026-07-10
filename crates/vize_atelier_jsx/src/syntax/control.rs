use oxc_ast::ast::{
    ArrowFunctionExpression, CallExpression, ConditionalExpression, Expression, Function,
    LogicalExpression, LogicalOperator, Statement,
};
use oxc_span::{GetSpan, Span};
use vize_carton::String;

use super::build::SyntaxBuilder;
use super::{JsxSyntaxBinding, JsxSyntaxBranch, JsxSyntaxExpression, JsxSyntaxNode};

impl SyntaxBuilder<'_> {
    /// Snapshot only expressions that represent render structure. Plain values
    /// return `None` and become an expression node at their containing site.
    pub(super) fn render_expression(&self, expression: &Expression<'_>) -> Option<JsxSyntaxNode> {
        match unwrap_parentheses(expression) {
            Expression::JSXElement(element) => Some(self.element(element)),
            Expression::JSXFragment(fragment) => Some(self.fragment(fragment)),
            Expression::LogicalExpression(logical) => self.logical(logical),
            Expression::ConditionalExpression(conditional) => self.conditional(conditional),
            Expression::CallExpression(call) => self.map_call(call),
            _ => None,
        }
    }

    fn logical(&self, logical: &LogicalExpression<'_>) -> Option<JsxSyntaxNode> {
        let body = self.render_expression(&logical.right)?;
        let left_span = logical.left.span();
        let left = self.slice(left_span);
        let condition = match logical.operator {
            LogicalOperator::And => self.expression(left_span),
            LogicalOperator::Or => {
                JsxSyntaxExpression::synthetic(synthesized_condition("!(", left, ")"), left_span)
            }
            LogicalOperator::Coalesce => JsxSyntaxExpression::synthetic(
                synthesized_condition("(", left, ") == null"),
                left_span,
            ),
        };
        Some(JsxSyntaxNode::If {
            branches: vec![JsxSyntaxBranch {
                condition: Some(condition),
                body: vec![body],
                span: logical.right.span().into(),
            }],
            span: logical.span.into(),
        })
    }

    fn conditional(&self, conditional: &ConditionalExpression<'_>) -> Option<JsxSyntaxNode> {
        let (consequent, consequent_renders) = self.branch(&conditional.consequent);
        let (alternate, alternate_renders) = self.branch(&conditional.alternate);
        if !consequent_renders && !alternate_renders {
            return None;
        }
        Some(JsxSyntaxNode::If {
            branches: vec![
                JsxSyntaxBranch {
                    condition: Some(self.expression(conditional.test.span())),
                    body: vec![consequent],
                    span: conditional.consequent.span().into(),
                },
                JsxSyntaxBranch {
                    condition: None,
                    body: vec![alternate],
                    span: conditional.alternate.span().into(),
                },
            ],
            span: conditional.span.into(),
        })
    }

    fn branch(&self, expression: &Expression<'_>) -> (JsxSyntaxNode, bool) {
        let expression = unwrap_parentheses(expression);
        match self.render_expression(expression) {
            Some(node) => (node, true),
            None => (
                JsxSyntaxNode::Expression {
                    expression: self.expression(expression.span()),
                    span: expression.span().into(),
                },
                false,
            ),
        }
    }

    fn map_call(&self, call: &CallExpression<'_>) -> Option<JsxSyntaxNode> {
        let Expression::StaticMemberExpression(member) = unwrap_parentheses(&call.callee) else {
            return None;
        };
        if member.property.name.as_str() != "map"
            || member.optional
            || call.optional
            || call.arguments.len() != 1
        {
            return None;
        }
        let callback = call.arguments.first()?.as_expression()?;
        let (parameters, returned) = match unwrap_parentheses(callback) {
            Expression::ArrowFunctionExpression(arrow) => {
                (&arrow.params, arrow_return_expression(arrow)?)
            }
            Expression::FunctionExpression(function) => {
                (&function.params, function_return_expression(function)?)
            }
            _ => return None,
        };
        let body = self.render_expression(returned)?;
        Some(JsxSyntaxNode::For {
            source: self.expression(member.object.span()),
            value: parameters
                .items
                .first()
                .map(|parameter| self.binding(parameter.pattern.span())),
            index: parameters
                .items
                .get(1)
                .map(|parameter| self.binding(parameter.pattern.span())),
            body: vec![body],
            span: call.span.into(),
        })
    }

    fn binding(&self, span: Span) -> JsxSyntaxBinding {
        JsxSyntaxBinding {
            pattern: self.slice(span).into(),
            span: span.into(),
        }
    }
}

fn unwrap_parentheses<'e, 'a>(mut expression: &'e Expression<'a>) -> &'e Expression<'a> {
    while let Expression::ParenthesizedExpression(parenthesized) = expression {
        expression = &parenthesized.expression;
    }
    expression
}

fn arrow_return_expression<'e, 'a>(
    arrow: &'e ArrowFunctionExpression<'a>,
) -> Option<&'e Expression<'a>> {
    if arrow.expression {
        let Statement::ExpressionStatement(statement) = arrow.body.statements.first()? else {
            return None;
        };
        Some(&statement.expression)
    } else {
        block_return_expression(&arrow.body.statements)
    }
}

fn function_return_expression<'e, 'a>(function: &'e Function<'a>) -> Option<&'e Expression<'a>> {
    block_return_expression(&function.body.as_ref()?.statements)
}

fn block_return_expression<'e, 'a>(statements: &'e [Statement<'a>]) -> Option<&'e Expression<'a>> {
    statements.iter().find_map(|statement| {
        let Statement::ReturnStatement(statement) = statement else {
            return None;
        };
        statement.argument.as_ref()
    })
}

fn synthesized_condition(prefix: &str, expression: &str, suffix: &str) -> Box<str> {
    let mut code = String::new("");
    code.reserve(prefix.len() + expression.len() + suffix.len());
    code.push_str(prefix);
    code.push_str(expression);
    code.push_str(suffix);
    code.as_str().into()
}
