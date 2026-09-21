//! Operand schemas for native elements, text runs, and bindings.
//! Each schema is exact: missing, duplicate, or foreign operands never pass.

use vize_carton::FxHashSet;
use vize_s3::{
    op::{OpId, OpKind},
    operand::{Operand, OperandRole as Role, ValueKind},
};

use super::super::{Binding, Content, TextPart};
use super::Result;
use crate::s3::{AdmissionFailure, LegacyReason};

pub(super) fn element<'a>(values: &[&Operand<'a>]) -> Result<Content<'a>> {
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
        || !matches!(tag.value.text,
            "div" | "span" | "main" | "section" | "article" | "header" | "footer"
            | "nav" | "aside" | "button" | "strong" | "em" | "b" | "i" | "small"
            | "label" | "input" | "img" | "br" | "hr" | "ul" | "ol" | "li")
    {
        return Err(LegacyReason::Element.into());
    }
    let mut attributes = std::vec::Vec::new();
    let mut names = FxHashSet::default();
    for value in values {
        if value.target.is_some() || value.region.is_some() {
            return Err(LegacyReason::Structure.into());
        }
        match value.role {
            Role::Tag | Role::Namespace if value.name.is_none() => {}
            Role::Attribute => {
                let name = value.name.ok_or(LegacyReason::Binding)?;
                if !attribute_name(name) || !names.insert(name) {
                    return Err(LegacyReason::Binding.into());
                }
                let text = match value.value.kind {
                    ValueKind::Absent => None,
                    ValueKind::Literal if !value.value.text.contains('&') => Some(value.value.text),
                    _ => return Err(LegacyReason::ExpressionOrEncoding.into()),
                };
                attributes.push((name, text, (value.value.span.start, value.value.span.end)));
            }
            _ => return Err(LegacyReason::Structure.into()),
        }
    }
    Ok(Content::Element {
        tag: tag.value.text,
        tag_span: (tag.value.span.start, tag.value.span.end),
        attributes,
    })
}

pub(super) fn binding<'a>(values: &[&Operand<'a>], kind: OpKind) -> Result<(OpId, Binding<'a>)> {
    let binding = one(values, Role::BindingKind)?;
    let event = kind == OpKind::SetEvent;
    // SetProp is also the generic model/sync op. Select its semantic family
    // before requiring the narrower bind/on operand schema.
    if binding.value.kind != ValueKind::Literal
        || binding.value.text != if event { "on" } else { "bind" }
    {
        return Err(LegacyReason::Binding.into());
    }
    let name = one(values, Role::Name)?;
    let value = one(values, Role::Value)?;
    let mut modifiers = std::vec::Vec::new();
    for value in values.iter().filter(|value| value.role == Role::Modifier) {
        if !event || value.value.kind != ValueKind::Literal || !event_name(value.value.text) {
            return Err(LegacyReason::Binding.into());
        }
        modifiers.push(value.value.text);
    }
    if values.len() != 3 + modifiers.len() {
        return Err(LegacyReason::Binding.into());
    }
    let target = binding.target.ok_or(LegacyReason::Structure)?;
    // `key` is admitted only as a loop key; its owner is checked once the
    // region tree is known.
    if values
        .iter()
        .any(|v| v.target != Some(target) || v.region.is_some() || v.name.is_some())
        || name.value.kind != ValueKind::Literal
        || if event {
            !event_name(name.value.text)
        } else {
            name.value.text != "key" && !attribute_name(name.value.text)
        }
    {
        return Err(LegacyReason::Binding.into());
    }
    if value.value.kind != ValueKind::Js || !reference(value.value.text) {
        return Err(LegacyReason::ExpressionOrEncoding.into());
    }
    Ok((
        target,
        Binding {
            name: name.value.text,
            value: value.value.text.trim(),
            event,
            modifiers,
            spans: [name.value.span, value.value.span].map(|span| (span.start, span.end)),
        },
    ))
}

pub(super) fn text<'a>(values: &[&Operand<'a>]) -> Result<Content<'a>> {
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
        let dynamic = match value.kind {
            ValueKind::Literal if !value.text.is_empty() && !value.text.contains('&') => false,
            ValueKind::Js if reference(value.text) => true,
            _ => return Err(LegacyReason::ExpressionOrEncoding.into()),
        };
        parts.push(TextPart {
            value: value.text,
            dynamic,
            span: (value.span.start, value.span.end),
        });
    }
    let dynamic = parts.iter().any(|part| part.dynamic);
    Ok(Content::Text { parts, dynamic })
}

pub(super) fn one<'b, 'a>(values: &'b [&Operand<'a>], role: Role) -> Result<&'b Operand<'a>> {
    let mut found = values.iter().filter(|v| v.role == role);
    let value = found
        .next()
        .ok_or(AdmissionFailure::Invalid("missing required native operand"))?;
    if found.next().is_some() {
        return Err(AdmissionFailure::Invalid("duplicate native operand role"));
    }
    Ok(value)
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
    let value = value.trim();
    let root = value.split('.').next().unwrap_or_default();
    root != "$event"
        && !oxc_syntax::keyword::is_reserved_keyword(root)
        && value.split('.').all(|part| {
            part.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_' || c == '$')
                && part
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
        })
}
