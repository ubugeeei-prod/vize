//! The original absent-handler law on one checked static event header.
//!
//! Values stay opaque: even an empty quoted value is outside this capability.
//! A second event is refused before reporting, preserving attribute/registry
//! ordering without sorting findings or changing the shared driver.

use super::{ConfiguredValidVOn, ValidVOn, is_valid_modifier};
use crate::native::template::{
    NativeTemplateAttributeKind as Kind, NativeTemplateAttributeProfile as Profile,
    NativeTemplateElement, NativeTemplateLintContext, NativeTemplateLintRefusal as Refusal,
    NativeTemplateRule,
};
use vize_l0::String;
use vize_l1::markup::NativeLintComponent;

impl NativeTemplateRule for ValidVOn {
    fn attribute_profile(&self) -> Profile {
        Profile::Events
    }

    fn run_on_template<'a>(
        &self,
        context: &mut NativeTemplateLintContext<'_, 'a>,
        owner: &NativeLintComponent<'a>,
    ) -> Result<(), Refusal> {
        check_owner(context, owner)
    }

    fn run_on_element<'a>(
        &self,
        context: &mut NativeTemplateLintContext<'_, 'a>,
        element: &NativeTemplateElement<'_, 'a>,
    ) -> Result<(), Refusal> {
        check_element(context, element, &[])
    }
}

impl NativeTemplateRule for ConfiguredValidVOn {
    fn attribute_profile(&self) -> Profile {
        Profile::Events
    }

    fn run_on_template<'a>(
        &self,
        context: &mut NativeTemplateLintContext<'_, 'a>,
        owner: &NativeLintComponent<'a>,
    ) -> Result<(), Refusal> {
        check_owner(context, owner)
    }

    fn run_on_element<'a>(
        &self,
        context: &mut NativeTemplateLintContext<'_, 'a>,
        element: &NativeTemplateElement<'_, 'a>,
    ) -> Result<(), Refusal> {
        check_element(context, element, &self.modifiers)
    }
}

fn check_owner<'a>(
    context: &NativeTemplateLintContext<'_, 'a>,
    owner: &NativeLintComponent<'a>,
) -> Result<(), Refusal> {
    if !core::ptr::eq(context.owner(), owner) {
        return Err(Refusal::SourceMismatch);
    }
    Ok(())
}

fn check_element<'a>(
    context: &mut NativeTemplateLintContext<'_, 'a>,
    element: &NativeTemplateElement<'_, 'a>,
    custom_modifiers: &[String],
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
    let Some(attribute) = first else {
        return Ok(());
    };
    if attribute.value().is_some() {
        return Err(Refusal::UnsupportedAttribute {
            span: attribute.range(),
        });
    }
    let mut has_verb = false;
    for modifier in context.event_modifiers(attribute)? {
        let modifier = modifier?;
        let text = modifier.text();
        if !is_valid_modifier(text, custom_modifiers) {
            context.error_event_modifier_with_help(
                &modifier,
                "vue/valid-v-on.unsupported_modifier",
                &[("modifier", text)],
                "vue/valid-v-on.modifier_help",
                None,
            )?;
        }
        has_verb |= matches!(text, "stop" | "prevent");
    }
    if !has_verb {
        context.error_attribute_with_help(
            attribute,
            "vue/valid-v-on.missing_event",
            &[],
            "vue/valid-v-on.help",
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
