use crate::PartitionKind;
use vize_s0::Span;
use vize_s2::op as s2;
use vize_s3::op::OpKind;

pub(super) fn binding_kind(binding: &s2::BindingOp<'_>) -> OpKind {
    match binding {
        s2::BindingOp::Bind(bind) if bind.name.is_none() => OpKind::SetDynamicProps,
        s2::BindingOp::Bind(_) => OpKind::SetProp,
        s2::BindingOp::On(_) => OpKind::SetEvent,
        s2::BindingOp::Model(_) | s2::BindingOp::VueSync(_) => OpKind::SetProp,
        s2::BindingOp::SlotContent(_) | s2::BindingOp::VueSlotScope(_) => OpKind::SlotOutlet,
        s2::BindingOp::VueDirective(_)
        | s2::BindingOp::VueOnce(_)
        | s2::BindingOp::VueMemo(_)
        | s2::BindingOp::VueShow(_)
        | s2::BindingOp::VueCloak(_) => OpKind::Directive,
        s2::BindingOp::VueCssBind(_) => OpKind::SetDynamicProps,
        s2::BindingOp::VueHtml(_) => OpKind::SetHtml,
        s2::BindingOp::VueText(_) => OpKind::SetText,
    }
}

pub(super) fn binding_partition(binding: &s2::BindingOp<'_>) -> PartitionKind {
    match binding {
        s2::BindingOp::VueOnce(_) | s2::BindingOp::VueCloak(_) => PartitionKind::Static,
        _ => PartitionKind::Dynamic,
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

pub(super) fn region_span(region: &s2::Region<'_>, fallback: Span) -> Span {
    let mut span: Option<Span> = None;
    for op in &region.ops {
        let next = match op {
            s2::Op::Element(op) => op.span,
            s2::Op::Component(op) => op.span,
            s2::Op::Text(op) => op.span,
            s2::Op::Interpolation(op) => op.span,
            s2::Op::Comment(op) => op.span,
            s2::Op::If(op) => op.span,
            s2::Op::For(op) => op.span,
            s2::Op::Slot(op) => op.span,
        };
        span = Some(match span {
            Some(current) => Span::new(current.start.min(next.start), current.end.max(next.end)),
            None => next,
        });
    }
    span.unwrap_or(fallback)
}
