//! Bounded original-SFC callback; no component/slot binding semantics are inferred.
//! Checked static attributes on ordinary HTML elements may contain literal text.
//! Components, nested templates, directives, interpolation and comments retain
//! the shared owner's whole-input refusal; no original history is widened.

use super::NoDeprecatedSlotScopeAttribute;
use crate::native::{
    sfc::{NativeSfcTemplateLintContext, NativeSfcTemplateOwner, NativeSfcTemplateRule},
    template::{
        NativeTemplateAttributeKind, NativeTemplateAttributeProfile, NativeTemplateElement,
        NativeTemplateLintRefusal, NativeTemplateTextProfile,
    },
};

#[cfg(test)]
mod tests;

impl NativeSfcTemplateRule for NoDeprecatedSlotScopeAttribute {
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
            if name == "slot-scope" {
                context.error_attribute_name_with_help(
                    attribute,
                    "vue/no-deprecated-slot-scope-attribute.message",
                    &[],
                    "vue/no-deprecated-slot-scope-attribute.help",
                )?;
            }
        }
        Ok(())
    }
}
