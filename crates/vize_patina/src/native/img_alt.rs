//! The first native syntax rule, retaining the existing `a11y/img-alt` code.

use vize_l0::diag::{Advisory, Diagnostic, DiagnosticPart, MessageLookup, PartKind, Stage};
use vize_l1::markup::NativeElement;

use super::{IMG_ALT_RULE, NativeLintFinding, NativeLintRefusal, header};

pub(super) fn check(
    element: &NativeElement<'_, '_>,
    messages: &impl MessageLookup,
) -> Result<Option<NativeLintFinding>, NativeLintRefusal> {
    let range = header::opening_range(element)?;
    let mut has_alt = false;
    for original in element.attributes() {
        has_alt |= match header::binding(element, &original)? {
            header::Binding::Static { name, .. } | header::Binding::Bind { name } => {
                name.eq_ignore_ascii_case("alt")
            }
            header::Binding::Other => false,
        };
    }
    if !element.surface().tag().eq_ignore_ascii_case("img") || has_alt {
        return Ok(None);
    }
    let diagnostic = Diagnostic::new(
        Advisory::Warning,
        Stage::Surface,
        range,
        messages.lookup("a11y/img-alt.message").as_ref(),
    )
    .with_part(DiagnosticPart::new(
        PartKind::Help,
        range,
        messages.lookup("a11y/img-alt.help").as_ref(),
    ));
    Ok(Some(NativeLintFinding {
        rule_name: IMG_ALT_RULE,
        diagnostic,
    }))
}
