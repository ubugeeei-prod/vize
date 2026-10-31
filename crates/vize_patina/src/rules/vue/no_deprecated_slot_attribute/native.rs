//! Configured native callback for the original template-only SFC owner.

use super::NoDeprecatedSlotAttribute;
use crate::native::{
    sfc::{NativeSfcTemplateLintContext, NativeSfcTemplateOwner, NativeSfcTemplateRule},
    template::{
        NativeTemplateAttributeKind, NativeTemplateAttributeProfile, NativeTemplateElement,
        NativeTemplateLintRefusal, NativeTemplateTextProfile,
    },
};

impl NativeSfcTemplateRule for NoDeprecatedSlotAttribute {
    fn attribute_profile(&self) -> NativeTemplateAttributeProfile {
        NativeTemplateAttributeProfile::StaticOnly
    }

    fn text_profile(&self) -> NativeTemplateTextProfile {
        NativeTemplateTextProfile::LiteralText
    }

    fn run_on_template<'a>(
        &self,
        context: &mut NativeSfcTemplateLintContext<'_, 'a>,
        owner: &NativeSfcTemplateOwner<'a>,
    ) -> Result<(), NativeTemplateLintRefusal> {
        if !core::ptr::eq(context.owner(), owner) {
            return Err(NativeTemplateLintRefusal::SourceMismatch);
        }
        Ok(())
    }

    fn run_on_element<'a>(
        &self,
        context: &mut NativeSfcTemplateLintContext<'_, 'a>,
        element: &NativeTemplateElement<'_, 'a>,
    ) -> Result<(), NativeTemplateLintRefusal> {
        if !core::ptr::eq(
            context.owner().template().component(),
            element.original().component(),
        ) {
            return Err(NativeTemplateLintRefusal::SourceMismatch);
        }
        for attribute in element.attributes() {
            let NativeTemplateAttributeKind::Static { name, .. } = attribute.kind() else {
                return Err(NativeTemplateLintRefusal::UnsupportedAttribute {
                    span: attribute.range(),
                });
            };
            if name == "slot" {
                context.error_attribute_name_with_help(
                    attribute,
                    "vue/no-deprecated-slot-attribute.message",
                    &[],
                    "vue/no-deprecated-slot-attribute.help",
                )?;
            }
        }
        Ok(())
    }
}
