//! A bare default Vue 3 component parsed with tag facts, without Descriptor selection.

use super::{ChildrenView, NativeComponent};
use crate::markup::ComponentSourceError;
use vize_l0::{Allocator, SourceBlock};

/// The exact original bare source and the component parsed once with tag facts.
/// This owner preserves source-backed header facts. It grants neither SFC
/// selection, File admission, DOM semantics nor any product's success.
/// Parser errors and unsupported observations remain available unchanged.
///
/// Raw parsed carriers cannot mint this owner:
/// ```compile_fail
/// use vize_l1::markup::{ComponentParse, ComponentTagFacts};
/// fn detached<'a>(carrier: ComponentParse<'a>) -> ComponentTagFacts<'a> {
///     ComponentTagFacts::from(carrier)
/// }
/// ```
/// It cannot substitute for Descriptor selection:
/// ```compile_fail
/// use vize_l1::markup::{ComponentTagFacts, NativeTemplateComponent};
/// fn selected<'a>(bare: ComponentTagFacts<'a>) -> NativeTemplateComponent<'a> {
///     bare.into()
/// }
/// ```
/// Custody is not cloneable:
/// ```compile_fail
/// use vize_l1::markup::ComponentTagFacts;
/// fn cloneable<T: Clone>() {}
/// cloneable::<ComponentTagFacts<'static>>();
/// ```
pub struct ComponentTagFacts<'a> {
    component: NativeComponent<'a>,
}

impl core::fmt::Debug for ComponentTagFacts<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("ComponentTagFacts")
            .field("component", &self.component)
            .finish_non_exhaustive()
    }
}

impl<'a> ComponentTagFacts<'a> {
    /// Parse this exact original block once with the existing default Vue 3
    /// tag-facts sink. No source rewriting, SFC wrapping or alternate dialect occurs.
    pub fn parse_in(
        allocator: &'a Allocator,
        block: SourceBlock<'a>,
    ) -> Result<Self, ComponentSourceError> {
        Ok(Self {
            component: NativeComponent::parse_tag_facts_in(allocator, block)?,
        })
    }

    #[must_use]
    pub fn component(&self) -> &NativeComponent<'a> {
        &self.component
    }

    /// Original root children with their authentic parent and ordinal custody.
    #[must_use]
    pub fn children(&self) -> ChildrenView<'_, 'a> {
        self.component.children()
    }
}
