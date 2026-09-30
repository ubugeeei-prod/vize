use crate::PartitionKind;
use vize_l0::Span;
use vize_l2::op as l2;
use vize_l3::op::OpKind;

pub(super) fn binding_kind(binding: &l2::BindingOp<'_>) -> OpKind {
    match binding {
        l2::BindingOp::Bind(bind) if bind.name.is_none() => OpKind::SetDynamicProps,
        l2::BindingOp::Bind(_) => OpKind::SetProp,
        l2::BindingOp::On(_) => OpKind::SetEvent,
        l2::BindingOp::Model(_) | l2::BindingOp::VueSync(_) => OpKind::SetProp,
        l2::BindingOp::SlotContent(_) | l2::BindingOp::VueSlotScope(_) => OpKind::SlotOutlet,
        l2::BindingOp::VueDirective(_)
        | l2::BindingOp::VueOnce(_)
        | l2::BindingOp::VueMemo(_)
        | l2::BindingOp::VueShow(_)
        | l2::BindingOp::VueCloak(_) => OpKind::Directive,
        l2::BindingOp::VueCssBind(_) => OpKind::SetDynamicProps,
        l2::BindingOp::VueHtml(_) => OpKind::SetHtml,
        l2::BindingOp::VueText(_) => OpKind::SetText,
    }
}

pub(super) fn binding_partition(binding: &l2::BindingOp<'_>) -> PartitionKind {
    match binding {
        l2::BindingOp::VueOnce(_) | l2::BindingOp::VueCloak(_) => PartitionKind::Static,
        _ => PartitionKind::Dynamic,
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

pub(super) fn region_span(region: &l2::Region<'_>, fallback: Span) -> Span {
    let mut span: Option<Span> = None;
    for op in &region.ops {
        let next = match op {
            l2::Op::Element(op) => op.span,
            l2::Op::Component(op) => op.span,
            l2::Op::Text(op) => op.span,
            l2::Op::Interpolation(op) => op.span,
            l2::Op::Comment(op) => op.span,
            l2::Op::If(op) => op.span,
            l2::Op::For(op) => op.span,
            l2::Op::Slot(op) => op.span,
        };
        span = Some(match span {
            Some(current) => Span::new(current.start.min(next.start), current.end.max(next.end)),
            None => next,
        });
    }
    span.unwrap_or(fallback)
}
