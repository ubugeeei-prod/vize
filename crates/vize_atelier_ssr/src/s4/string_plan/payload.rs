use vize_s2::op as s2;

/// Borrowed SSR payload carried by an S4 string-plan segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SsrStringPayload<'a> {
    pub kind: SsrStringPayloadKind,
    pub source: &'a str,
}

/// The meaning of a borrowed payload before SSR code generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SsrStringPayloadKind {
    TagName,
    AttributeName,
    ComponentName,
    Text,
    Expression,
    Comment,
    ForBinding,
    SlotName,
    Directive,
}

impl<'a> SsrStringPayload<'a> {
    pub const fn new(kind: SsrStringPayloadKind, source: &'a str) -> Self {
        Self { kind, source }
    }
}

pub(super) fn slot_name_payload<'a>(name: &s2::DynamicName<'a>) -> Option<SsrStringPayload<'a>> {
    name_source(*name).map(|source| SsrStringPayload::new(SsrStringPayloadKind::SlotName, source))
}

pub(super) fn name_source(name: s2::DynamicName<'_>) -> Option<&str> {
    match name {
        s2::DynamicName::Static(name) => Some(name),
        s2::DynamicName::Dynamic(expr) => Some(expr.source()),
    }
}
