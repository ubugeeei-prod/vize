//! Original array elements and genuine comma-token elisions, consumed once.

use oxc_ast::ast::{ArrayExpression, ArrayExpressionElement};
use oxc_span::GetSpan;
use vize_l0::{Span, Vec};

use super::{
    Doc, ExpressionRefusal, MAX_EXPRESSION_DOCUMENT_DEPTH,
    source::{Context, Gap},
};

impl<'a> Context<'_, 'a> {
    pub(super) fn array(
        &mut self,
        array: &ArrayExpression<'a>,
        span: Span,
        depth: usize,
    ) -> Result<Doc<'a>, ExpressionRefusal> {
        let open = self.token(Span::new(span.start, span.start + 1), "[")?;
        let close = self.token(Span::new(span.end - 1, span.end), "]")?;
        let mut parts = Vec::new_in(&self.allocator);
        parts.push(self.text(open)?);
        let mut cursor = open.end;
        let mut preceding_expression = false;
        for element in &array.elements {
            let element_span = self.decoded_span(element.span())?;
            if element_span.start < cursor
                || element_span.start >= element_span.end
                || element_span.end > close.start
            {
                return Err(ExpressionRefusal::InvalidFraming { span });
            }
            if preceding_expression {
                let comma = self.token(Span::new(cursor, element_span.start), ",")?;
                parts.push(self.gap(Span::new(cursor, comma.start), Gap::Empty)?);
                parts.push(self.text(comma)?);
                cursor = comma.end;
            }
            parts.push(self.gap(Span::new(cursor, element_span.start), Gap::Space)?);
            if let ArrayExpressionElement::Elision(_) = element {
                // The parser's list consumes this same comma after retaining
                // its Elision node. It is emitted once, never counted twice.
                if depth >= MAX_EXPRESSION_DOCUMENT_DEPTH {
                    return Err(ExpressionRefusal::DepthLimit { span: element_span });
                }
                let comma = self.token(element_span, ",")?;
                if comma != element_span {
                    return Err(ExpressionRefusal::InvalidFraming { span: element_span });
                }
                parts.push(self.text(comma)?);
                preceding_expression = false;
            } else {
                let expression = element
                    .as_expression()
                    .ok_or(ExpressionRefusal::UnsupportedNode { span })?;
                parts.push(self.node(expression, span, depth + 1)?);
                preceding_expression = true;
            }
            cursor = element_span.end;
        }
        // A final expression may have a real trailing separator, which is not
        // retained in ArrayExpression. A final Elision already owns its comma.
        let suffix = Span::new(cursor, close.start);
        if let Some(comma) = self.optional_token(suffix, ",")? {
            if !preceding_expression {
                return Err(ExpressionRefusal::InvalidFraming { span: suffix });
            }
            parts.push(self.gap(Span::new(cursor, comma.start), Gap::Empty)?);
            parts.push(self.text(comma)?);
            cursor = comma.end;
        }
        parts.push(self.gap(Span::new(cursor, close.start), Gap::Space)?);
        parts.push(self.text(close)?);
        Ok(Doc::concat(parts).group(self.allocator))
    }
}
