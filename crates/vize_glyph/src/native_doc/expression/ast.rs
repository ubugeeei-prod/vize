//! One immutable walk of genuine supported AST families.

use oxc_ast::ast::{CallExpression, ComputedMemberExpression, Expression, StaticMemberExpression};
use oxc_span::GetSpan;
use vize_l0::{Span, Vec};

use super::{
    Doc, ExpressionRefusal, MAX_EXPRESSION_DOCUMENT_DEPTH,
    source::{Context, Gap},
};

impl<'a> Context<'_, 'a> {
    pub fn node(
        &mut self,
        expression: &Expression<'a>,
        parent: Span,
        depth: usize,
    ) -> Result<Doc<'a>, ExpressionRefusal> {
        let span = self.decoded_span(expression.span())?;
        if span.start < parent.start || span.end > parent.end || span.start >= span.end {
            return Err(ExpressionRefusal::InvalidFraming { span });
        }
        if depth > MAX_EXPRESSION_DOCUMENT_DEPTH {
            return Err(ExpressionRefusal::DepthLimit { span });
        }
        self.authored_span(span)?;
        match expression {
            Expression::Identifier(_)
            | Expression::NumericLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_) => self.text(span),
            Expression::ParenthesizedExpression(parentheses) => {
                let inner = self.decoded_span(parentheses.expression.span())?;
                if inner.start <= span.start || inner.end >= span.end {
                    return Err(ExpressionRefusal::InvalidFraming { span });
                }
                let open = self.token(Span::new(span.start, inner.start), "(")?;
                if open.start != span.start {
                    return Err(ExpressionRefusal::InvalidFraming { span });
                }
                let mut parts = Vec::new_in(&self.allocator);
                parts.push(self.text(open)?);
                parts.push(self.gap(Span::new(open.end, inner.start), Gap::Empty)?);
                parts.push(self.node(&parentheses.expression, span, depth + 1)?);
                let close = self.token(Span::new(inner.end, span.end), ")")?;
                if close.end != span.end {
                    return Err(ExpressionRefusal::InvalidFraming { span });
                }
                parts.push(self.gap(Span::new(inner.end, close.start), Gap::Empty)?);
                parts.push(self.text(close)?);
                Ok(Doc::concat(parts).group(self.allocator))
            }
            Expression::BinaryExpression(binary) => self.infix(
                &binary.left,
                binary.operator.as_str(),
                &binary.right,
                span,
                depth,
            ),
            Expression::LogicalExpression(logical) => self.infix(
                &logical.left,
                logical.operator.as_str(),
                &logical.right,
                span,
                depth,
            ),
            Expression::UnaryExpression(unary) => {
                self.prefix(unary.operator.as_str(), &unary.argument, span, depth)
            }
            Expression::StaticMemberExpression(member) => self.static_member(member, span, depth),
            Expression::ComputedMemberExpression(member) => {
                self.computed_member(member, span, depth)
            }
            Expression::CallExpression(call) => self.call(call, span, depth),
            _ => Err(ExpressionRefusal::UnsupportedNode { span }),
        }
    }

    fn call(
        &mut self,
        call: &CallExpression<'a>,
        span: Span,
        depth: usize,
    ) -> Result<Doc<'a>, ExpressionRefusal> {
        if call.optional || call.type_arguments.is_some() {
            return Err(ExpressionRefusal::UnsupportedNode { span });
        }
        let callee = self.decoded_span(call.callee.span())?;
        let close = self.token(Span::new(span.end - 1, span.end), ")")?;
        if callee.start != span.start || callee.end >= close.start {
            return Err(ExpressionRefusal::InvalidFraming { span });
        }
        let first = call
            .arguments
            .first()
            .map(|argument| self.decoded_span(argument.span()))
            .transpose()?;
        let mut parts = Vec::new_in(&self.allocator);
        parts.push(self.node(&call.callee, span, depth + 1)?);
        let open = self.token(
            Span::new(
                callee.end,
                first.map_or(close.start, |argument| argument.start),
            ),
            "(",
        )?;
        parts.push(self.gap(Span::new(callee.end, open.start), Gap::Space)?);
        parts.push(self.text(open)?);
        let mut cursor = open.end;
        for (index, argument) in call.arguments.iter().enumerate() {
            let argument = argument
                .as_expression()
                .ok_or(ExpressionRefusal::UnsupportedNode { span })?;
            let argument_span = self.decoded_span(argument.span())?;
            if argument_span.start < cursor
                || argument_span.start >= argument_span.end
                || argument_span.end > close.start
            {
                return Err(ExpressionRefusal::InvalidFraming { span });
            }
            if index != 0 {
                let comma = self.token(Span::new(cursor, argument_span.start), ",")?;
                parts.push(self.gap(Span::new(cursor, comma.start), Gap::Empty)?);
                parts.push(self.text(comma)?);
                cursor = comma.end;
            }
            parts.push(self.gap(Span::new(cursor, argument_span.start), Gap::Space)?);
            parts.push(self.node(argument, span, depth + 1)?);
            cursor = argument_span.end;
        }
        // Call ASTs discard the trailing separator. Its checked source spelling
        // is retained, never inferred from the number of arguments.
        if !call.arguments.is_empty()
            && let Some(comma) = self.optional_token(Span::new(cursor, close.start), ",")?
        {
            parts.push(self.gap(Span::new(cursor, comma.start), Gap::Empty)?);
            parts.push(self.text(comma)?);
            cursor = comma.end;
        }
        parts.push(self.gap(Span::new(cursor, close.start), Gap::Space)?);
        parts.push(self.text(close)?);
        Ok(Doc::concat(parts))
    }

    fn static_member(
        &mut self,
        member: &StaticMemberExpression<'a>,
        span: Span,
        depth: usize,
    ) -> Result<Doc<'a>, ExpressionRefusal> {
        if member.optional {
            return Err(ExpressionRefusal::UnsupportedNode { span });
        }
        let object = self.decoded_span(member.object.span())?;
        let property = self.decoded_span(member.property.span())?;
        if object.start != span.start
            || property.end != span.end
            || object.end >= property.start
            || property.start >= property.end
        {
            return Err(ExpressionRefusal::InvalidFraming { span });
        }
        let mut parts = Vec::new_in(&self.allocator);
        parts.push(self.node(&member.object, span, depth + 1)?);
        let dot = self.token(Span::new(object.end, property.start), ".")?;
        // Keep numeric literal spellings separate from the member-access dot.
        parts.push(self.gap(Span::new(object.end, dot.start), Gap::Space)?);
        parts.push(self.text(dot)?);
        parts.push(self.gap(Span::new(dot.end, property.start), Gap::Space)?);
        parts.push(self.text(property)?);
        Ok(Doc::concat(parts))
    }

    fn computed_member(
        &mut self,
        member: &ComputedMemberExpression<'a>,
        span: Span,
        depth: usize,
    ) -> Result<Doc<'a>, ExpressionRefusal> {
        if member.optional {
            return Err(ExpressionRefusal::UnsupportedNode { span });
        }
        let object = self.decoded_span(member.object.span())?;
        let key = self.decoded_span(member.expression.span())?;
        if object.start != span.start
            || object.end >= key.start
            || key.start >= key.end
            || key.end >= span.end
        {
            return Err(ExpressionRefusal::InvalidFraming { span });
        }
        let mut parts = Vec::new_in(&self.allocator);
        parts.push(self.node(&member.object, span, depth + 1)?);
        let open = self.token(Span::new(object.end, key.start), "[")?;
        parts.push(self.gap(Span::new(object.end, open.start), Gap::Space)?);
        parts.push(self.text(open)?);
        parts.push(self.gap(Span::new(open.end, key.start), Gap::Space)?);
        parts.push(self.node(&member.expression, span, depth + 1)?);
        let close = self.token(Span::new(key.end, span.end), "]")?;
        if close.end != span.end {
            return Err(ExpressionRefusal::InvalidFraming { span });
        }
        parts.push(self.gap(Span::new(key.end, close.start), Gap::Space)?);
        parts.push(self.text(close)?);
        Ok(Doc::concat(parts))
    }

    fn prefix(
        &mut self,
        spelling: &str,
        argument: &Expression<'a>,
        span: Span,
        depth: usize,
    ) -> Result<Doc<'a>, ExpressionRefusal> {
        let argument_span = self.decoded_span(argument.span())?;
        if argument_span.start <= span.start || argument_span.end != span.end {
            return Err(ExpressionRefusal::InvalidFraming { span });
        }
        let operator = self.token(Span::new(span.start, argument_span.start), spelling)?;
        if operator.start != span.start {
            return Err(ExpressionRefusal::InvalidFraming { span });
        }
        let mut parts = Vec::new_in(&self.allocator);
        parts.push(self.text(operator)?);
        // A separator also keeps adjacent unary signs from becoming update tokens.
        parts.push(self.gap(Span::new(operator.end, argument_span.start), Gap::Space)?);
        parts.push(self.node(argument, span, depth + 1)?);
        Ok(Doc::concat(parts))
    }

    fn infix(
        &mut self,
        left: &Expression<'a>,
        spelling: &str,
        right: &Expression<'a>,
        span: Span,
        depth: usize,
    ) -> Result<Doc<'a>, ExpressionRefusal> {
        let left_span = self.decoded_span(left.span())?;
        let right_span = self.decoded_span(right.span())?;
        if left_span.start != span.start
            || right_span.end != span.end
            || left_span.end >= right_span.start
        {
            return Err(ExpressionRefusal::InvalidFraming { span });
        }
        let mut parts = Vec::new_in(&self.allocator);
        parts.push(self.node(left, span, depth + 1)?);
        let operator = self.token(Span::new(left_span.end, right_span.start), spelling)?;
        parts.push(self.gap(Span::new(left_span.end, operator.start), Gap::Space)?);
        parts.push(self.text(operator)?);
        let mut continuation = Vec::new_in(&self.allocator);
        continuation.push(self.gap(Span::new(operator.end, right_span.start), Gap::Break)?);
        continuation.push(self.node(right, span, depth + 1)?);
        parts.push(Doc::concat(continuation).indent(1, self.allocator));
        Ok(Doc::concat(parts).group(self.allocator))
    }
}
