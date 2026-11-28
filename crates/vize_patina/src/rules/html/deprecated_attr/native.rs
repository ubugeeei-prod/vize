//! Exact static deprecated attributes in the existing strict bare host.

use super::{DeprecatedAttr, deprecated_attr_suggestion};
use crate::native::template::{
    NativeTemplateAttributeKind as Kind, NativeTemplateAttributeProfile as Profile,
    NativeTemplateElement, NativeTemplateLintContext, NativeTemplateLintRefusal as Refusal,
    NativeTemplateRule,
};
use vize_l1::markup::ComponentTagAnalysis;

impl NativeTemplateRule for DeprecatedAttr {
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
        let tag = element.original().surface().tag();
        for attribute in element.attributes() {
            let name = match attribute.kind() {
                Kind::Static { name, .. } => name,
                Kind::Bind { .. } | Kind::DynamicBind { .. } => continue,
                Kind::On { .. } => {
                    return Err(Refusal::UnsupportedAttribute {
                        span: attribute.range(),
                    });
                }
            };
            if let Some(suggestion) = deprecated_attr_suggestion(tag, name) {
                context.warn_attribute_with_formatted_help(
                    attribute,
                    "html/deprecated-attr.message",
                    &[("attr", name), ("tag", tag)],
                    "html/deprecated-attr.help",
                    &[("suggestion", suggestion)],
                )?;
            }
        }
        Ok(())
    }
}
