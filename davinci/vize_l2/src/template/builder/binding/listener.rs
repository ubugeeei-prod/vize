//! Check and attach an authored static event before its owner's children.
use super::{BindingOp, Box, DynamicName, ExprRef, L2TemplateError, check};
use crate::expr::JsExpr;
use crate::op::{OnHandlerRef, OnOp};
use crate::template::RegionBuilder;
use vize_l0::{SourceRoot, Span, id::NodeId};

impl<'a> RegionBuilder<'_, 'a> {
    pub(crate) fn attach_event_reference(
        &mut self,
        name_span: Span,
        modifiers: vize_l0::Vec<'a, &'a str>,
        handler: Option<&'a JsExpr<'a>>,
        span: Span,
    ) -> Result<NodeId, L2TemplateError> {
        let builder = &mut self.builder;
        let id = builder.prepare_binding(span)?;
        let source = builder.parts.source;
        check::owned_span(source, id, name_span, Some(span))?;
        let name = name_span.slice(source);
        let invalid_name = L2TemplateError::InvalidBindingName {
            node: id,
            span: name_span,
        };
        if name.is_empty() {
            return Err(invalid_name);
        }
        let root = SourceRoot::new(source)
            .map_err(|_| L2TemplateError::SourceLimit)?
            .whole_block();
        let mut previous_end = name_span.end;
        for &modifier in &modifiers {
            let range = root
                .span_of(modifier)
                .filter(|r| r.start >= previous_end && r.end <= span.end && r.start < r.end)
                .ok_or(invalid_name)?;
            previous_end = range.end;
        }
        if let Some(handler) = handler {
            check::expression(source, id, ExprRef::Js(handler), span)?;
            if handler.span.start < previous_end
                || handler.span.slice(source) != handler.source
                || !matches!(handler.ast, oxc_ast::ast::Expression::Identifier(name) if name.name.as_str() == handler.source)
            {
                return Err(L2TemplateError::InvalidJsSpan {
                    node: id,
                    span: handler.span,
                });
            }
        }
        let frame = builder
            .frames
            .last_mut()
            .ok_or(L2TemplateError::BindingWithoutOwner { span })?;
        let _ = builder.walk.mint_attached();
        frame.bindings.push(BindingOp::On(Box::new_in(
            OnOp {
                name: Some(DynamicName::Static(name)),
                modifiers,
                handler: handler.map(OnHandlerRef::Js),
                span,
            },
            &builder.allocator,
        )));
        Ok(id)
    }
}
