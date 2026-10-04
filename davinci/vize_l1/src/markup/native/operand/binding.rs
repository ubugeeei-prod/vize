//! Distinct original binding expression and same-event short admission.

use super::{NativeAttribute, NativeAttributeHead, NativeTemplateComponent, Origin};
use crate::embed::syntax::RetainedExpression;
use oxc_parser::AdmittedExpression;
use vize_l0::Span;

mod head;
pub use head::NativeStaticBindingHead;

/// Normally owned original value and argument, without conditional relabelling.
/// Its private Origin is derived only from the same selected original header.
/// This grants no File, target, quote policy or whole formatter authority.
///
/// ```compile_fail
/// use vize_l1::markup::NativeAttributeBindingExpression;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<NativeAttributeBindingExpression<'static>>();
/// ```
///
/// ```compile_fail
/// use vize_l0::Span;
/// use vize_l1::markup::NativeAttributeBindingExpression;
/// fn replace_argument(mut owner: NativeAttributeBindingExpression<'_>) {
///     owner.argument = Span::new(0, 0);
/// }
/// ```
pub struct NativeAttributeBindingExpression<'a> {
    origin: Origin<'a>,
    argument: Span,
    syntax: RetainedExpression<'a>,
}

impl<'a> NativeAttributeBindingExpression<'a> {
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
        selected: &'s NativeTemplateComponent<'a>,
        attribute: NativeAttribute<'s, 'a>,
    ) -> Option<NativeAttributeBindingExpressionView<'s, 'a>> {
        if !self.origin.matches(selected, &attribute) {
            return None;
        }
        self.syntax.admitted_expression()?;
        Some(NativeAttributeBindingExpressionView {
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
/// use vize_l1::markup::{NativeAttribute, NativeAttributeBindingExpression,
///     NativeAttributeBindingExpressionView, NativeTemplateComponent};
/// fn forge<'s, 'a>(owner: &'s NativeAttributeBindingExpression<'a>,
///     selected: &'s NativeTemplateComponent<'a>, attribute: NativeAttribute<'s, 'a>
/// ) -> NativeAttributeBindingExpressionView<'s, 'a> {
///     NativeAttributeBindingExpressionView { owner, selected, attribute }
/// }
/// ```
///
/// ```compile_fail
/// use vize_l1::markup::{NativeAttribute, NativeAttributeBindingExpression,
///     NativeAttributeBindingExpressionView, NativeTemplateComponent};
/// fn escape<'s, 'a>(owner: NativeAttributeBindingExpression<'a>,
///     selected: &'s NativeTemplateComponent<'a>, attribute: NativeAttribute<'s, 'a>
/// ) -> NativeAttributeBindingExpressionView<'s, 'a> {
///     owner.admitted_for(selected, attribute).unwrap()
/// }
/// ```
pub struct NativeAttributeBindingExpressionView<'s, 'a> {
    owner: &'s NativeAttributeBindingExpression<'a>,
    selected: &'s NativeTemplateComponent<'a>,
    attribute: NativeAttribute<'s, 'a>,
}

impl<'s, 'a> NativeAttributeBindingExpressionView<'s, 'a> {
    #[must_use]
    pub fn selected(&self) -> &'s NativeTemplateComponent<'a> {
        self.selected
    }
    #[must_use]
    pub fn attribute(&self) -> &NativeAttribute<'s, 'a> {
        &self.attribute
    }
    #[must_use]
    pub fn operand(&self) -> &'s NativeAttributeBindingExpression<'a> {
        self.owner
    }
    #[must_use]
    pub fn expression(&self) -> Option<AdmittedExpression<'s, 'a>> {
        self.owner.syntax.admitted_expression()
    }
}

#[cfg(test)]
mod tests;
