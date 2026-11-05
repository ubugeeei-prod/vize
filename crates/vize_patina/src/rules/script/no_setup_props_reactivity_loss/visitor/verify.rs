use super::Visitor;
use oxc_ast::ast::{
    Argument, ArrayExpressionElement, CallExpression, ChainElement, Expression, ObjectPropertyKind,
    PropertyKey,
};
use oxc_span::{GetSpan, Span};

impl Visitor<'_> {
    pub(super) fn verify_value(&mut self, pattern: Option<Span>, right: &Expression<'_>) {
        if self.is_call_like(right) {
            return;
        }
        let peeled = Self::peel(right);
        if matches!(
            peeled,
            Expression::ArrayExpression(_) | Expression::ObjectExpression(_)
        ) && let Some(scope) = self.access_scope(peeled)
        {
            self.report_get(peeled.span(), scope);
            return;
        }
        let (root, via_member) = Self::member_root(right);
        match root {
            Expression::TemplateLiteral(literal) => {
                for expr in &literal.expressions {
                    if let Some(scope) = self.access_scope(expr) {
                        self.report_get(expr.span(), scope);
                        return;
                    }
                }
            }
            Expression::ConditionalExpression(cond) => {
                for expr in [&cond.test, &cond.consequent, &cond.alternate] {
                    if let Some(scope) = self.access_scope(expr) {
                        self.report_get(expr.span(), scope);
                        return;
                    }
                }
            }
            Expression::Identifier(id) => {
                if let Some(span) = pattern
                    && let Some(scope) = self.container_scope(id.name.as_str())
                {
                    self.report_get(span, scope);
                    return;
                }
                if let Some(scope) = self.prop_scope(id.name.as_str(), via_member) {
                    self.report_get(id.span, scope);
                }
            }
            _ => {}
        }
    }

    fn is_call_like(&self, expr: &Expression<'_>) -> bool {
        match Self::peel(expr) {
            Expression::CallExpression(_) => true,
            Expression::ChainExpression(chain) => {
                matches!(chain.expression, ChainElement::CallExpression(_))
            }
            _ => false,
        }
    }

    pub(super) fn access_scope(&self, expr: &Expression<'_>) -> Option<bool> {
        match Self::peel(expr) {
            Expression::Identifier(id) => self.prop_scope(id.name.as_str(), false),
            Expression::StaticMemberExpression(member) => self.access_static(&member.object),
            Expression::PrivateFieldExpression(field) => self.access_static(&field.object),
            Expression::ComputedMemberExpression(member) => self
                .access_static(&member.object)
                .or_else(|| self.access_scope(&member.expression)),
            Expression::ChainExpression(chain) => self.access_chain(&chain.expression),
            Expression::CallExpression(call) => self.call_access(call),
            Expression::ArrayExpression(array) => array
                .elements
                .iter()
                .find_map(|element| self.access_element(element)),
            Expression::ObjectExpression(object) => object
                .properties
                .iter()
                .find_map(|property| self.access_property(property)),
            Expression::TemplateLiteral(literal) => literal
                .expressions
                .iter()
                .find_map(|expr| self.access_scope(expr)),
            Expression::ConditionalExpression(cond) => self
                .access_scope(&cond.test)
                .or_else(|| self.access_scope(&cond.consequent))
                .or_else(|| self.access_scope(&cond.alternate)),
            Expression::BinaryExpression(binary) => self
                .access_scope(&binary.left)
                .or_else(|| self.access_scope(&binary.right)),
            Expression::LogicalExpression(logical) => self
                .access_scope(&logical.left)
                .or_else(|| self.access_scope(&logical.right)),
            Expression::UnaryExpression(unary) => self.access_scope(&unary.argument),
            Expression::SequenceExpression(sequence) => sequence
                .expressions
                .iter()
                .find_map(|expr| self.access_scope(expr)),
            Expression::NewExpression(new) => new
                .arguments
                .iter()
                .find_map(|argument| self.access_argument(argument)),
            Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_) => None,
            _ => None,
        }
    }

    fn access_static(&self, object: &Expression<'_>) -> Option<bool> {
        if let Expression::Identifier(id) = Self::peel(object)
            && let Some(scope) = self.prop_scope(id.name.as_str(), true)
        {
            return Some(scope);
        }
        self.access_scope(object)
    }

    fn access_chain(&self, element: &ChainElement<'_>) -> Option<bool> {
        match element {
            ChainElement::CallExpression(call) => self.call_access(call),
            ChainElement::TSNonNullExpression(inner) => self.access_scope(&inner.expression),
            ChainElement::StaticMemberExpression(member) => self.access_static(&member.object),
            ChainElement::PrivateFieldExpression(field) => self.access_static(&field.object),
            ChainElement::ComputedMemberExpression(member) => self
                .access_static(&member.object)
                .or_else(|| self.access_scope(&member.expression)),
        }
    }

    pub(super) fn call_access(&self, call: &CallExpression<'_>) -> Option<bool> {
        self.access_scope(&call.callee).or_else(|| {
            call.arguments
                .iter()
                .find_map(|argument| self.access_argument(argument))
        })
    }

    fn access_argument(&self, argument: &Argument<'_>) -> Option<bool> {
        argument
            .as_expression()
            .and_then(|expr| self.access_scope(expr))
    }

    fn access_element(&self, element: &ArrayExpressionElement<'_>) -> Option<bool> {
        element
            .as_expression()
            .and_then(|expr| self.access_scope(expr))
    }

    fn access_property(&self, property: &ObjectPropertyKind<'_>) -> Option<bool> {
        match property {
            ObjectPropertyKind::ObjectProperty(property) => self
                .access_scope(&property.value)
                .or_else(|| self.key_access(&property.key)),
            ObjectPropertyKind::SpreadProperty(spread) => self.access_scope(&spread.argument),
        }
    }

    fn key_access(&self, key: &PropertyKey<'_>) -> Option<bool> {
        key.as_expression().and_then(|expr| self.access_scope(expr))
    }
}
