use vize_s0::Span;
use vize_s2::op as s2;

use super::SsrStringSegmentKind;

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

pub(super) fn binding_source<'a>(binding: &s2::BindingOp<'a>) -> Option<&'a str> {
    match binding {
        s2::BindingOp::Bind(op) => op
            .value
            .map(|value| value.source())
            .or_else(|| op.name.and_then(name_source)),
        s2::BindingOp::On(op) => op
            .handler
            .map(|handler| handler.source())
            .or_else(|| op.name.and_then(name_source)),
        s2::BindingOp::Model(op) => Some(op.contract.read.source()),
        s2::BindingOp::SlotContent(op) => op
            .params
            .map(|params| params.source())
            .or_else(|| op.name.and_then(name_source)),
        s2::BindingOp::VueDirective(op) => op
            .value
            .map(|value| value.source())
            .or_else(|| op.argument.and_then(name_source))
            .or(Some(op.name)),
        s2::BindingOp::VueCssBind(op) => Some(op.value.source()),
        s2::BindingOp::VueSync(op) => Some(op.value.source()),
        s2::BindingOp::VueSlotScope(op) => op.params.map(|params| params.source()).or(op.name),
        s2::BindingOp::VueMemo(op) => Some(op.value.source()),
        s2::BindingOp::VueShow(op) => Some(op.value.source()),
        s2::BindingOp::VueHtml(op) => op.value.map(|value| value.source()),
        s2::BindingOp::VueText(op) => op.value.map(|value| value.source()),
        s2::BindingOp::VueOnce(_) | s2::BindingOp::VueCloak(_) => None,
    }
}

pub(super) fn slot_name<'a>(name: &s2::DynamicName<'a>) -> Option<&'a str> {
    name_source(*name)
}

fn name_source(name: s2::DynamicName<'_>) -> Option<&str> {
    match name {
        s2::DynamicName::Static(name) => Some(name),
        s2::DynamicName::Dynamic(expr) => Some(expr.source()),
    }
}
