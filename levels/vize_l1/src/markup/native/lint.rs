//! Normally owned bare default Vue 3 lint syntax, without Descriptor selection.

use super::{NativeChildren, NativeComponent};
use crate::markup::ComponentSourceError;
use vize_l0::{Allocator, SourceBlock};

/// The exact original bare source and once-parsed lint-enabled component.
/// This owner preserves source-backed lint-header receipts. It grants neither
/// SFC selection, File admission, DOM semantics nor whole-product lint success.
/// Parser errors and unsupported observations remain available unchanged.
///
/// Raw parsed carriers cannot mint this owner:
/// ```compile_fail
/// use vize_l1::markup::{ComponentParse, NativeLintComponent};
/// fn detached<'a>(carrier: ComponentParse<'a>) -> NativeLintComponent<'a> {
///     NativeLintComponent::from(carrier)
/// }
/// ```
/// It cannot substitute for Descriptor selection:
/// ```compile_fail
/// use vize_l1::markup::{NativeLintComponent, NativeTemplateComponent};
/// fn selected<'a>(bare: NativeLintComponent<'a>) -> NativeTemplateComponent<'a> {
///     bare.into()
/// }
/// ```
/// Custody is not cloneable:
/// ```compile_fail
/// use vize_l1::markup::NativeLintComponent;
/// fn cloneable<T: Clone>() {}
/// cloneable::<NativeLintComponent<'static>>();
/// ```
pub struct NativeLintComponent<'a> {
    component: NativeComponent<'a>,
}

impl core::fmt::Debug for NativeLintComponent<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("NativeLintComponent")
            .field("component", &self.component)
            .finish_non_exhaustive()
    }
}

impl<'a> NativeLintComponent<'a> {
    /// Parse this exact original block once with the existing default Vue 3
    /// lint sink. No source rewriting, SFC wrapping or alternate dialect occurs.
    pub fn parse_in(
        allocator: &'a Allocator,
        block: SourceBlock<'a>,
    ) -> Result<Self, ComponentSourceError> {
        Ok(Self {
            component: NativeComponent::parse_lint_in(allocator, block)?,
        })
    }

    #[must_use]
    pub fn component(&self) -> &NativeComponent<'a> {
        &self.component
    }

    /// Original root children with their authentic parent and ordinal custody.
    #[must_use]
    pub fn children(&self) -> NativeChildren<'_, 'a> {
        self.component.children()
    }
}
