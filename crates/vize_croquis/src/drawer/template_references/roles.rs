use super::{
    Capture, Local, TemplateReference, TemplateReferenceGapKind, TemplateReferenceOwner,
    source_span,
};
use crate::drawer::helpers::{
    EventHandlerExpression, RetainedIdentifierRef, RetainedParameterRefs,
    classify_retained_event_handler, retained_expression_refs, retained_handler_refs,
    retained_parameter_refs,
};
use vize_armature::parser::{TemplateSyntax, TemplateSyntaxRecord};
use vize_l0::Span;
use vize_relief::{ExpressionNode, JsExpression};

impl<'facts, 'source> Capture<'facts, 'source> {
    pub(super) fn record(
        &self,
        expression: &ExpressionNode<'_>,
    ) -> Option<&'facts TemplateSyntaxRecord<'source>> {
        let ExpressionNode::Simple(node) = expression else {
            return None;
        };
        self.syntax.syntax_for(
            self.source,
            oxc_span::Span::new(node.loc.span.start, node.loc.span.end),
            node.content,
        )
    }

    pub(super) fn expression(&mut self, expression: &ExpressionNode<'_>) {
        let Some(record) = self.record(expression) else {
            self.gap(
                expression.loc().span,
                TemplateReferenceGapKind::MissingSyntax,
            );
            return;
        };
        let (references, handler) = match record.syntax() {
            TemplateSyntax::Expression(Some(expression)) => {
                (retained_expression_refs(expression), false)
            }
            TemplateSyntax::HandlerExpression(expression) => {
                let references = retained_expression_refs(expression);
                // The shared legacy classifier also collects callback binding
                // names. Only enter it after the complete bounded token walk
                // admits the expression, including every parameter pattern.
                let handler = references.is_some()
                    && matches!(
                        classify_retained_event_handler(expression),
                        EventHandlerExpression::Inline
                    );
                (references, handler)
            }
            TemplateSyntax::Handler(owner) => (owner.body().and_then(retained_handler_refs), true),
            _ => (None, false),
        };
        let Some(references) = references else {
            self.gap(
                source_span(record.span()),
                TemplateReferenceGapKind::RefusedSyntax,
            );
            return;
        };
        self.append(
            references,
            |span| record.authored_span(span),
            record.span(),
            handler,
        );
    }

    pub(super) fn slot_scope(&mut self, record: &TemplateSyntaxRecord<'_>) {
        let references = match record.syntax() {
            TemplateSyntax::Slot(Some(retained)) => retained
                .as_slot_parameters()
                .and_then(Result::ok)
                .and_then(retained_parameter_refs),
            TemplateSyntax::LegacySlot(owner) => owner
                .admitted_parameters()
                .and_then(|admitted| retained_parameter_refs(admitted.parameters())),
            _ => None,
        };
        let Some(references) = references else {
            self.gap(
                source_span(record.span()),
                TemplateReferenceGapKind::RefusedSyntax,
            );
            self.unknown_scope = true;
            return;
        };
        if !self.parameters(references, |span| record.authored_span(span), record.span()) {
            self.unknown_scope = true;
        }
    }

    pub(super) fn for_scope(&mut self, record: &TemplateSyntaxRecord<'_>) {
        let TemplateSyntax::For(head) = record.syntax() else {
            self.gap(
                source_span(record.span()),
                TemplateReferenceGapKind::RefusedSyntax,
            );
            self.unknown_scope = true;
            return;
        };
        let admitted = (|| {
            if head.hole().is_some() {
                return None;
            }
            let collection = head.collection()?.ok()?;
            let aliases = head.aliases()?.ok()?;
            let references = retained_expression_refs(&JsExpression {
                ast: collection.expression()?,
                raw: collection.source().text(),
            })?;
            let parameters = retained_parameter_refs(aliases.admitted_parameters()?.parameters())?;
            Some((collection, aliases, references, parameters))
        })();
        let Some((collection, aliases, references, parameters)) = admitted else {
            self.gap(
                source_span(record.span()),
                TemplateReferenceGapKind::RefusedSyntax,
            );
            self.unknown_scope = true;
            return;
        };
        // Collection reads are resolved in the incoming scope, before any
        // aliases or their parameter defaults can introduce child bindings.
        let old_references = self.packet.references.len();
        if !self.append(
            references,
            |span| collection.authored_span(span).ok(),
            record.span(),
            false,
        ) || !self.parameters(
            parameters,
            |span| aliases.authored_span(span).ok(),
            record.span(),
        ) {
            self.packet.references.truncate(old_references);
            self.unknown_scope = true;
        }
    }

    pub(super) fn is_original_for_collection(
        &self,
        record: &TemplateSyntaxRecord<'_>,
        expression: &ExpressionNode<'_>,
    ) -> bool {
        let (TemplateSyntax::For(head), ExpressionNode::Simple(expression)) =
            (record.syntax(), expression)
        else {
            return false;
        };
        let Some(Ok(collection)) = head.collection() else {
            return false;
        };
        let span = collection.source().span();
        span.start == expression.loc.span.start
            && span.end == expression.loc.span.end
            && collection.source().text() == expression.content
    }

    fn parameters(
        &mut self,
        parameters: RetainedParameterRefs,
        project: impl Fn(oxc_span::Span) -> Option<Span>,
        span: oxc_span::Span,
    ) -> bool {
        let bindings: Option<Vec<_>> = parameters
            .bindings
            .into_iter()
            .map(|binding| {
                Some(Local {
                    name: binding.name,
                    span: project(binding.span)?,
                })
            })
            .collect();
        let Some(bindings) = bindings else {
            self.gap(
                source_span(span),
                TemplateReferenceGapKind::InvalidCoordinates,
            );
            return false;
        };
        if !self.spend(bindings.len(), source_span(span))
            || !self.append(parameters.references, project, span, false)
        {
            return false;
        }
        self.locals.extend(bindings);
        true
    }

    fn append(
        &mut self,
        references: Vec<RetainedIdentifierRef>,
        project: impl Fn(oxc_span::Span) -> Option<Span>,
        span: oxc_span::Span,
        handler: bool,
    ) -> bool {
        // Validate the complete goal before publishing any of its references.
        let references: Option<Vec<_>> = references
            .into_iter()
            .map(|reference| {
                let span = project(reference.span)?;
                self.source.get(span.start as usize..span.end as usize)?;
                let owner = if handler && reference.name == "$event" {
                    TemplateReferenceOwner::TemplateLocal { declaration: None }
                } else if self.unknown_scope {
                    TemplateReferenceOwner::Unknown
                } else if let Some(local) = self
                    .locals
                    .iter()
                    .rev()
                    .find(|local| local.name == reference.name)
                {
                    TemplateReferenceOwner::TemplateLocal {
                        declaration: Some(local.span),
                    }
                } else {
                    TemplateReferenceOwner::ComponentContext
                };
                Some(TemplateReference {
                    name: reference.name,
                    span,
                    owner,
                })
            })
            .collect();
        let Some(references) = references else {
            self.gap(
                source_span(span),
                TemplateReferenceGapKind::InvalidCoordinates,
            );
            return false;
        };
        if !self.spend(references.len(), source_span(span)) {
            return false;
        }
        self.packet.references.extend(references);
        true
    }
}
