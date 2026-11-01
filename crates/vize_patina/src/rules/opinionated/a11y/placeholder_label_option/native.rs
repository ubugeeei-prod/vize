//! Checked direct-option headers only. Text, bindings and entities never
//! manufacture an option value; unsupported owners refuse the whole result.

use super::PlaceholderLabelOption;
use crate::native::template::{
    NativeTemplateAttributeKind as Kind, NativeTemplateElement, NativeTemplateLintContext,
    NativeTemplateLintRefusal as Refusal, NativeTemplateOptionValue, NativeTemplateOwnerProfile,
    NativeTemplateRule,
};
use vize_l1::markup::NativeLintComponent;

impl NativeTemplateRule for PlaceholderLabelOption {
    fn owner_profile(&self) -> NativeTemplateOwnerProfile {
        NativeTemplateOwnerProfile::ClosedSelect
    }

    fn run_on_template<'a>(
        &self,
        context: &mut NativeTemplateLintContext<'_, 'a>,
        root: &NativeLintComponent<'a>,
    ) -> Result<(), Refusal> {
        if !core::ptr::eq(context.owner(), root) {
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
        let mut required = false;
        let mut multiple = false;
        let mut size_one = true;
        let mut explicit_value = false;
        for attribute in element.attributes() {
            match attribute.kind() {
                Kind::Static { name, value } => {
                    if value.is_some_and(|value| value.contains('&')) {
                        return Err(Refusal::UnsupportedAttribute {
                            span: attribute.range(),
                        });
                    }
                    match name {
                        "required" => required = true,
                        "multiple" => multiple = true,
                        "size" => size_one = value.is_some_and(|value| value.trim() == "1"),
                        "value" => explicit_value = true,
                        _ => {}
                    }
                }
                Kind::Bind { .. } | Kind::DynamicBind { .. } | Kind::On { .. } => {
                    return Err(Refusal::UnsupportedAttribute {
                        span: attribute.range(),
                    });
                }
            }
        }
        let tag = element.original().surface().tag();
        if tag == "option" && !explicit_value {
            return Err(Refusal::UnsupportedContext {
                span: context.element_opening_range(element)?,
            });
        }
        if tag != "select" {
            return Ok(());
        }
        let owner = element
            .closed_select_owner()
            .ok_or(Refusal::UnsupportedContext {
                span: context.element_opening_range(element)?,
            })?;
        // This also authenticates the real frozen callback for empty owners.
        let Some(first) = context.first_option(&owner)? else {
            return Ok(());
        };
        let nonempty = match context.first_option_value(&owner, &first)? {
            NativeTemplateOptionValue::Static { value, .. } => {
                value.is_some_and(|value| !value.is_empty())
            }
            NativeTemplateOptionValue::Absent => {
                return Err(Refusal::UnsupportedContext {
                    span: context.first_option_opening_range(&owner, &first)?,
                });
            }
            NativeTemplateOptionValue::Bound { range } => {
                return Err(Refusal::UnsupportedAttribute { span: range });
            }
        };
        if required && !multiple && size_one && nonempty {
            context.warn_first_option_with_help(
                &owner,
                &first,
                "a11y/placeholder-label-option.message",
                "a11y/placeholder-label-option.help",
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
