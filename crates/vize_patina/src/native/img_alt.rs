//! The first native syntax rule, retaining the existing `a11y/img-alt` code.

use vize_l0::diag::MessageLookup;
use vize_l1::markup::NativeElement;

use super::{IMG_ALT_RULE, NativeLintFinding, NativeLintRefusal, header};
use crate::attribute_policy::{ImageAltMessages, ImageAltPolicy};

pub(super) fn check(
    element: &NativeElement<'_, '_>,
    messages: &impl MessageLookup,
) -> Result<Option<NativeLintFinding>, NativeLintRefusal> {
    let range = header::opening_range(element)?;
    let mut has_alt = false;
    for original in element.attributes() {
        has_alt |= match header::binding(element, &original)? {
            header::Binding::Static { name, .. } | header::Binding::Bind { name, .. } => {
                name.eq_ignore_ascii_case("alt")
            }
            header::Binding::Other { .. } => false,
        };
    }
    if !ImageAltPolicy::for_tag(element.surface().tag()).is_missing(has_alt) {
        return Ok(None);
    }
    Ok(Some(NativeLintFinding {
        rule_name: IMG_ALT_RULE,
        diagnostic: ImageAltMessages::new(messages).diagnostic(range),
    }))
}
