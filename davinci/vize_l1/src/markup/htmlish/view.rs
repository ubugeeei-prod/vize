//! Original parser ownership and short readonly surface projections.
//! No projection grants Descriptor selection, File or native body completion.

use crate::markup::{ComponentParse, ComponentSourceError, parse_component};
use vize_l0::{Allocator, SourceBlock};

mod child;
mod element;
mod interpolation;
mod operand;
mod selected;
mod text;
mod text_value;
pub use child::{ChildView, ChildrenView};
pub use element::{
    AttributeView, AttributesView, ElementClosingName, ElementNameRefusal, ElementNames,
    ElementView,
};
pub use interpolation::{
    InterpolationError, InterpolationFailure, InterpolationOperand, InterpolationView,
};
pub use operand::{
    AttributeBindingExpression, AttributeBindingExpressionView, AttributeElse, AttributeElseView,
    AttributeExpression, AttributeExpressionFailure, AttributeExpressionView, AttributeForHead,
    AttributeForHeadFailure, AttributeForHeadView, AttributeHandler, AttributeHandlerFailure,
    AttributeHandlerView, AttributeHead, AttributeModelExpression, AttributeModelExpressionView,
    AttributeOperandError, AttributeValueFailure, AttributeValueOperand, AttributeValueView,
    ConditionKind, ModelHead, StaticBindingHead,
};
pub use selected::{ScriptSelection, SelectedTemplateComponent, TemplateGrammar};
pub use tag_analysis::{TagAnalysis, TagAnalysisKind, TagAnalysisRefusal};
pub use tag_facts_component::ComponentTagAnalysis;
pub use text::{RootText, RootTextError, RootTextProfile, RootTextView};
pub use text_value::{TextValue, TextValueError, TextValueFailure, TextValueView};

/// Complete once-parsed Component privately paired with its checked source.
/// Raw mutable carriers cannot be inserted; consuming transfer drops authority.
/// A genuinely parsed arbitrary block is not proof of SFC template selection.
///
/// ```compile_fail
/// use vize_l1::markup::OwnedComponent;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<OwnedComponent<'static>>();
/// ```
pub struct OwnedComponent<'a> {
    allocator: &'a Allocator,
    block: SourceBlock<'a>,
    component: ComponentParse<'a>,
}

impl core::fmt::Debug for OwnedComponent<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("OwnedComponent")
            .field("block", &self.block)
            .field("component", &self.component)
            .finish_non_exhaustive()
    }
}

impl<'a> OwnedComponent<'a> {
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
        Self::parse_tag_facts_in(allocator, block)
    }
    // The caller's sealed owner determines bare or Descriptor-selected custody.
    // Both use the same existing default Vue 3 lint event sink exactly once.
    fn parse_tag_facts_in(
        allocator: &'a Allocator,
        block: SourceBlock<'a>,
    ) -> Result<Self, ComponentSourceError> {
        Ok(Self {
            allocator,
            block,
            component: crate::framework::vue::version::v3::surface::parse_component_with_tag_facts(
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
    pub fn children(&self) -> ChildrenView<'_, 'a> {
        ChildrenView::root(self)
    }
    /// The immutable original root slot; no iteration or membership fabrication.
    fn root_child_at(&self, ordinal: usize) -> Option<&crate::markup::SurfaceChild<'a>> {
        self.component.tree.children.get(ordinal)
    }
    #[must_use]
    pub fn into_carrier(self) -> ComponentParse<'a> {
        self.component
    }
}

#[cfg(test)]
mod reborrow_tests;
mod tag_analysis;
mod tag_facts_component;
#[cfg(test)]
mod tests;
