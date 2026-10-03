//! Original parser ownership and short readonly surface projections.
//! No projection grants Descriptor selection, File or native body completion.

use super::{ComponentParse, ComponentSourceError, parse_component};
use vize_l0::{Allocator, SourceBlock};

mod child;
mod element;
mod interpolation;
mod lint_tag;
mod operand;
mod selected;
pub use child::{NativeChild, NativeChildren};
pub use element::{NativeAttribute, NativeAttributes, NativeElement};
pub use interpolation::{
    NativeInterpolationError, NativeInterpolationFailure, NativeInterpolationOperand,
    NativeInterpolationView,
};
pub use lint_tag::{NativeLintTag, NativeLintTagKind, NativeLintTagRefusal};
pub use operand::{
    NativeAttributeExpression, NativeAttributeExpressionFailure, NativeAttributeExpressionView,
    NativeAttributeForHead, NativeAttributeForHeadFailure, NativeAttributeForHeadView,
    NativeAttributeHandler, NativeAttributeHandlerFailure, NativeAttributeHandlerView,
    NativeAttributeOperandError, NativeConditionKind,
};
pub use selected::{NativeScriptSelection, NativeTemplateComponent, NativeTemplateGrammar};

/// Complete once-parsed Component privately paired with its checked source.
/// Raw mutable carriers cannot be inserted; consuming transfer drops authority.
/// A genuinely parsed arbitrary block is not proof of SFC template selection.
///
/// ```compile_fail
/// use vize_l1::markup::NativeComponent;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<NativeComponent<'static>>();
/// ```
pub struct NativeComponent<'a> {
    allocator: &'a Allocator,
    block: SourceBlock<'a>,
    component: ComponentParse<'a>,
}

impl core::fmt::Debug for NativeComponent<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("NativeComponent")
            .field("block", &self.block)
            .field("component", &self.component)
            .finish_non_exhaustive()
    }
}

impl<'a> NativeComponent<'a> {
    /// Call the existing native parser once for this exact block.
    pub fn parse_in(
        allocator: &'a Allocator,
        block: SourceBlock<'a>,
    ) -> Result<Self, ComponentSourceError> {
        Ok(Self {
            allocator,
            block,
            component: parse_component(allocator, block.source())?,
        })
    }
    /// Retain original lint-header facts only for genuine Descriptor selection.
    pub(super) fn parse_selected_in(
        allocator: &'a Allocator,
        block: SourceBlock<'a>,
    ) -> Result<Self, ComponentSourceError> {
        Ok(Self {
            allocator,
            block,
            component: crate::dialect::vue3::surface::parse_component_with_lint(
                allocator,
                block.source(),
            )?,
        })
    }
    #[must_use]
    pub fn allocator(&self) -> &'a Allocator {
        self.allocator
    }
    #[must_use]
    pub fn block(&self) -> SourceBlock<'a> {
        self.block
    }
    #[must_use]
    pub fn carrier(&self) -> &ComponentParse<'a> {
        &self.component
    }
    /// Project actual root children in their original direct order.
    #[must_use]
    pub fn children(&self) -> NativeChildren<'_, 'a> {
        NativeChildren::root(self)
    }
    #[must_use]
    pub fn into_carrier(self) -> ComponentParse<'a> {
        self.component
    }
}

#[cfg(test)]
mod reborrow_tests;
#[cfg(test)]
mod tests;
