//! Partial native input capability over one original checked header.
//! Label ancestry, expression-derived types and decoded values are unprovided;
//! refusing any such owner discards the complete pending result.

use super::FormControlHasLabel;
use crate::native::template::{
    NativeTemplateAttributeKind as Kind, NativeTemplateAttributeProfile as Profile,
    NativeTemplateElement, NativeTemplateLintContext, NativeTemplateLintRefusal as Refusal,
    NativeTemplateRule,
};
use vize_l1::markup::NativeLintComponent;

impl NativeTemplateRule for FormControlHasLabel {
    fn attribute_profile(&self) -> Profile {
        Profile::Bindings
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
        let original = element.original();
        if !core::ptr::eq(context.owner().component(), original.component()) {
            return Err(Refusal::SourceMismatch);
        }
        let tag = original.surface().tag();
        // This callback has no authenticated inherited-label fact. Refuse the
        // owner before its children, including when an earlier input warned.
        if tag == "label" {
            return Err(Refusal::UnsupportedContext {
                span: crate::native::header::opening_range(original)?,
            });
        }
        if tag != "input" {
            return Ok(());
        }
        let mut hidden = false;
        let mut unknown_hidden = false;
        let mut exempt_type = false;
        let mut labelled = false;
        let mut placeholder = false;
        for attribute in element.attributes() {
            match attribute.kind() {
                Kind::Static { name, value } => {
                    // Native values remain exact opaque source. Do not infer
                    // the legacy parser's entity-decoded string semantics.
                    if value.is_some_and(|value| value.contains('&')) {
                        return Err(Refusal::UnsupportedAttribute {
                            span: attribute.range(),
                        });
                    }
                    let nonempty = value.is_some_and(|value| !value.trim().is_empty());
                    match name {
                        "hidden" => {
                            hidden = value.is_none_or(|value| {
                                value.is_empty() || value.eq_ignore_ascii_case("hidden")
                            })
                        }
                        "type" => {
                            exempt_type = value.is_some_and(|value| {
                                matches!(value, "hidden" | "submit" | "reset" | "button" | "image")
                            })
                        }
                        "id" | "aria-label" | "aria-labelledby" | "title" => labelled |= nonempty,
                        "placeholder" => placeholder |= nonempty,
                        _ => {}
                    }
                }
                Kind::Bind { name: "type", .. } => {
                    return Err(Refusal::UnsupportedAttribute {
                        span: attribute.range(),
                    });
                }
                Kind::Bind { name, .. } => {
                    unknown_hidden |= name == "hidden";
                    labelled |= matches!(name, "id" | "aria-label" | "aria-labelledby");
                }
                // Dynamic argument expressions have no authenticated label
                // name here; do not turn an unknown runtime binding clean.
                Kind::DynamicBind { .. } => {
                    return Err(Refusal::UnsupportedAttribute {
                        span: attribute.range(),
                    });
                }
            }
        }
        if exempt_type || (hidden && !unknown_hidden) || labelled {
            return Ok(());
        }
        let help = if placeholder {
            "a11y/form-control-has-label.help_placeholder"
        } else {
            "a11y/form-control-has-label.help"
        };
        context.warn_void_element_with_help(
            element,
            "a11y/form-control-has-label.message",
            &[("tag", "input")],
            help,
        )
    }
}

#[cfg(test)]
mod tests;
