//! Attributes, directives and the element category.
//!
//! Directive names stay raw in the L1 tree; the Vue dialect's own syntax hook
//! ([`VueDirectives`]) splits them into name, argument and modifiers, so this
//! writer never re-parses directive syntax.

use vize_l0::Span;
use vize_l1::markup::entity::EntityContext;
use vize_l1::markup::{
    ArgSyntax, Attribute, DirectiveName, DirectivePrefix, DirectiveSyntax, Element, VueDirectives,
};

use super::tree::Writer;

/// The element category, as the SFC compiler classifies it: `v-pre` content
/// is plain markup, `<slot>` an outlet, `<template>` with a structural
/// directive a fragment, and a tag that is not a platform element (or a
/// `vue:` cast, `<component>` or a built-in) a component.
pub(super) fn tag_type(writer: &Writer<'_>, element: &Element<'_>, verbatim: bool) -> &'static str {
    if verbatim {
        return "ELEMENT";
    }
    let tag = element.tag();
    let attrs = &element.open.attrs;
    if tag == "slot" {
        "SLOT"
    } else if tag == "template" && attrs.iter().any(|attr| is_structural(writer, attr)) {
        "TEMPLATE"
    } else if is_component(writer, tag, attrs) {
        "COMPONENT"
    } else {
        "ELEMENT"
    }
}

fn is_structural(writer: &Writer<'_>, attr: &Attribute<'_>) -> bool {
    match head(writer, attr) {
        Some(head) => match head.prefix {
            DirectivePrefix::Slot => true,
            DirectivePrefix::Full => matches!(
                slice(writer.source, head.name),
                "if" | "else-if" | "else" | "for" | "slot"
            ),
            _ => false,
        },
        None => false,
    }
}

fn is_component(writer: &Writer<'_>, tag: &str, attrs: &[Attribute<'_>]) -> bool {
    if matches!(
        tag,
        "Teleport" | "Suspense" | "KeepAlive" | "BaseTransition" | "Transition" | "TransitionGroup"
    ) {
        return true;
    }
    if writer.custom_elements.matches(tag) {
        return false;
    }
    tag == "component"
        || !vize_l0::is_platform_tag(tag)
        || attrs.iter().any(|attr| {
            attr.name.text == "is"
                && attr
                    .value
                    .as_ref()
                    .is_some_and(|value| value.content.text.starts_with("vue:"))
        })
}

/// The decomposed directive head, or `None` for a plain attribute. A head
/// outside the syntax provider's admission is reported in `errors` and kept
/// as an attribute.
fn head(writer: &Writer<'_>, attr: &Attribute<'_>) -> Option<DirectiveName> {
    let head = VueDirectives
        .decompose(attr.name.text, writer.start(&attr.name))
        .ok()??;
    // `v-` with no name is a plain attribute after its diagnostic.
    (head.prefix != DirectivePrefix::Full || head.name.start < head.name.end).then_some(head)
}

/// Write the props of one opening tag. Inside `v-pre` every prop is a plain
/// attribute, except the `v-pre` directive on the element that opens it.
pub(super) fn write(
    writer: &mut Writer<'_>,
    attrs: &[Attribute<'_>],
    verbatim: bool,
    parent_verbatim: bool,
) {
    for (index, attr) in attrs.iter().enumerate() {
        if index > 0 {
            writer.json.raw(",");
        }
        let head = if parent_verbatim {
            None
        } else {
            head(writer, attr).filter(|head| {
                !verbatim
                    || (head.prefix == DirectivePrefix::Full
                        && slice(writer.source, head.name) == "pre")
            })
        };
        match head {
            Some(head) => directive(writer, attr, head),
            None => attribute(writer, attr),
        }
    }
}

/// The end of an attribute's extent: its closing quote, value, `=` or name.
fn attr_end(writer: &Writer<'_>, attr: &Attribute<'_>) -> u32 {
    match (&attr.value, &attr.eq) {
        (Some(value), _) => match &value.close_quote {
            Some(quote) => writer.end(quote),
            None => writer.end(&value.content),
        },
        (None, Some(eq)) => writer.end(eq),
        (None, None) => writer.end(&attr.name),
    }
}

fn attribute(writer: &mut Writer<'_>, attr: &Attribute<'_>) {
    let start = writer.cursor.at(writer.start(&attr.name));
    let end = attr_end(writer, attr);
    writer.json.raw("{\"type\":\"ATTRIBUTE\",\"name\":");
    writer.json.string(attr.name.text);
    writer.json.raw(",\"nameLoc\":");
    writer.loc_from(start, writer.end(&attr.name));
    writer.json.raw(",\"value\":");
    match &attr.value {
        Some(value) => {
            // The value's location includes its quotes, as in Vue.
            let value_start = match &value.open_quote {
                Some(quote) => writer.start(quote),
                None => writer.start(&value.content),
            };
            writer.json.raw("{\"type\":\"TEXT\",\"content\":");
            writer
                .json
                .decoded(value.content.text, EntityContext::Attribute);
            writer.json.raw(",\"loc\":");
            writer.loc(value_start, end);
            writer.json.raw("}");
        }
        None => writer.json.raw("null"),
    }
    writer.json.raw(",\"loc\":");
    writer.loc_from(start, end);
    writer.json.raw("}");
}

fn directive(writer: &mut Writer<'_>, attr: &Attribute<'_>, head: DirectiveName) {
    let name_start = writer.start(&attr.name);
    let start = writer.cursor.at(name_start);
    let end = attr_end(writer, attr);
    let name = match head.prefix {
        DirectivePrefix::Full => slice(writer.source, head.name),
        DirectivePrefix::Bind | DirectivePrefix::Prop => "bind",
        DirectivePrefix::On => "on",
        DirectivePrefix::Slot => "slot",
    };
    // `.name` is `v-bind:name.prop`; its implicit modifier is the `.` itself.
    let prop_modifier =
        (head.prefix == DirectivePrefix::Prop).then(|| writer.cursor.at(name_start + 1));
    writer.json.raw("{\"type\":\"DIRECTIVE\",\"name\":");
    writer.json.string(name);
    writer.json.raw(",\"rawName\":");
    writer.json.string(attr.name.text);
    writer.json.raw(",\"arg\":");
    match head.arg {
        Some(ArgSyntax::Static(span)) => {
            writer.expression(slice(writer.source, span), true, span.start, span.end, None);
        }
        Some(ArgSyntax::Dynamic(span)) => {
            // The location covers the brackets, as in Vue.
            let closed = writer
                .source
                .get(span.end as usize..)
                .is_some_and(|rest| rest.starts_with(']'));
            let content = slice(writer.source, span);
            let open = span.start.saturating_sub(1);
            writer.expression(content, false, open, span.end + u32::from(closed), None);
        }
        None => writer.json.raw("null"),
    }
    writer.json.raw(",\"modifiers\":[");
    let mut first = true;
    if let Some(dot_end) = prop_modifier {
        writer
            .json
            .raw("{\"type\":\"SIMPLE_EXPRESSION\",\"content\":\"prop\",\"isStatic\":true,\"loc\":");
        writer.loc_pair(start, dot_end);
        writer.json.raw("}");
        first = false;
    }
    let run = slice(writer.source, head.modifiers);
    let mut at = head.modifiers.start;
    for modifier in run.split('.').skip(1) {
        // Skip the `.` before this modifier.
        at += 1;
        if !first {
            writer.json.raw(",");
        }
        first = false;
        let modifier_end = at + modifier.len() as u32;
        writer.expression(modifier, true, at, modifier_end, None);
        at = modifier_end;
    }
    writer.json.raw("],\"exp\":");
    match &attr.value {
        Some(value) => writer.expression(
            value.content.text,
            false,
            writer.start(&value.content),
            writer.end(&value.content),
            Some(EntityContext::Attribute),
        ),
        None => writer.json.raw("null"),
    }
    writer.json.raw(",\"loc\":");
    writer.loc_from(start, end);
    writer.json.raw("}");
}

fn slice(source: &str, span: Span) -> &str {
    source
        .get(span.start as usize..span.end as usize)
        .unwrap_or_default()
}
