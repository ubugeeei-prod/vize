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
    /// The exact setup Program that supplied this import, when its complete
    /// statement walk saw no ECMAScript module exports.
    #[must_use]
    pub fn nonexporting_setup_span(&self) -> Option<Span> {
        let [unit] = self.binding.file().units() else {
            return None;
        };
        (unit.walk_completed() && !unit.origin.has_export).then_some(unit.span)
    }

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
                    let spelling = original_operand(source.source(), value, bind.span)?;
                    let row = (name, Some(spelling), bind.span, Some(value.span()));
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

fn original_operand<'a>(
    source: &'a str,
    value: crate::expr::ExprRef<'_>,
    binding: Span,
) -> Result<&'a str, ComponentPropIssue> {
    let extent = value.span();
    let spelling = source
        .get(extent.start as usize..extent.end as usize)
        .ok_or(ComponentPropIssue::OriginalSource)?;
    // A same-name shorthand can synthesize a camelized expression at the
    // kebab-case argument's span. Only publish bytes from the original file.
    if !core::ptr::eq(spelling, value.source()) && spelling != value.source() {
        return Err(ComponentPropIssue::Binding(binding));
    }
    Ok(spelling)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::ExprRef;
    use vize_l0::Allocator;

    #[test]
    fn original_operand_checks_synthesized_and_explicit_bytes() {
        let arena = Allocator::new();
        let source = ":user-name='userName'";
        let binding = Span::new(0, 10);
        let synthesized = ExprRef::parse_js_in(&arena, "userName", Span::new(1, 10));
        assert_eq!(
            original_operand(source, synthesized, binding),
            Err(ComponentPropIssue::Binding(binding))
        );
        let valid_extent = Span::new(12, 20);
        for text in ["userName", valid_extent.slice(source)] {
            let explicit = ExprRef::parse_js_in(&arena, text, valid_extent);
            assert!(core::ptr::eq(
                original_operand(source, explicit, Span::new(0, 21)).unwrap(),
                valid_extent.slice(source)
            ));
        }
        let invalid = ExprRef::parse_js_in(&arena, "x", Span::new(20, 22));
        assert_eq!(
            original_operand(source, invalid, binding),
            Err(ComponentPropIssue::OriginalSource)
        );
    }
}
