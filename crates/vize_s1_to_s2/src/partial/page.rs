//! Owned kept fragments and the S1 holes they sit beside.
//!
//! The lowering already kept the ops. This walk only *names* them, in the
//! page-order numbering [`crate::pass::walk`] owns, so a fact producer can
//! run after the arena is gone.

use alloc::vec::Vec;

use vize_davinci::id::NodeId;
use vize_s0::{Span, String};
use vize_s1::{Element, ElementClose, SurfaceChild, SurfaceTree, Token};
use vize_s2::op::{BindingOp, Op};
use vize_s2::scope::ScopeOrigin;

use crate::lower::Lowered;
use crate::pass::walk::{PageWalk, visit_ops};

use super::{FragmentKind, HoleFact, HoleKind};

pub(crate) struct KeptFragment {
    pub node: u32,
    pub span: Span,
    pub kind: FragmentKind,
    /// Where names introduced here are visible. A binding's own span is the
    /// attribute; its names are visible across the owner element.
    pub visible: Span,
}

pub(crate) struct KeptName {
    pub node: u32,
    pub index: u16,
    pub name: String,
    pub span: Span,
}

pub(crate) struct PartialPage {
    pub holes: Vec<HoleFact>,
    pub fragments: Vec<KeptFragment>,
    pub names: Vec<KeptName>,
}

pub(crate) fn collect(lowered: &mut Lowered<'_>, tree: &SurfaceTree<'_>) -> PartialPage {
    debug_assert!(
        tree.source.as_ptr() >= lowered.source.as_ptr()
            && tree.source.as_ptr() as usize + tree.source.len()
                <= lowered.source.as_ptr() as usize + lowered.source.len(),
        "hole spans are measured against the lowering's source"
    );
    let holes = collect_holes(lowered.source, tree);
    let mut fragments = Vec::new();
    let mut walk = PageWalk::new();
    visit_ops(&mut walk, &mut lowered.root.ops[..], &mut |id, op| {
        note(&mut fragments, id, op);
    });
    let names = scope_names(lowered);
    PartialPage {
        holes,
        fragments,
        names,
    }
}

fn scope_names(lowered: &Lowered<'_>) -> Vec<KeptName> {
    let mut names = Vec::new();
    for (id, facts) in lowered.scopes.iter() {
        for (index, binding) in facts.bindings.iter().enumerate() {
            let span = match binding.origin {
                ScopeOrigin::Authored { span } => span,
                ScopeOrigin::Synthesized { .. } => Span::new(0, 0),
            };
            names.push(KeptName {
                node: id.index(),
                index: u16::try_from(index).unwrap_or(u16::MAX),
                name: String::from(binding.name.as_str()),
                span,
            });
        }
    }
    names
}

fn note(out: &mut Vec<KeptFragment>, id: Option<NodeId>, op: &Op<'_>) {
    let Some(id) = id else {
        return;
    };
    let (kind, span, bindings) = op_surface(op);
    out.push(KeptFragment {
        node: id.index(),
        span,
        kind,
        visible: span,
    });
    let Some(bindings) = bindings else {
        return;
    };
    let mut index = id.index();
    for binding in bindings {
        index = index.saturating_add(1);
        let Some(node) = NodeId::from_index(index) else {
            return;
        };
        out.push(KeptFragment {
            node: node.index(),
            span: binding_span(binding),
            kind: FragmentKind::Binding,
            visible: span,
        });
    }
}

fn op_surface<'a>(op: &'a Op<'a>) -> (FragmentKind, Span, Option<&'a [BindingOp<'a>]>) {
    match op {
        Op::Element(op) => (FragmentKind::Element, op.span, Some(&op.bindings[..])),
        Op::Component(op) => (FragmentKind::Component, op.span, Some(&op.bindings[..])),
        Op::Text(op) => (FragmentKind::Text, op.span, None),
        Op::Interpolation(op) => (FragmentKind::Interpolation, op.span, None),
        Op::Comment(op) => (FragmentKind::Comment, op.span, None),
        Op::If(op) => (FragmentKind::If, op.span, None),
        Op::For(op) => (FragmentKind::For, op.span, None),
        Op::Slot(op) => (FragmentKind::Slot, op.span, Some(&op.bindings[..])),
    }
}

fn binding_span(binding: &BindingOp<'_>) -> Span {
    match binding {
        BindingOp::Bind(op) => op.span,
        BindingOp::On(op) => op.span,
        BindingOp::Model(op) => op.span,
        BindingOp::SlotContent(op) => op.span,
        BindingOp::VueDirective(op) => op.span,
        BindingOp::VueCssBind(op) => op.span,
        BindingOp::VueSync(op) => op.span,
        BindingOp::VueSlotScope(op) => op.span,
        BindingOp::VueOnce(op) => op.span,
        BindingOp::VueMemo(op) => op.span,
        BindingOp::VueShow(op) => op.span,
        BindingOp::VueHtml(op) => op.span,
        BindingOp::VueText(op) => op.span,
        BindingOp::VueCloak(op) => op.span,
    }
}

fn collect_holes(source: &str, tree: &SurfaceTree<'_>) -> Vec<HoleFact> {
    let mut holes = Vec::new();
    walk_children(source, &tree.children, &mut holes);
    holes
}

fn walk_children(source: &str, children: &[SurfaceChild<'_>], holes: &mut Vec<HoleFact>) {
    for child in children {
        match child {
            SurfaceChild::Element(element) => walk_element(source, element, holes),
            SurfaceChild::Interpolation(node) => {
                record_token(source, &node.open, holes);
                record_token(source, &node.content, holes);
                record_token(source, &node.close, holes);
            }
            SurfaceChild::Unexpected(token) => {
                let span = token_span(source, token);
                if !span.is_empty() {
                    holes.push(HoleFact {
                        kind: HoleKind::Unexpected,
                        span,
                    });
                }
                record_token(source, token, holes);
            }
            SurfaceChild::Text(token)
            | SurfaceChild::Comment(token)
            | SurfaceChild::Cdata(token)
            | SurfaceChild::ProcessingInstruction(token) => record_token(source, token, holes),
        }
    }
}

fn walk_element(source: &str, element: &Element<'_>, holes: &mut Vec<HoleFact>) {
    record_token(source, &element.open.lt_name, holes);
    for attr in &element.open.attrs {
        record_token(source, &attr.name, holes);
        if let Some(eq) = &attr.eq {
            record_token(source, eq, holes);
        }
        if let Some(value) = &attr.value {
            if let Some(quote) = &value.open_quote {
                record_token(source, quote, holes);
            }
            record_token(source, &value.content, holes);
            if let Some(quote) = &value.close_quote {
                record_token(source, quote, holes);
            }
        }
    }
    if let Some(slash) = &element.open.slash {
        record_token(source, slash, holes);
    }
    record_token(source, &element.open.gt, holes);
    walk_children(source, &element.children, holes);
    match &element.close {
        ElementClose::Present(close) => {
            record_token(source, &close.lt_slash_name, holes);
            record_token(source, &close.gt, holes);
        }
        ElementClose::Missing => {
            let end = token_span(source, &element.open.gt).end;
            holes.push(HoleFact {
                kind: HoleKind::Missing,
                span: Span::new(end, end),
            });
        }
        ElementClose::Implicit | ElementClose::NotExpected => {}
    }
}

fn record_token(source: &str, token: &Token<'_>, holes: &mut Vec<HoleFact>) {
    if token.is_missing() {
        holes.push(HoleFact {
            kind: HoleKind::Missing,
            span: token_span(source, token),
        });
    }
}

fn token_span(source: &str, token: &Token<'_>) -> Span {
    let base = source.as_ptr() as usize;
    let ptr = token.text.as_ptr() as usize;
    let end_bound = base.saturating_add(source.len());
    if ptr < base || ptr > end_bound {
        return Span::new(0, 0);
    }
    let start = u32::try_from(ptr - base).unwrap_or(0);
    let len = u32::try_from(token.text.len()).unwrap_or(0);
    Span::new(start, start.saturating_add(len))
}
