//! Configured bare-HTML callback over the existing strict checked owner.
//! Component tags, directives and SFC ownership keep their existing refusal.

use super::NoDeprecatedInlineTemplate;
use crate::native::template::{
    NativeTemplateAttributeKind, NativeTemplateElement, NativeTemplateLintContext,
    NativeTemplateLintRefusal, NativeTemplateRule,
};
use vize_l1::markup::NativeLintComponent;

impl NativeTemplateRule for NoDeprecatedInlineTemplate {
    fn run_on_template<'a>(
        &self,
        context: &mut NativeTemplateLintContext<'_, 'a>,
        owner: &NativeLintComponent<'a>,
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
                    "vue/no-deprecated-inline-template.message",
                    &[],
                    "vue/no-deprecated-inline-template.help",
                )?;
            }
        }
        Ok(())
    }
}
