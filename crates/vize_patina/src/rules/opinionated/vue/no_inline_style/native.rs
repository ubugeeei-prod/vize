//! Static style syntax through the existing checked bare-template host.

use super::NoInlineStyle;
use crate::native::template::{
    NativeTemplateAttributeKind as Kind, NativeTemplateAttributeProfile as Profile,
    NativeTemplateElement, NativeTemplateLintContext, NativeTemplateLintRefusal as Refusal,
    NativeTemplateRule,
};
use vize_l1::markup::ComponentTagAnalysis;

impl NativeTemplateRule for NoInlineStyle {
    fn attribute_profile(&self) -> Profile {
        Profile::Bindings
    }

    fn run_on_template<'a>(
        &self,
        context: &mut NativeTemplateLintContext<'_, 'a>,
        owner: &ComponentTagAnalysis<'a>,
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
        for attribute in element.attributes() {
            match attribute.kind() {
                Kind::Static { name: "style", .. } => context.warn_attribute_with_help(
                    attribute,
                    "vue/no-inline-style.message",
                    &[],
                    "vue/no-inline-style.help",
                )?,
                Kind::Static { .. } | Kind::Bind { .. } | Kind::DynamicBind { .. } => {}
                Kind::On { .. } => {
                    return Err(Refusal::UnsupportedAttribute {
                        span: attribute.range(),
                    });
                }
            }
        }
        Ok(())
    }
}
