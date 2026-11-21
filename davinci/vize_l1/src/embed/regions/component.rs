//! Original tokens used when a component call is checked in the host language.
use alloc::vec::Vec;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeValue, JSXElementName, JSXOpeningElement};
use oxc_span::GetSpan;
use vize_l0::Span;

/// Borrowed through the same module surface that owns the original parse.
/// Only simple callable names without explicit type arguments have entries.
#[derive(Default, Debug)]
pub struct ComponentSites {
    callees: Vec<(u32, Span)>,
    literals: Vec<(Span, Span)>,
}

impl ComponentSites {
    pub(super) fn record(&mut self, opening: &JSXOpeningElement<'_>) {
        if opening.type_arguments.is_some()
            || !matches!(
                opening.name,
                JSXElementName::IdentifierReference(_) | JSXElementName::MemberExpression(_)
            )
        {
            return;
        }
        let name = opening.name.span();
        self.callees
            .push((opening.span.start, Span::new(name.start, name.end)));
        for item in &opening.attributes {
            if let JSXAttributeItem::Attribute(attr) = item
                && let Some(JSXAttributeValue::StringLiteral(value)) = &attr.value
            {
                self.literals.push((
                    Span::new(attr.span.start, attr.span.end),
                    Span::new(value.span.start, value.span.end),
                ));
            }
        }
    }

    pub(super) fn finish(mut self) -> Self {
        self.callees.sort_unstable_by_key(|row| row.0);
        // Nested values can put a child between two attributes of its parent.
        self.literals.sort_unstable_by_key(|row| row.0.start);
        self
    }

    /// Exact callee of the opening at this source position.
    #[must_use]
    pub fn callee(&self, opening: u32) -> Option<Span> {
        let i = self
            .callees
            .binary_search_by_key(&opening, |row| row.0)
            .ok()?;
        Some(self.callees[i].1)
    }

    /// The complete literal token, including its quotes, for this attribute.
    #[must_use]
    pub fn literal(&self, attribute: Span) -> Option<Span> {
        let i = self
            .literals
            .binary_search_by_key(&attribute.start, |row| row.0.start)
            .ok()?;
        let (owner, token) = self.literals[i];
        (owner == attribute).then_some(token)
    }
}
