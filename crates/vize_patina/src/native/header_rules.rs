//! Classification-dependent rules over one original complete authored header.

use vize_l0::{
    SmallVec, Span, String,
    diag::{Advisory, Diagnostic, DiagnosticPart, MessageLookup, PartKind, Stage},
};
use vize_l1::markup::{NativeElement, NativeLintTagKind};

use super::{
    NO_ACCESS_KEY_RULE, NO_AUTOFOCUS_RULE, NO_DISTRACTING_ELEMENTS_RULE, NativeLintFinding,
    NativeLintRefusal, attribute, header,
};

fn inspect(
    element: &NativeElement<'_, '_>,
    mut visit: impl FnMut(header::Binding<'_>),
) -> Result<(NativeLintTagKind, Span), NativeLintRefusal> {
    let receipt = element
        .lint_tag()
        .map_err(|reason| NativeLintRefusal::LintTag { reason })?;
    let opening = header::opening_range(element)?;
    let mut names: SmallVec<[&str; 8]> = SmallVec::new();
    for original in element.attributes() {
        let binding = header::strict_binding(element, &original)?;
        if let header::Binding::Static { name, .. } = &binding {
            if names
                .iter()
                .any(|previous| previous.eq_ignore_ascii_case(name))
            {
                let span =
                    attribute::span(element.component().block(), original.surface().name.text)?;
                return Err(NativeLintRefusal::DuplicateAttribute { span });
            }
            names.push(name);
        }
        visit(binding);
    }
    Ok((receipt.kind(), opening))
}

struct AttributeRule {
    name: &'static str,
    attribute: &'static str,
    message: &'static str,
    help: &'static str,
}

fn attribute_rule(
    element: &NativeElement<'_, '_>,
    messages: &impl MessageLookup,
    rule: AttributeRule,
) -> Result<Vec<NativeLintFinding>, NativeLintRefusal> {
    let mut matches: SmallVec<[Span; 4]> = SmallVec::new();
    let (kind, _) = inspect(element, |binding| match binding {
        header::Binding::Static { name, range, .. } | header::Binding::Bind { name, range }
            if name == rule.attribute =>
        {
            matches.push(range);
        }
        _ => {}
    })?;
    if kind == NativeLintTagKind::Component {
        return Ok(Vec::new());
    }
    Ok(matches
        .into_iter()
        .map(|range| {
            finding(
                rule.name,
                range,
                messages.lookup(rule.message).as_ref(),
                messages.lookup(rule.help).as_ref(),
            )
        })
        .collect())
}

pub(super) fn no_autofocus(
    element: &NativeElement<'_, '_>,
    messages: &impl MessageLookup,
) -> Result<Vec<NativeLintFinding>, NativeLintRefusal> {
    attribute_rule(
        element,
        messages,
        AttributeRule {
            name: NO_AUTOFOCUS_RULE,
            attribute: "autofocus",
            message: "a11y/no-autofocus.message",
            help: "a11y/no-autofocus.help",
        },
    )
}

pub(super) fn no_access_key(
    element: &NativeElement<'_, '_>,
    messages: &impl MessageLookup,
) -> Result<Vec<NativeLintFinding>, NativeLintRefusal> {
    attribute_rule(
        element,
        messages,
        AttributeRule {
            name: NO_ACCESS_KEY_RULE,
            attribute: "accesskey",
            message: "a11y/no-access-key.message",
            help: "a11y/no-access-key.help",
        },
    )
}

pub(super) fn no_distracting_elements(
    element: &NativeElement<'_, '_>,
    messages: &impl MessageLookup,
) -> Result<Option<NativeLintFinding>, NativeLintRefusal> {
    let (kind, range) = inspect(element, |_| {})?;
    let tag = element.surface().tag();
    if kind == NativeLintTagKind::Component || !matches!(tag, "marquee" | "blink") {
        return Ok(None);
    }
    let template = messages.lookup("a11y/no-distracting-elements.message");
    let mut message = String::new("");
    for (index, piece) in template.split("{tag}").enumerate() {
        if index != 0 {
            message.push_str(tag);
        }
        message.push_str(piece);
    }
    Ok(Some(finding(
        NO_DISTRACTING_ELEMENTS_RULE,
        range,
        message.as_str(),
        messages
            .lookup("a11y/no-distracting-elements.help")
            .as_ref(),
    )))
}

fn finding(rule_name: &'static str, range: Span, message: &str, help: &str) -> NativeLintFinding {
    NativeLintFinding {
        rule_name,
        diagnostic: Diagnostic::new(Advisory::Warning, Stage::Surface, range, message)
            .with_part(DiagnosticPart::new(PartKind::Help, range, help)),
    }
}
