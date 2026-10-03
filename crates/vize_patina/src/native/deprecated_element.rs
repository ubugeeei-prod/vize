//! One opt-in authored-tag Warning, using the original strict header admission.

use vize_l0::{
    String,
    diag::{Advisory, Diagnostic, DiagnosticPart, MessageLookup, PartKind, Stage},
};
use vize_l1::markup::{NativeElement, NativeLintTagKind};

use crate::tag_policy::DEPRECATED_ELEMENTS;

use super::{DEPRECATED_ELEMENT_RULE, NativeLintFinding, NativeLintRefusal, header_rules};

pub(super) fn check(
    element: &NativeElement<'_, '_>,
    messages: &impl MessageLookup,
) -> Result<Option<NativeLintFinding>, NativeLintRefusal> {
    let (kind, opening) = header_rules::inspect(element, |_| {})?;
    let tag = element.surface().tag();
    if kind == NativeLintTagKind::Component || !DEPRECATED_ELEMENTS.contains(&tag) {
        return Ok(None);
    }
    let template = messages.lookup("html/deprecated-element.message");
    let mut message = String::new("");
    for (index, piece) in template.split("{tag}").enumerate() {
        if index != 0 {
            message.push_str(tag);
        }
        message.push_str(piece);
    }
    let help = messages.lookup("html/deprecated-element.help");
    Ok(Some(NativeLintFinding {
        rule_name: DEPRECATED_ELEMENT_RULE,
        diagnostic: Diagnostic::new(Advisory::Warning, Stage::Surface, opening, message.as_str())
            .with_part(DiagnosticPart::new(PartKind::Help, opening, help.as_ref())),
    }))
}
