//! Operand schemas for native elements, text runs, and bindings.
//! Each schema is exact: missing, duplicate, or foreign operands never pass.

use vize_s3::{
    op::{OpId, OpKind},
    operand::{Operand, OperandRole as Role, ValueKind},
};

use super::super::{Binding, BindingKind, Content, Expr, TextPart};
use super::Result;
use crate::s3::{AdmissionFailure, LegacyReason, retained::Retained};

pub(super) fn element<'a>(values: &[Operand<'a>]) -> Result<Content<'a>> {
    if values.iter().any(|value| value.role == Role::Comment) {
        return Err(LegacyReason::Operation.into());
    }
    let tag = one(values, Role::Tag)?;
    let namespace = one(values, Role::Namespace)?;
    if tag.value.kind != ValueKind::Literal
        || namespace.value.kind != ValueKind::Literal
        || namespace.value.text != "html"
        // These HTML tags have ordinary template parsing. Parser-context
        // elements (tables, raw text, select, templates, namespaces) require a
        // separate contract before their child indexes can be materialized.
        // List items are admitted with the same nesting guard as buttons.
        // A `<template>` is admitted only as slot content (checked once its
        // slot binding attaches), and never carries attributes.
        || !matches!(tag.value.text,
            "div" | "span" | "main" | "section" | "article" | "header" | "footer"
            | "nav" | "aside" | "button" | "strong" | "em" | "b" | "i" | "small"
            | "label" | "input" | "img" | "br" | "hr" | "ul" | "ol" | "li" | "template")
        || tag.value.text == "template" && values.len() != 2
    {
        return Err(LegacyReason::Element.into());
    }
    let mut attributes = std::vec::Vec::new();
    for value in values {
        if value.target.is_some() || value.region.is_some() {
            return Err(LegacyReason::Structure.into());
        }
        match value.role {
            Role::Tag | Role::Namespace if value.name.is_none() => {}
            Role::Attribute => {
                let name = value.name.ok_or(LegacyReason::Binding)?;
                // The legacy parser reports repeats case-insensitively.
                if !attribute_name(name)
                    || attributes
                        .iter()
                        .any(|(seen, _): &(&str, _)| seen.eq_ignore_ascii_case(name))
                {
                    return Err(LegacyReason::Binding.into());
                }
                let text = match value.value.kind {
                    ValueKind::Absent => None,
                    ValueKind::Literal if !value.value.text.contains('&') => Some(value.value.text),
                    _ => return Err(LegacyReason::ExpressionOrEncoding.into()),
                };
                attributes.push((name, text));
            }
            _ => return Err(LegacyReason::Structure.into()),
        }
    }
    Ok(Content::Element {
        tag: tag.value.text,
        attributes,
    })
}

pub(super) fn binding<'a>(
    values: &[Operand<'a>],
    kind: OpKind,
    retained: &Retained<'_, 'a>,
) -> Result<(OpId, Binding<'a>)> {
    let binding = one(values, Role::BindingKind)?;
    // Generic ops carry several families (SetProp is also model/sync, a
    // Directive op also once/memo/cloak/custom). Select the family first.
    let family = match (kind, binding.value.kind, binding.value.text) {
        (OpKind::SetProp, ValueKind::Literal, "bind") => BindingKind::Prop,
        (OpKind::SetEvent, ValueKind::Literal, "on") => BindingKind::Event,
        (OpKind::Directive, ValueKind::Literal, "vue.show") => BindingKind::Show,
        (OpKind::SetHtml, ValueKind::Literal, "vue.html") => BindingKind::Html,
        (OpKind::SetText, ValueKind::Literal, "vue.text") => BindingKind::Text,
        (OpKind::Directive, ..) => return Err(LegacyReason::Operation.into()),
        _ => return Err(LegacyReason::Binding.into()),
    };
    let target = binding.target.ok_or(LegacyReason::Structure)?;
    if values
        .iter()
        .any(|v| v.target != Some(target) || v.region.is_some() || v.name.is_some())
    {
        return Err(LegacyReason::Binding.into());
    }
    let value = one(values, Role::Value)?;
    let (name, modifiers) = if matches!(family, BindingKind::Prop | BindingKind::Event) {
        named(values, family)?
    } else if values.len() == 2 {
        ("", std::vec::Vec::new())
    } else {
        return Err(LegacyReason::Binding.into());
    };
    let value = if family == BindingKind::Event {
        handler(retained, value)?
    } else {
        js(retained, value)?
    };
    Ok((
        target,
        Binding {
            kind: family,
            name,
            value,
            modifiers,
            merge: None,
        },
    ))
}

/// The `v-bind:name` / `v-on:name.modifiers` schema.
fn named<'a>(
    values: &[Operand<'a>],
    family: BindingKind,
) -> Result<(&'a str, std::vec::Vec<&'a str>)> {
    let event = family == BindingKind::Event;
    let name = one(values, Role::Name)?;
    let mut modifiers = std::vec::Vec::new();
    for value in values.iter().filter(|value| value.role == Role::Modifier) {
        if !event || value.value.kind != ValueKind::Literal || !event_name(value.value.text) {
            return Err(LegacyReason::Binding.into());
        }
        modifiers.push(value.value.text);
    }
    // `key` is admitted only as a loop key; its owner is checked once the
    // region tree is known.
    if values.len() != 3 + modifiers.len()
        || name.value.kind != ValueKind::Literal
        || if event {
            !event_name(name.value.text)
        } else {
            name.value.text != "key" && !attribute_name(name.value.text)
        }
    {
        return Err(LegacyReason::Binding.into());
    }
    Ok((name.value.text, modifiers))
}

pub(super) fn text<'a>(values: &[Operand<'a>], retained: &Retained<'_, 'a>) -> Result<Content<'a>> {
    if values.is_empty()
        || values.iter().any(|value| {
            value.role != Role::Text
                || value.target.is_some()
                || value.region.is_some()
                || value.name.is_some()
        })
        || values
            .windows(2)
            .any(|pair| pair[0].value.span.end != pair[1].value.span.start)
    {
        return Err(AdmissionFailure::Invalid("invalid native text run"));
    }
    let mut parts = std::vec::Vec::new();
    for operand in values {
        let value = operand.value;
        let (value, dynamic) = match value.kind {
            ValueKind::Literal if !value.text.is_empty() && !value.text.contains('&') => {
                (Expr::plain(value.text), false)
            }
            _ => (js(retained, operand)?, true),
        };
        parts.push(TextPart { value, dynamic });
    }
    let dynamic = parts.iter().any(|part| part.dynamic);
    Ok(Content::Text { parts, dynamic })
}

pub(super) fn one<'b, 'a>(values: &'b [Operand<'a>], role: Role) -> Result<&'b Operand<'a>> {
    let mut found = values.iter().filter(|v| v.role == role);
    let value = found
        .next()
        .ok_or(AdmissionFailure::Invalid("missing required native operand"))?;
    if found.next().is_some() {
        return Err(AdmissionFailure::Invalid("duplicate native operand role"));
    }
    Ok(value)
}

/// A JavaScript operand the generator can resolve without reparsing: a direct
/// reference, or an expression whose retained AST moved into the output arena.
pub(super) fn js<'a>(retained: &Retained<'_, 'a>, operand: &Operand<'a>) -> Result<Expr<'a>> {
    expression(retained, operand, false)
}

/// [`js`] for an event handler. A direct reference takes the generator's
/// simple-path fast path elsewhere, but a component handler is classified
/// first; its retained AST, when S2 has one, keeps that parse-free.
fn handler<'a>(retained: &Retained<'_, 'a>, operand: &Operand<'a>) -> Result<Expr<'a>> {
    expression(retained, operand, true)
}

fn expression<'a>(
    retained: &Retained<'_, 'a>,
    operand: &Operand<'a>,
    classified: bool,
) -> Result<Expr<'a>> {
    let value = operand.value;
    if value.kind != ValueKind::Js || context_reserved(value.text) {
        return Err(LegacyReason::ExpressionOrEncoding.into());
    }
    if reference(value.text) {
        return Ok(
            match classified
                .then(|| retained.expression(value.text, value.span))
                .flatten()
            {
                Some(js) => Expr {
                    text: value.text,
                    js: Some(js),
                },
                None => Expr::plain(value.text.trim()),
            },
        );
    }
    // `$event`-rooted paths stay on the legacy lane (see the P3-6 record).
    let root = value.text.trim().split('.').next().unwrap_or_default();
    if root == "$event" && value.text.trim().split('.').all(identifier_segment) {
        return Err(LegacyReason::ExpressionOrEncoding.into());
    }
    let js = retained
        .expression(value.text, value.span)
        .ok_or(LegacyReason::ExpressionOrEncoding)?;
    Ok(Expr {
        text: value.text,
        js: Some(js),
    })
}

/// The shared generator leaves these roots bare while the retained lane's
/// prefixing rewrites them onto `_ctx`; the lanes would observe different
/// bindings. The textual test over-approximates (it also matches strings).
fn context_reserved(text: &str) -> bool {
    // Every reserved root starts with `_` or `$`; most expressions have none.
    text.bytes().any(|b| b == b'_' || b == b'$')
        && ["_ctx", "$props", "$attrs", "$slots", "$emit"]
            .iter()
            .any(|name| text.contains(name))
}

fn identifier_segment(part: &str) -> bool {
    let bytes = part.as_bytes();
    bytes
        .first()
        .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'_' || *b == b'$')
        && bytes
            .iter()
            .all(|b| b.is_ascii_alphanumeric() || *b == b'_' || *b == b'$')
}

fn event_name(name: &str) -> bool {
    name.starts_with(|c: char| c.is_ascii_alphabetic())
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | ':'))
}

fn attribute_name(name: &str) -> bool {
    !matches!(
        name,
        "key" | "ref" | "ref_for" | "ref_key" | "is" | "innerHTML" | "textContent"
    ) && !name.starts_with("on")
        && name.starts_with(|c: char| c.is_ascii_alphabetic())
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

pub(in crate::s3) fn reference(value: &str) -> bool {
    // A deliberately narrower grammar than JavaScript. The S3 producer has
    // already classified it as JS; no reparsing or opaque reinterpretation.
    let mut segments = value.trim().split('.');
    let root = segments.next().unwrap_or_default();
    identifier_segment(root)
        && segments.all(identifier_segment)
        && root != "$event"
        && !oxc_syntax::keyword::is_reserved_keyword(root)
}
