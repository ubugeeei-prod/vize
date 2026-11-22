//! Authored attributes and bindings on the same resolved component operation.
use super::ImportedComponentSite;
use crate::op::{BindingOp, DynamicName};
use vize_l0::{SourceRoot, Span};

/// A prop's cooked value and the original spelling that supplied it. A bare
/// attribute has no value. A bound prop also carries the expression's extent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImportedComponentProp<'a> {
    pub name: &'a str,
    pub value: Option<&'a str>,
    pub span: Span,
    pub name_span: Span,
    pub expression: Option<Span>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentPropIssue {
    OriginalSource,
    Binding(Span),
}

impl<'a> ImportedComponentSite<'_, 'a> {
    /// Merge the component's semantic attribute and binding groups in authored
    /// order. Returns a refusal when a binding needs another prop contract.
    /// A late refusal invalidates all earlier rows of this request.
    pub fn visit_passed_props(
        &self,
        mut visit: impl FnMut(ImportedComponentProp<'a>),
    ) -> Result<(), ComponentPropIssue> {
        let owner = self.component;
        let source = SourceRoot::new(self.binding.file().artifact().source())
            .map_err(|_| ComponentPropIssue::OriginalSource)?
            .whole_block();
        let mut attributes = owner.attributes.iter().peekable();
        let mut bindings = owner.bindings.iter().peekable();
        let mut previous = self.name.end;
        loop {
            let binding = match bindings.peek() {
                Some(BindingOp::Bind(bind)) => Some(bind.as_ref()),
                Some(_) => return Err(ComponentPropIssue::Binding(owner.span)),
                None => None,
            };
            let (name, value, span, expression) = match (attributes.peek(), binding) {
                (Some(attribute), bind)
                    if bind.is_none_or(|b| attribute.span.start < b.span.start) =>
                {
                    let row = (attribute.name, attribute.value, attribute.span, None);
                    attributes.next();
                    row
                }
                (_, Some(bind)) => {
                    let (Some(DynamicName::Static(name)), Some(value)) = (bind.name, bind.value)
                    else {
                        return Err(ComponentPropIssue::Binding(bind.span));
                    };
                    if !bind.modifiers.is_empty() {
                        return Err(ComponentPropIssue::Binding(bind.span));
                    }
                    let row = (name, Some(value.source()), bind.span, Some(value.span()));
                    bindings.next();
                    row
                }
                (None, None) => break,
                _ => return Err(ComponentPropIssue::OriginalSource),
            };
            let name_span = source
                .span_of(name)
                .ok_or(ComponentPropIssue::OriginalSource)?;
            if span.start < previous
                || span.end > owner.span.end
                || name_span.start < span.start
                || name_span.end > span.end
                || !source.contains_block_span(span)
                || expression.is_some_and(|e| e.start < span.start || e.end > span.end)
            {
                return Err(ComponentPropIssue::OriginalSource);
            }
            visit(ImportedComponentProp {
                name,
                value,
                span,
                name_span,
                expression,
            });
            previous = span.end;
        }
        Ok(())
    }
}
