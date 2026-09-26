use vize_l0::Span;
use vize_l2::op as l2;

use super::{SsrStringPayload, SsrStringPayloadKind, SsrStringSegmentKind, payload::name_source};

pub(super) fn binding_kind(binding: &l2::BindingOp<'_>) -> SsrStringSegmentKind {
    match binding {
        l2::BindingOp::Bind(_) | l2::BindingOp::VueSync(_) | l2::BindingOp::Model(_) => {
            SsrStringSegmentKind::DynamicAttribute
        }
        l2::BindingOp::VueHtml(_) => SsrStringSegmentKind::RawHtml,
        l2::BindingOp::VueText(_) => SsrStringSegmentKind::DynamicText,
        l2::BindingOp::SlotContent(_) | l2::BindingOp::VueSlotScope(_) => {
            SsrStringSegmentKind::SlotContent
        }
        l2::BindingOp::On(_)
        | l2::BindingOp::VueCssBind(_)
        | l2::BindingOp::VueDirective(_)
        | l2::BindingOp::VueOnce(_)
        | l2::BindingOp::VueMemo(_)
        | l2::BindingOp::VueShow(_)
        | l2::BindingOp::VueCloak(_) => SsrStringSegmentKind::Directive,
    }
}

pub(super) fn binding_span(binding: &l2::BindingOp<'_>) -> Span {
    match binding {
        l2::BindingOp::Bind(op) => op.span,
        l2::BindingOp::On(op) => op.span,
        l2::BindingOp::Model(op) => op.span,
        l2::BindingOp::SlotContent(op) => op.span,
        l2::BindingOp::VueDirective(op) => op.span,
        l2::BindingOp::VueCssBind(op) => op.span,
        l2::BindingOp::VueSync(op) => op.span,
        l2::BindingOp::VueSlotScope(op) => op.span,
        l2::BindingOp::VueOnce(op) => op.span,
        l2::BindingOp::VueMemo(op) => op.span,
        l2::BindingOp::VueShow(op) => op.span,
        l2::BindingOp::VueHtml(op) => op.span,
        l2::BindingOp::VueText(op) => op.span,
        l2::BindingOp::VueCloak(op) => op.span,
    }
}

pub(super) fn binding_payload<'a>(binding: &l2::BindingOp<'a>) -> Option<SsrStringPayload<'a>> {
    match binding {
        l2::BindingOp::Bind(op) => op
            .value
            .map(|value| expression_payload(value.source()))
            .or_else(|| {
                op.name.and_then(name_source).map(|source| {
                    SsrStringPayload::new(SsrStringPayloadKind::AttributeName, source)
                })
            }),
        l2::BindingOp::On(op) => op
            .handler
            .map(|handler| directive_payload(handler.source()))
            .or_else(|| {
                op.name
                    .and_then(name_source)
                    .map(|source| SsrStringPayload::new(SsrStringPayloadKind::Directive, source))
            }),
        l2::BindingOp::Model(op) => Some(expression_payload(op.contract.read.source())),
        l2::BindingOp::SlotContent(op) => op
            .params
            .map(|params| expression_payload(params.source()))
            .or_else(|| op.name.and_then(name_source).map(slot_payload)),
        l2::BindingOp::VueDirective(op) => op
            .value
            .map(|value| expression_payload(value.source()))
            .or_else(|| op.argument.and_then(name_source).map(directive_payload))
            .or_else(|| Some(directive_payload(op.name))),
        l2::BindingOp::VueCssBind(op) => Some(expression_payload(op.value.source())),
        l2::BindingOp::VueSync(op) => Some(expression_payload(op.value.source())),
        l2::BindingOp::VueSlotScope(op) => op
            .params
            .map(|params| expression_payload(params.source()))
            .or_else(|| op.name.map(slot_payload)),
        l2::BindingOp::VueMemo(op) => Some(expression_payload(op.value.source())),
        l2::BindingOp::VueShow(op) => Some(expression_payload(op.value.source())),
        l2::BindingOp::VueHtml(op) => op.value.map(|value| expression_payload(value.source())),
        l2::BindingOp::VueText(op) => op.value.map(|value| expression_payload(value.source())),
        l2::BindingOp::VueOnce(_) | l2::BindingOp::VueCloak(_) => None,
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
