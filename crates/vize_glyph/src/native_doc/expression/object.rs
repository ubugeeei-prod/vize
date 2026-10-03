//! Ordered original explicit static data properties, without key rewriting.

use oxc_ast::ast::{ObjectExpression, ObjectPropertyKind, PropertyKey, PropertyKind};
use oxc_span::GetSpan;
use vize_l0::{Span, Vec};

use super::{
    Doc, ExpressionRefusal, MAX_EXPRESSION_DOCUMENT_DEPTH,
    source::{Context, Gap},
};

impl<'a> Context<'_, 'a> {
    pub(super) fn object(
        &mut self,
        object: &ObjectExpression<'a>,
        span: Span,
        depth: usize,
    ) -> Result<Doc<'a>, ExpressionRefusal> {
        let open = self.token(Span::new(span.start, span.start + 1), "{")?;
        let close = self.token(Span::new(span.end - 1, span.end), "}")?;
        let mut parts = Vec::new_in(&self.allocator);
        parts.push(self.text(open)?);
        let mut cursor = open.end;
        for (index, property) in object.properties.iter().enumerate() {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return Err(ExpressionRefusal::UnsupportedNode { span });
            };
            let property_span = self.decoded_span(property.span)?;
            if property.kind != PropertyKind::Init
                || property.method
                || property.shorthand
                || property.computed
            {
                return Err(ExpressionRefusal::UnsupportedNode {
                    span: property_span,
                });
            }
            match &property.key {
                PropertyKey::StaticIdentifier(key) if key.name.as_str() != "__proto__" => {}
                PropertyKey::StringLiteral(key) if key.value.as_str() != "__proto__" => {}
                PropertyKey::NumericLiteral(_) => {}
                _ => {
                    return Err(ExpressionRefusal::UnsupportedNode {
                        span: property_span,
                    });
                }
            }
            let key = self.decoded_span(property.key.span())?;
            let value = self.decoded_span(property.value.span())?;
            if property_span.start < cursor
                || property_span.start != key.start
                || key.start >= key.end
                || key.end >= value.start
                || value.start >= value.end
                || value.end != property_span.end
                || property_span.end > close.start
            {
                return Err(ExpressionRefusal::InvalidFraming {
                    span: property_span,
                });
            }
            if index != 0 {
                let comma = self.token(Span::new(cursor, property_span.start), ",")?;
                parts.push(self.gap(Span::new(cursor, comma.start), Gap::Empty)?);
                parts.push(self.text(comma)?);
                cursor = comma.end;
            }
            parts.push(self.gap(Span::new(cursor, key.start), Gap::Space)?);
            // The genuine static key is a child, without a surrogate Expr or
            // an additional ObjectProperty recursion level.
            if depth >= MAX_EXPRESSION_DOCUMENT_DEPTH {
                return Err(ExpressionRefusal::DepthLimit { span: key });
            }
            parts.push(self.text(key)?);
            let colon = self.token(Span::new(key.end, value.start), ":")?;
            parts.push(self.gap(Span::new(key.end, colon.start), Gap::Empty)?);
            parts.push(self.text(colon)?);
            parts.push(self.gap(Span::new(colon.end, value.start), Gap::Space)?);
            parts.push(self.node(&property.value, property_span, depth + 1)?);
            cursor = property_span.end;
        }
        if let Some(comma) = self.optional_token(Span::new(cursor, close.start), ",")? {
            if object.properties.is_empty() {
                return Err(ExpressionRefusal::InvalidFraming { span });
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
