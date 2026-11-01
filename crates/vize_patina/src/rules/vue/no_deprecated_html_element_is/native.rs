//! Static original HTML attributes through the existing strict bare owner.
//! Values requiring entity decoding remain unprovided as a whole input.

use super::NoDeprecatedHtmlElementIs;
use crate::native::template::{
    NativeTemplateAttributeKind as Kind, NativeTemplateElement, NativeTemplateLintContext,
    NativeTemplateLintRefusal as Refusal, NativeTemplateRule,
};
use vize_l1::markup::NativeLintComponent;

#[cfg(test)]
mod tests;

impl NativeTemplateRule for NoDeprecatedHtmlElementIs {
    fn run_on_template<'a>(
        &self,
        context: &mut NativeTemplateLintContext<'_, 'a>,
        owner: &NativeLintComponent<'a>,
    ) -> Result<(), Refusal> {
        if !core::ptr::eq(context.owner(), owner) {
            return Err(Refusal::SourceMismatch);
        }
        Ok(())
    }

    fn run_on_element<'a>(
        &self,
        context: &mut NativeTemplateLintContext<'_, 'a>,
        element: &NativeTemplateElement<'_, 'a>,
    ) -> Result<(), Refusal> {
        if !core::ptr::eq(context.owner().component(), element.original().component()) {
            return Err(Refusal::SourceMismatch);
        }
        // The shared strict owner already proves an ordinary HTML element;
        // components, dynamic-component owners and recovery remain refused.
        for attribute in element.attributes() {
            let Kind::Static { name, value } = attribute.kind() else {
                return Err(Refusal::UnsupportedAttribute {
                    span: attribute.range(),
                });
            };
            if name != "is" {
                continue;
            }
            if value.is_some_and(|text| text.contains('&')) {
                return Err(Refusal::UnsupportedAttribute {
                    span: attribute.range(),
                });
            }
            if value.is_some_and(|text| text.starts_with("vue:")) {
                continue;
            }
            context.error_attribute_with_help(
                attribute,
                "vue/no-deprecated-html-element-is.message",
                &[],
                "vue/no-deprecated-html-element-is.help",
            )?;
        }
        Ok(())
    }
}
