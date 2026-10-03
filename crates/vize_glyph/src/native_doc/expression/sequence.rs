//! Every original sequence child and its exact intervening comma, in order.

use oxc_ast::ast::SequenceExpression;
use oxc_span::GetSpan;
use vize_l0::{Span, Vec};

use super::{
    Doc, ExpressionRefusal,
    source::{Context, Gap},
};

impl<'a> Context<'_, 'a> {
    pub(super) fn sequence(
        &mut self,
        sequence: &SequenceExpression<'a>,
        span: Span,
        depth: usize,
    ) -> Result<Doc<'a>, ExpressionRefusal> {
        let [first, .., last] = sequence.expressions.as_slice() else {
            return Err(ExpressionRefusal::InvalidFraming { span });
        };
        let first = self.decoded_span(first.span())?;
        let last = self.decoded_span(last.span())?;
        if first.start != span.start || last.end != span.end {
            return Err(ExpressionRefusal::InvalidFraming { span });
        }
        let mut parts = Vec::new_in(&self.allocator);
        let mut cursor = span.start;
        for (index, expression) in sequence.expressions.iter().enumerate() {
            let child = self.decoded_span(expression.span())?;
            if child.start < cursor || child.start >= child.end || child.end > span.end {
                return Err(ExpressionRefusal::InvalidFraming { span });
            }
            if index != 0 {
                let comma = self.token(Span::new(cursor, child.start), ",")?;
                parts.push(self.gap(Span::new(cursor, comma.start), Gap::Empty)?);
                parts.push(self.text(comma)?);
                parts.push(self.gap(Span::new(comma.end, child.start), Gap::Space)?);
            }
            parts.push(self.node(expression, span, depth + 1)?);
            cursor = child.end;
        }
        Ok(Doc::concat(parts).group(self.allocator))
    }
}
