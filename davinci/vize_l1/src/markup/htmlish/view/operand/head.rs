//! A short original header selection reused by layout and conditional syntax.

use super::{
    AttributeExpression, AttributeExpressionFailure, AttributeOperandError, AttributeView,
    ConditionKind, Origin, SelectedTemplateComponent, observe_conditional,
};
use crate::embed::SourceError;
use crate::framework::vue::version::v3::operand::conditional_parts;
use crate::markup::{DirectiveName, DirectiveSyntax, VueDirectives};
use vize_l0::{SourceBlock, SourceRoot};

/// The same actual selected/header borrow and its single dialect decomposition.
/// Copied parts are readonly metadata, not a caller-assembled observation proof.
/// This grants no File, body, target or complete formatter authority.
///
/// ```compile_fail
/// use vize_l1::markup::AttributeHead;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<AttributeHead<'static, 'static>>();
/// ```
///
/// ```compile_fail
/// use vize_l0::SourceBlock;
/// use vize_l1::markup::{AttributeHead, AttributeView,
///     SelectedTemplateComponent, DirectiveName, ConditionKind};
/// fn forge<'o, 'a>(selected: &'o SelectedTemplateComponent<'a>,
///     attribute: AttributeView<'o, 'a>, name: SourceBlock<'a>,
///     directive: Option<DirectiveName>, condition: Option<ConditionKind>
/// ) -> AttributeHead<'o, 'a> {
///     AttributeHead { selected, attribute, name, directive, condition }
/// }
/// ```
///
/// ```compile_fail
/// use vize_l1::markup::{AttributeHead, SelectedTemplateComponent};
/// fn escape<'a>(selected: SelectedTemplateComponent<'a>) -> AttributeHead<'a, 'a> {
///     let element = selected.children().next().unwrap().into_element().unwrap();
///     selected.observe_attribute_head(element.attributes().next().unwrap()).unwrap()
/// }
/// ```
pub struct AttributeHead<'o, 'a> {
    selected: &'o SelectedTemplateComponent<'a>,
    attribute: AttributeView<'o, 'a>,
    name: SourceBlock<'a>,
    directive: Option<DirectiveName>,
    condition: Option<ConditionKind>,
}

impl<'a> SelectedTemplateComponent<'a> {
    /// Select the actual current attribute once, without preparing its value.
    /// Original membership/recovery/verbatim errors precede dialect selection.
    pub fn observe_attribute_head<'o>(
        &'o self,
        attribute: AttributeView<'o, 'a>,
    ) -> Result<AttributeHead<'o, 'a>, AttributeOperandError> {
        Origin::check_original_header(self, &attribute)?;
        let block = self.component().block();
        let raw = attribute.surface().name.text;
        let offset = block.offset_of(raw).ok_or(AttributeOperandError::Source(
            SourceError::InvalidAuthoredSpan,
        ))?;
        // The selected block already validates its whole root. This smaller
        // frame checks the exact original token slice, without source rematching.
        let name = SourceRoot::new(block.root_source())
            .and_then(|root| root.block(raw, offset))
            .map_err(|_| AttributeOperandError::Source(SourceError::InvalidAuthoredSpan))?;
        let directive = VueDirectives
            .decompose(raw, offset)
            .map_err(AttributeOperandError::Directive)?;
        let condition = conditional_parts(directive, name.end(), name.root_source());
        Ok(AttributeHead {
            selected: self,
            attribute,
            name,
            directive,
            condition,
        })
    }
}

impl<'o, 'a> AttributeHead<'o, 'a> {
    #[must_use]
    pub fn selected(&self) -> &'o SelectedTemplateComponent<'a> {
        self.selected
    }
    #[must_use]
    pub fn attribute(&self) -> &AttributeView<'o, 'a> {
        &self.attribute
    }
    /// File-absolute original name window; not a template-local error offset.
    #[must_use]
    pub const fn name_block(&self) -> SourceBlock<'a> {
        self.name
    }
    #[must_use]
    pub const fn directive(&self) -> Option<DirectiveName> {
        self.directive
    }
    #[must_use]
    pub const fn condition_kind(&self) -> Option<ConditionKind> {
        self.condition
    }
    /// Prepare/parse the original conditional value through the shared tail.
    /// Reusing this head does not decompose or scan the name a second time.
    /// Receivers call this once at their event and retain the normal owner
    /// before checking admission. Separate public observations remain possible.
    pub fn observe_expression(
        &self,
    ) -> Result<AttributeExpression<'a>, AttributeExpressionFailure<'a>> {
        let kind = self
            .condition
            .ok_or(AttributeOperandError::UnsupportedDirective)?;
        observe_conditional(self.selected, self.attribute.reborrow(), kind)
    }
}

#[cfg(test)]
mod tests;
