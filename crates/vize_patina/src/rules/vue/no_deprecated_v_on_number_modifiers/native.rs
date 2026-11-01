//! Same original numeric law over a genuine checked bare event modifier.

use super::{NoDeprecatedVOnNumberModifiers, decimal_key_code, named_key};
use crate::native::template::{
    NativeTemplateAttributeKind as Kind, NativeTemplateAttributeProfile as Profile,
    NativeTemplateElement, NativeTemplateLintContext, NativeTemplateLintRefusal as Refusal,
    NativeTemplateRule,
};
use vize_l1::markup::NativeLintComponent;

#[cfg(test)]
mod tests;

impl NativeTemplateRule for NoDeprecatedVOnNumberModifiers {
    fn attribute_profile(&self) -> Profile {
        Profile::Events
    }

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
        for attribute in element.attributes() {
            if !matches!(attribute.kind(), Kind::On { .. }) {
                continue;
            }
            let mut first = None;
            for modifier in context.event_modifiers(attribute)? {
                let modifier = modifier?;
                if let Some(number) = decimal_key_code(modifier.text()) {
                    first = Some((modifier, number));
                    break;
                }
            }
            let Some((modifier, number)) = first else {
                continue;
            };
            if (0.0..=9.0).contains(&number) {
                continue;
            }
            context.error_event_modifier_with_help(
                &modifier,
                "vue/no-deprecated-v-on-number-modifiers.message",
                &[("keyCode", modifier.text())],
                "vue/no-deprecated-v-on-number-modifiers.help",
                named_key(number)
                    .map(|key| ("Replace the numeric keyCode modifier with a named key", key)),
            )?;
        }
        Ok(())
    }
}
