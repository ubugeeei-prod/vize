//! Distinct original model operand and same-event short admission.

use super::{NativeAttribute, NativeAttributeHead, NativeTemplateComponent, Origin};
use crate::embed::syntax::RetainedExpression;
use oxc_parser::AdmittedExpression;
use vize_l0::Span;

mod head;
pub use head::NativeModelHead;

/// Normally owned original model value, without conditional relabelling.
/// Its private Origin is derived only from the same selected original header.
/// A model operand by itself grants no write, File, body or target authority.
///
/// ```compile_fail
/// use vize_l1::markup::NativeAttributeModelExpression;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<NativeAttributeModelExpression<'static>>();
/// ```
///
/// ```compile_fail
/// use vize_l0::Span;
/// use vize_l1::markup::NativeAttributeModelExpression;
/// fn replace_value(mut owner: NativeAttributeModelExpression<'_>) {
///     owner.origin.value_span = Span::new(0, 0);
/// }
/// ```
pub struct NativeAttributeModelExpression<'a> {
    origin: Origin<'a>,
    syntax: RetainedExpression<'a>,
    trim: Option<Span>,
}

impl<'a> NativeAttributeModelExpression<'a> {
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
    /// Authored modifier name, retained only by the supported original head.
    #[must_use]
    pub const fn trim_span(&self) -> Option<Span> {
        self.trim
    }
    /// Join only with the actual component/header/ordinal/source/profile owner.
    /// Equal-byte copies and independently parsed siblings confer no authority.
    pub fn admitted_for<'s>(
        &'s self,
        selected: &'s NativeTemplateComponent<'a>,
        attribute: NativeAttribute<'s, 'a>,
    ) -> Option<NativeAttributeModelExpressionView<'s, 'a>> {
        if !self.origin.matches(selected, &attribute) {
            return None;
        }
        self.syntax.admitted_expression()?;
        Some(NativeAttributeModelExpressionView {
            owner: self,
            selected,
            attribute,
        })
    }
    /// Transfer the stock observation while discarding Model/header authority.
    #[must_use]
    pub fn into_syntax(self) -> RetainedExpression<'a> {
        self.syntax
    }
}

/// Short readonly join borrowed from the same private original Model owner.
///
/// ```compile_fail
/// use vize_l1::markup::{NativeAttribute, NativeAttributeModelExpression,
///     NativeAttributeModelExpressionView, NativeTemplateComponent};
/// fn forge<'s, 'a>(owner: &'s NativeAttributeModelExpression<'a>,
///     selected: &'s NativeTemplateComponent<'a>, attribute: NativeAttribute<'s, 'a>
/// ) -> NativeAttributeModelExpressionView<'s, 'a> {
///     NativeAttributeModelExpressionView { owner, selected, attribute }
/// }
/// ```
///
/// ```compile_fail
/// use vize_l1::markup::{NativeAttribute, NativeAttributeModelExpression,
///     NativeAttributeModelExpressionView, NativeTemplateComponent};
/// fn escape<'s, 'a>(owner: NativeAttributeModelExpression<'a>,
///     selected: &'s NativeTemplateComponent<'a>, attribute: NativeAttribute<'s, 'a>
/// ) -> NativeAttributeModelExpressionView<'s, 'a> {
///     owner.admitted_for(selected, attribute).unwrap()
/// }
/// ```
pub struct NativeAttributeModelExpressionView<'s, 'a> {
    owner: &'s NativeAttributeModelExpression<'a>,
    selected: &'s NativeTemplateComponent<'a>,
    attribute: NativeAttribute<'s, 'a>,
}

impl<'s, 'a> NativeAttributeModelExpressionView<'s, 'a> {
    #[must_use]
    pub fn selected(&self) -> &'s NativeTemplateComponent<'a> {
        self.selected
    }
    #[must_use]
    pub fn attribute(&self) -> &NativeAttribute<'s, 'a> {
        &self.attribute
    }
    #[must_use]
    pub fn operand(&self) -> &'s NativeAttributeModelExpression<'a> {
        self.owner
    }
    #[must_use]
    pub fn expression(&self) -> Option<AdmittedExpression<'s, 'a>> {
        self.owner.syntax.admitted_expression()
    }
}

#[cfg(test)]
mod tests;
