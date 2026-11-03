//! A short original header selection reused by layout and conditional syntax.

use super::{
    NativeAttribute, NativeAttributeExpression, NativeAttributeExpressionFailure,
    NativeAttributeOperandError, NativeConditionKind, NativeTemplateComponent, Origin,
    observe_conditional,
};
use crate::dialect::vue3::operand::conditional_parts;
use crate::embed::SourceError;
use crate::markup::{DirectiveName, DirectiveSyntax, VueDirectives};
use vize_l0::{SourceBlock, SourceRoot};

/// The same actual selected/header borrow and its single dialect decomposition.
/// Copied parts are readonly metadata, not a caller-assembled observation proof.
/// This grants no File, body, target or complete formatter authority.
///
/// ```compile_fail
/// use vize_l1::markup::NativeAttributeHead;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<NativeAttributeHead<'static, 'static>>();
/// ```
///
/// ```compile_fail
/// use vize_l0::SourceBlock;
/// use vize_l1::markup::{NativeAttributeHead, NativeAttribute,
///     NativeTemplateComponent, DirectiveName, NativeConditionKind};
/// fn forge<'o, 'a>(selected: &'o NativeTemplateComponent<'a>,
///     attribute: NativeAttribute<'o, 'a>, name: SourceBlock<'a>,
///     directive: Option<DirectiveName>, condition: Option<NativeConditionKind>
/// ) -> NativeAttributeHead<'o, 'a> {
///     NativeAttributeHead { selected, attribute, name, directive, condition }
/// }
/// ```
///
/// ```compile_fail
/// use vize_l1::markup::{NativeAttributeHead, NativeTemplateComponent};
/// fn escape<'a>(selected: NativeTemplateComponent<'a>) -> NativeAttributeHead<'a, 'a> {
///     let element = selected.children().next().unwrap().into_element().unwrap();
///     selected.observe_attribute_head(element.attributes().next().unwrap()).unwrap()
/// }
/// ```
pub struct NativeAttributeHead<'o, 'a> {
    selected: &'o NativeTemplateComponent<'a>,
    attribute: NativeAttribute<'o, 'a>,
    name: SourceBlock<'a>,
    directive: Option<DirectiveName>,
    condition: Option<NativeConditionKind>,
}

impl<'a> NativeTemplateComponent<'a> {
    /// Select the actual current attribute once, without preparing its value.
    /// Original membership/recovery/verbatim errors precede dialect selection.
    pub fn observe_attribute_head<'o>(
        &'o self,
        attribute: NativeAttribute<'o, 'a>,
    ) -> Result<NativeAttributeHead<'o, 'a>, NativeAttributeOperandError> {
        Origin::check_original_header(self, &attribute)?;
        let block = self.component().block();
        let raw = attribute.surface().name.text;
        let offset = block
            .offset_of(raw)
            .ok_or(NativeAttributeOperandError::Source(
                SourceError::InvalidAuthoredSpan,
            ))?;
        // The selected block already validates its whole root. This smaller
        // frame checks the exact original token slice, without source rematching.
        let name = SourceRoot::new(block.root_source())
            .and_then(|root| root.block(raw, offset))
            .map_err(|_| NativeAttributeOperandError::Source(SourceError::InvalidAuthoredSpan))?;
        let directive = VueDirectives
            .decompose(raw, offset)
            .map_err(NativeAttributeOperandError::Directive)?;
        let condition = conditional_parts(directive, name.end(), name.root_source());
        Ok(NativeAttributeHead {
            selected: self,
            attribute,
            name,
            directive,
            condition,
        })
    }
}

impl<'o, 'a> NativeAttributeHead<'o, 'a> {
    #[must_use]
    pub fn selected(&self) -> &'o NativeTemplateComponent<'a> {
        self.selected
    }
    #[must_use]
    pub fn attribute(&self) -> &NativeAttribute<'o, 'a> {
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
    pub const fn condition_kind(&self) -> Option<NativeConditionKind> {
        self.condition
    }
    /// Prepare/parse the original conditional value through the shared tail.
    /// Reusing this head does not decompose or scan the name a second time.
    /// Receivers call this once at their event and retain the normal owner
    /// before checking admission. Separate public observations remain possible.
    pub fn observe_expression(
        &self,
    ) -> Result<NativeAttributeExpression<'a>, NativeAttributeExpressionFailure<'a>> {
        let kind = self
            .condition
            .ok_or(NativeAttributeOperandError::UnsupportedDirective)?;
        observe_conditional(self.selected, self.attribute.reborrow(), kind)
    }
}

#[cfg(test)]
mod tests;
