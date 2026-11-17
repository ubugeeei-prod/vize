//! Distinct original binding expression and same-event short admission.

use super::{AttributeHead, AttributeView, Origin, TemplateComponent};
use crate::embed::syntax::RetainedExpression;
use oxc_parser::AdmittedExpression;
use vize_l0::Span;

mod head;
pub use head::StaticBindingHead;

/// Normally owned original value and argument, without conditional relabelling.
/// Its private Origin is derived only from the same selected original header.
/// This grants no File, target, quote policy or whole formatter authority.
///
/// ```compile_fail
/// use vize_l1::markup::AttributeBindingExpression;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<AttributeBindingExpression<'static>>();
/// ```
///
/// ```compile_fail
/// use vize_l0::Span;
/// use vize_l1::markup::AttributeBindingExpression;
/// fn replace_argument(mut owner: AttributeBindingExpression<'_>) {
///     owner.argument = Span::new(0, 0);
/// }
/// ```
pub struct AttributeBindingExpression<'a> {
    origin: Origin<'a>,
    argument: Span,
    syntax: RetainedExpression<'a>,
}

impl<'a> AttributeBindingExpression<'a> {
    /// Complete retained syntax also remains available for local syntax holes.
    #[must_use]
    pub fn syntax(&self) -> &RetainedExpression<'a> {
        &self.syntax
    }
    #[must_use]
    pub const fn raw_value(&self) -> &'a str {
        self.origin.raw_value
    }
    #[must_use]
    pub const fn value_span(&self) -> Span {
        self.origin.value_span
    }
    #[must_use]
    pub const fn name_span(&self) -> Span {
        self.origin.name_span
    }
    #[must_use]
    pub const fn argument_span(&self) -> Span {
        self.argument
    }
    /// Join only with the actual component/header/ordinal/source/profile owner.
    /// Equal-byte copies and independently parsed siblings confer no authority.
    pub fn admitted_for<'s>(
        &'s self,
        selected: &'s TemplateComponent<'a>,
        attribute: AttributeView<'s, 'a>,
    ) -> Option<AttributeBindingExpressionView<'s, 'a>> {
        if !self.origin.matches(selected, &attribute) {
            return None;
        }
        self.syntax.admitted_expression()?;
        Some(AttributeBindingExpressionView {
            owner: self,
            selected,
            attribute,
        })
    }
    /// Transfer the stock observation while discarding binding/header authority.
    #[must_use]
    pub fn into_syntax(self) -> RetainedExpression<'a> {
        self.syntax
    }
}

/// Short readonly join borrowed from the same private original binding owner.
///
/// ```compile_fail
/// use vize_l1::markup::{AttributeView, AttributeBindingExpression,
///     AttributeBindingExpressionView, TemplateComponent};
/// fn forge<'s, 'a>(owner: &'s AttributeBindingExpression<'a>,
///     selected: &'s TemplateComponent<'a>, attribute: AttributeView<'s, 'a>
/// ) -> AttributeBindingExpressionView<'s, 'a> {
///     AttributeBindingExpressionView { owner, selected, attribute }
/// }
/// ```
///
/// ```compile_fail
/// use vize_l1::markup::{AttributeView, AttributeBindingExpression,
///     AttributeBindingExpressionView, TemplateComponent};
/// fn escape<'s, 'a>(owner: AttributeBindingExpression<'a>,
///     selected: &'s TemplateComponent<'a>, attribute: AttributeView<'s, 'a>
/// ) -> AttributeBindingExpressionView<'s, 'a> {
///     owner.admitted_for(selected, attribute).unwrap()
/// }
/// ```
pub struct AttributeBindingExpressionView<'s, 'a> {
    owner: &'s AttributeBindingExpression<'a>,
    selected: &'s TemplateComponent<'a>,
    attribute: AttributeView<'s, 'a>,
}

impl<'s, 'a> AttributeBindingExpressionView<'s, 'a> {
    #[must_use]
    pub fn selected(&self) -> &'s TemplateComponent<'a> {
        self.selected
    }
    #[must_use]
    pub fn attribute(&self) -> &AttributeView<'s, 'a> {
        &self.attribute
    }
    #[must_use]
    pub fn operand(&self) -> &'s AttributeBindingExpression<'a> {
        self.owner
    }
    #[must_use]
    pub fn expression(&self) -> Option<AdmittedExpression<'s, 'a>> {
        self.owner.syntax.admitted_expression()
    }
}

#[cfg(test)]
mod tests;
