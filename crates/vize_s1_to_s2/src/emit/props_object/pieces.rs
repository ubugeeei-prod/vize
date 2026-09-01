use alloc::vec::Vec as StdVec;

use vize_s0::Span;
use vize_s2::op::{Attribute, BindOp, BindingOp, DynamicName, ModelOp, OnOp, VueHtmlOp, VueTextOp};

use super::super::error::UnsupportedReason as Reason;
use super::super::model_key::{ModelModifiersKey, ModelName, ModelUpdateKey};
use super::super::{EmitError, model};

pub(in crate::emit) enum Piece<'a> {
    Attr(&'a Attribute<'a>),
    Bind(&'a BindOp<'a>),
    On(&'a OnOp<'a>),
    VueHtml(&'a VueHtmlOp<'a>),
    VueText(&'a VueTextOp<'a>),
    ModelValue {
        name: ModelName<'a>,
        model: &'a ModelOp<'a>,
        span: Span,
    },
    ModelUpdate {
        key: ModelUpdateKey<'a>,
        model: &'a ModelOp<'a>,
        span: Span,
    },
    ModelModifiers {
        name: ModelModifiersKey<'a>,
        modifiers: StdVec<&'a str>,
        span: Span,
    },
}

impl Piece<'_> {
    pub(in crate::emit) fn span(&self) -> Span {
        match self {
            Self::Attr(attr) => attr.span,
            Self::Bind(bind) => bind.span,
            Self::On(on) => on.span,
            Self::VueHtml(html) => html.span,
            Self::VueText(text) => text.span,
            Self::ModelValue { span, .. }
            | Self::ModelUpdate { span, .. }
            | Self::ModelModifiers { span, .. } => *span,
        }
    }
}

pub(in crate::emit) fn pieces<'a>(
    attributes: &'a [Attribute<'a>],
    bindings: &'a [BindingOp<'a>],
    skip_is: bool,
) -> Result<StdVec<Piece<'a>>, EmitError> {
    let mut out = StdVec::new();
    for attr in attributes.iter() {
        if skip_is && attr.name == "is" {
            continue;
        }
        out.push(Piece::Attr(attr));
    }
    for binding in bindings.iter() {
        match binding {
            BindingOp::Bind(bind)
                if skip_is && matches!(bind.name, Some(DynamicName::Static("is"))) => {}
            BindingOp::Bind(bind) => out.push(Piece::Bind(bind)),
            BindingOp::On(on) => out.push(Piece::On(on)),
            BindingOp::Model(model_op) => model::expand(model_op, &mut out)?,
            BindingOp::VueHtml(html) => out.push(Piece::VueHtml(html)),
            BindingOp::VueText(text) => out.push(Piece::VueText(text)),
            BindingOp::SlotContent(_) => {}
            BindingOp::VueDirective(_) => {}
            BindingOp::VueOnce(_) => {}
            BindingOp::VueMemo(_) => {}
            BindingOp::VueShow(_) => {}
            BindingOp::VueCloak(_) => {}
            _ => {
                return Err(EmitError::unsupported_binding(
                    Reason::UnsupportedBindingKind,
                    binding,
                ));
            }
        }
    }
    out.sort_by_key(|piece| piece.span().start);
    Ok(out)
}
