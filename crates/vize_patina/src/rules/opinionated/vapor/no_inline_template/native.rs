//! The actual rule instance's checked bare-template callback.

use super::NoInlineTemplate;
use crate::native::template::{
    NativeTemplateAttributeKind, NativeTemplateElement, NativeTemplateLintContext,
    NativeTemplateLintRefusal, NativeTemplateRule,
};
use vize_l1::markup::ComponentTagAnalysis;

impl NativeTemplateRule for NoInlineTemplate {
    fn run_on_template<'a>(
        &self,
        context: &mut NativeTemplateLintContext<'_, 'a>,
        owner: &ComponentTagAnalysis<'a>,
    ) -> Result<(), NativeTemplateLintRefusal> {
        if !core::ptr::eq(context.owner(), owner) {
            return Err(NativeTemplateLintRefusal::SourceMismatch);
        }
        Ok(())
    }

    fn run_on_element<'a>(
        &self,
        context: &mut NativeTemplateLintContext<'_, 'a>,
        element: &NativeTemplateElement<'_, 'a>,
    ) -> Result<(), NativeTemplateLintRefusal> {
        if !core::ptr::eq(context.owner().component(), element.original().component()) {
            return Err(NativeTemplateLintRefusal::SourceMismatch);
        }
        for attribute in element.attributes() {
            let NativeTemplateAttributeKind::Static { name, .. } = attribute.kind() else {
                return Err(NativeTemplateLintRefusal::UnsupportedAttribute {
                    span: attribute.range(),
                });
            };
            if name.eq_ignore_ascii_case("inline-template") {
                context.error_attribute_with_help(
                    attribute,
                    "vapor/no-inline-template.message",
                    &[],
                    "vapor/no-inline-template.help",
                )?;
            }
        }
        Ok(())
    }
}
