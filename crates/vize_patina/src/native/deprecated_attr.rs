//! Exact static HTML attribute policy over the genuine selected header.

use vize_l0::{
    SmallVec, Span, String,
    diag::{Advisory, Diagnostic, DiagnosticPart, MessageLookup, PartKind, Stage},
};
use vize_l1::markup::{NativeElement, NativeLintTagKind};

use crate::attribute_policy::deprecated_attr_suggestion;

use super::{DEPRECATED_ATTR_RULE, NativeLintFinding, NativeLintRefusal, header, header_rules};

pub(super) fn check(
    element: &NativeElement<'_, '_>,
    messages: &impl MessageLookup,
) -> Result<Vec<NativeLintFinding>, NativeLintRefusal> {
    let tag = element.surface().tag();
    let mut matches: SmallVec<[(&str, &'static str, Span); 4]> = SmallVec::new();
    let (receipt, _) = header_rules::inspect_attributes(element, |_, binding| {
        if let header::Binding::Static { name, range, .. } = binding
            && let Some(suggestion) = deprecated_attr_suggestion(tag, name)
        {
            matches.push((name, suggestion, range));
        }
    })?;
    if receipt.kind() == NativeLintTagKind::Component {
        return Ok(Vec::new());
    }
    Ok(matches
        .into_iter()
        .map(|(name, suggestion, range)| {
            let message = substitute(
                &substitute(
                    messages.lookup("html/deprecated-attr.message").as_ref(),
                    "{attr}",
                    name,
                ),
                "{tag}",
                tag,
            );
            let help = substitute(
                messages.lookup("html/deprecated-attr.help").as_ref(),
                "{suggestion}",
                suggestion,
            );
            NativeLintFinding {
                rule_name: DEPRECATED_ATTR_RULE,
                diagnostic: Diagnostic::new(
                    Advisory::Warning,
                    Stage::Surface,
                    range,
                    message.as_str(),
                )
                .with_part(DiagnosticPart::new(
                    PartKind::Help,
                    range,
                    help.as_str(),
                )),
            }
        })
        .collect())
}

// Preserve the registered rule's attr-then-tag substitutions, including all
// repeated placeholders and literal placeholders injected by authored tags.
fn substitute(template: &str, placeholder: &str, value: &str) -> String {
    let mut result = String::new("");
    for (index, piece) in template.split(placeholder).enumerate() {
        if index != 0 {
            result.push_str(value);
        }
        result.push_str(piece);
    }
    result
}
