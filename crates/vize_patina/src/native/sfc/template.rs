//! A distinct genuine template-only SFC owner and configured rule capability.

mod context;
mod driver;
mod owner;

pub use context::NativeSfcTemplateLintContext;
pub(super) use driver::lint;
pub use owner::NativeSfcTemplateOwner;

use crate::native::template::{
    NativeTemplateAttributeProfile, NativeTemplateElement, NativeTemplateLintRefusal,
    NativeTemplateTextProfile,
};

/// Offered only by the actual registered instance. This owner is neither a
/// synthetic bare template nor the primitive TypeScript setup SFC owner.
pub trait NativeSfcTemplateRule: Send + Sync {
    /// Declares the strongest original-attribute grammar accepted by the rule.
    fn attribute_profile(&self) -> NativeTemplateAttributeProfile {
        NativeTemplateAttributeProfile::StaticOnly
    }

    /// Declares the strongest original-text grammar accepted by the rule.
    fn text_profile(&self) -> NativeTemplateTextProfile {
        NativeTemplateTextProfile::WhitespaceOnly
    }

    /// Runs once against the authenticated whole template owner.
    fn run_on_template<'a>(
        &self,
        context: &mut NativeSfcTemplateLintContext<'_, 'a>,
        owner: &NativeSfcTemplateOwner<'a>,
    ) -> Result<(), NativeTemplateLintRefusal>;

    /// Runs for each admitted original element during the existing traversal.
    fn run_on_element<'a>(
        &self,
        _context: &mut NativeSfcTemplateLintContext<'_, 'a>,
        _element: &NativeTemplateElement<'_, 'a>,
    ) -> Result<(), NativeTemplateLintRefusal> {
        Ok(())
    }
}
