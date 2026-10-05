//! Exact authored accesskey syntax through the existing sealed bare host.

use super::NoAccessKey;
use crate::native::template::{
    NativeTemplateAttributeKind as Kind, NativeTemplateAttributeProfile as Profile,
    NativeTemplateElement, NativeTemplateLintContext, NativeTemplateLintRefusal as Refusal,
    NativeTemplateRule,
};
use vize_l1::markup::NativeLintComponent;

impl NativeTemplateRule for NoAccessKey {
    fn attribute_profile(&self) -> Profile {
        Profile::Bindings
    }

    fn run_on_template<'a>(
        &self,
        context: &mut NativeTemplateLintContext<'_, 'a>,
        root: &NativeLintComponent<'a>,
    ) -> Result<(), Refusal> {
        if !core::ptr::eq(context.owner(), root) {
            return Err(Refusal::SourceMismatch);
        }
        Ok(())
    }

    fn run_on_element<'a>(
        &self,
        context: &mut NativeTemplateLintContext<'_, 'a>,
        element: &NativeTemplateElement<'_, 'a>,
    ) -> Result<(), Refusal> {
        // Check even an empty/dynamic-only header before any absence finding.
        if !core::ptr::eq(context.owner().component(), element.original().component()) {
            return Err(Refusal::SourceMismatch);
        }
        for attribute in element.attributes() {
            let name = match attribute.kind() {
                Kind::Static { name, .. } | Kind::Bind { name, .. } => name,
                Kind::DynamicBind { .. } => continue,
            };
            if name == "accesskey" {
                context.warn_attribute_with_help(
                    attribute,
                    "a11y/no-access-key.message",
                    &[],
                    "a11y/no-access-key.help",
                )?;
            }
        }
        Ok(())
    }
}
