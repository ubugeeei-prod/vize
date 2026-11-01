//! The unchanged first exact `.native` law, on one genuine event header.
//!
//! One event attribute per element is a deliberate closed subset: the native
//! driver dispatches callbacks before attributes, whereas the legacy visitor
//! dispatches attributes before callbacks. Refuse a second event before any
//! finding here, rather than reorder or publish a partial multi-rule result.

use super::NoDeprecatedVOnNativeModifier;
use crate::native::template::{
    NativeTemplateAttributeKind as Kind, NativeTemplateAttributeProfile as Profile,
    NativeTemplateElement, NativeTemplateLintContext, NativeTemplateLintRefusal as Refusal,
    NativeTemplateRule,
};
use vize_l1::markup::NativeLintComponent;

#[cfg(test)]
mod tests;

impl NativeTemplateRule for NoDeprecatedVOnNativeModifier {
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
        let mut events = element
            .attributes()
            .iter()
            .filter(|attribute| matches!(attribute.kind(), Kind::On { .. }));
        let first = events.next();
        if let Some(second) = events.next() {
            return Err(Refusal::UnsupportedAttribute {
                span: second.range(),
            });
        }
        if let Some(attribute) = first {
            for modifier in context.event_modifiers(attribute)? {
                let modifier = modifier?;
                if modifier.text() == "native" {
                    context.error_event_modifier_with_help(
                        &modifier,
                        "vue/no-deprecated-v-on-native-modifier.message",
                        &[],
                        "vue/no-deprecated-v-on-native-modifier.help",
                        None,
                    )?;
                    break;
                }
            }
        }
        Ok(())
    }
}
