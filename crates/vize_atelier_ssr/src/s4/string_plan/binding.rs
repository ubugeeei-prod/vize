use vize_s0::Span;
use vize_s2::op as s2;

use super::{SsrStringPayload, SsrStringPayloadKind, SsrStringSegmentKind, payload::name_source};

pub(super) fn binding_kind(binding: &s2::BindingOp<'_>) -> SsrStringSegmentKind {
    match binding {
        s2::BindingOp::Bind(_) | s2::BindingOp::VueSync(_) | s2::BindingOp::Model(_) => {
            SsrStringSegmentKind::DynamicAttribute
        }
        s2::BindingOp::VueHtml(_) => SsrStringSegmentKind::RawHtml,
        s2::BindingOp::VueText(_) => SsrStringSegmentKind::DynamicText,
        s2::BindingOp::SlotContent(_) | s2::BindingOp::VueSlotScope(_) => {
            SsrStringSegmentKind::SlotOutlet
        }
        s2::BindingOp::On(_)
        | s2::BindingOp::VueCssBind(_)
        | s2::BindingOp::VueDirective(_)
        | s2::BindingOp::VueOnce(_)
        | s2::BindingOp::VueMemo(_)
        | s2::BindingOp::VueShow(_)
        | s2::BindingOp::VueCloak(_) => SsrStringSegmentKind::Directive,
    }
}

pub(super) fn binding_span(binding: &s2::BindingOp<'_>) -> Span {
    match binding {
        s2::BindingOp::Bind(op) => op.span,
        s2::BindingOp::On(op) => op.span,
        s2::BindingOp::Model(op) => op.span,
        s2::BindingOp::SlotContent(op) => op.span,
        s2::BindingOp::VueDirective(op) => op.span,
        s2::BindingOp::VueCssBind(op) => op.span,
        s2::BindingOp::VueSync(op) => op.span,
        s2::BindingOp::VueSlotScope(op) => op.span,
        s2::BindingOp::VueOnce(op) => op.span,
        s2::BindingOp::VueMemo(op) => op.span,
        s2::BindingOp::VueShow(op) => op.span,
        s2::BindingOp::VueHtml(op) => op.span,
        s2::BindingOp::VueText(op) => op.span,
        s2::BindingOp::VueCloak(op) => op.span,
    }
}

pub(super) fn binding_payload<'a>(binding: &s2::BindingOp<'a>) -> Option<SsrStringPayload<'a>> {
    match binding {
        s2::BindingOp::Bind(op) => op
            .value
            .map(|value| expression_payload(value.source()))
            .or_else(|| {
                op.name.and_then(name_source).map(|source| {
                    SsrStringPayload::new(SsrStringPayloadKind::AttributeName, source)
                })
            }),
        s2::BindingOp::On(op) => op
            .handler
            .map(|handler| directive_payload(handler.source()))
            .or_else(|| {
                op.name
                    .and_then(name_source)
                    .map(|source| SsrStringPayload::new(SsrStringPayloadKind::Directive, source))
            }),
        s2::BindingOp::Model(op) => Some(expression_payload(op.contract.read.source())),
        s2::BindingOp::SlotContent(op) => op
            .params
            .map(|params| expression_payload(params.source()))
            .or_else(|| op.name.and_then(name_source).map(slot_payload)),
        s2::BindingOp::VueDirective(op) => op
            .value
            .map(|value| expression_payload(value.source()))
            .or_else(|| op.argument.and_then(name_source).map(directive_payload))
            .or_else(|| Some(directive_payload(op.name))),
        s2::BindingOp::VueCssBind(op) => Some(expression_payload(op.value.source())),
        s2::BindingOp::VueSync(op) => Some(expression_payload(op.value.source())),
        s2::BindingOp::VueSlotScope(op) => op
            .params
            .map(|params| expression_payload(params.source()))
            .or_else(|| op.name.map(slot_payload)),
        s2::BindingOp::VueMemo(op) => Some(expression_payload(op.value.source())),
        s2::BindingOp::VueShow(op) => Some(expression_payload(op.value.source())),
        s2::BindingOp::VueHtml(op) => op.value.map(|value| expression_payload(value.source())),
        s2::BindingOp::VueText(op) => op.value.map(|value| expression_payload(value.source())),
        s2::BindingOp::VueOnce(_) | s2::BindingOp::VueCloak(_) => None,
    }
}

fn expression_payload(source: &str) -> SsrStringPayload<'_> {
    SsrStringPayload::new(SsrStringPayloadKind::Expression, source)
}

fn directive_payload(source: &str) -> SsrStringPayload<'_> {
    SsrStringPayload::new(SsrStringPayloadKind::Directive, source)
}

fn slot_payload(source: &str) -> SsrStringPayload<'_> {
    SsrStringPayload::new(SsrStringPayloadKind::SlotName, source)
}
