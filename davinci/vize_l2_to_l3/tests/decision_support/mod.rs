use vize_l0::{Allocator, Box, Span, Vec, id::NodeId};
use vize_l2::{
    artifact::{Artifact, ArtifactParts},
    expr::{ExprRef, OpaqueExpr, OpaqueReason},
    op::{
        BindOp, BindingContract, BindingOp, ComponentOp, DynamicName, ElementOp, InterpolationOp,
        ModelOp, Namespace, OnOp, Op, Region, SlotContentOp, TextOp, VueCloakOp, VueCssBindOp,
        VueDirectiveOp, VueHtmlOp, VueMemoOp, VueOnceOp, VueShowOp, VueSlotScopeOp, VueSyncOp,
        VueTextOp,
    },
    scope::{ScopeFacts, ScopeTag},
};

pub const SPAN: Span = Span::new(0, 1);

pub fn expression(allocator: &Allocator) -> ExprRef<'_> {
    ExprRef::Opaque(allocator.alloc(OpaqueExpr {
        reason: OpaqueReason::ParseRejected,
        source: "%",
        span: SPAN,
    }))
}

pub fn region<'a>(allocator: &'a Allocator, ops: impl IntoIterator<Item = Op<'a>>) -> Region<'a> {
    Region {
        ops: Vec::from_iter_in(ops, &allocator),
    }
}

pub fn seal<'a>(root: Region<'a>) -> Artifact<'a> {
    seal_scoped(root, [])
}

pub fn seal_scoped<'a>(
    root: Region<'a>,
    scope_nodes: impl IntoIterator<Item = NodeId>,
) -> Artifact<'a> {
    let scopes = scope_nodes
        .into_iter()
        .enumerate()
        .map(|(index, node)| {
            (
                node,
                ScopeFacts {
                    tag: ScopeTag::from_index(index as u32),
                    bindings: std::vec::Vec::new(),
                },
            )
        })
        .collect();
    Artifact::try_new(ArtifactParts {
        source: "%",
        root,
        provenance: std::vec::Vec::new(),
        scopes,
    })
    .expect("native fixture has valid spans and canonical controls")
}

pub fn element<'a>(
    allocator: &'a Allocator,
    bindings: impl IntoIterator<Item = BindingOp<'a>>,
    children: impl IntoIterator<Item = Op<'a>>,
) -> Op<'a> {
    Op::Element(Box::new_in(
        ElementOp {
            tag: "div",
            namespace: Namespace::Html,
            attributes: Vec::new_in(&allocator),
            bindings: Vec::from_iter_in(bindings, &allocator),
            children: region(allocator, children),
            span: SPAN,
        },
        &allocator,
    ))
}

pub fn component<'a>(
    allocator: &'a Allocator,
    bindings: impl IntoIterator<Item = BindingOp<'a>>,
) -> Op<'a> {
    Op::Component(Box::new_in(
        ComponentOp {
            name: "Widget",
            attributes: Vec::new_in(&allocator),
            bindings: Vec::from_iter_in(bindings, &allocator),
            children: region(allocator, []),
            span: SPAN,
        },
        &allocator,
    ))
}

pub fn text(allocator: &Allocator) -> Op<'_> {
    Op::Text(Box::new_in(
        TextOp {
            content: "text",
            span: SPAN,
        },
        &allocator,
    ))
}

pub fn interpolation(allocator: &Allocator) -> Op<'_> {
    Op::Interpolation(Box::new_in(
        InterpolationOp {
            expression: expression(allocator),
            span: SPAN,
        },
        &allocator,
    ))
}

pub fn event(allocator: &Allocator) -> BindingOp<'_> {
    BindingOp::On(Box::new_in(
        OnOp {
            name: Some(DynamicName::Static("click")),
            modifiers: Vec::new_in(&allocator),
            handler: Some(expression(allocator).into()),
            span: SPAN,
        },
        &allocator,
    ))
}

pub fn bind(allocator: &Allocator) -> BindingOp<'_> {
    BindingOp::Bind(Box::new_in(
        BindOp {
            name: Some(DynamicName::Static("id")),
            modifiers: Vec::new_in(&allocator),
            value: Some(expression(allocator)),
            span: SPAN,
        },
        &allocator,
    ))
}

pub fn cloak(allocator: &Allocator) -> BindingOp<'_> {
    BindingOp::VueCloak(Box::new_in(VueCloakOp { span: SPAN }, &allocator))
}

/// Real arena payloads for every attached op; no serialization or legacy producer.
pub fn every_binding(allocator: &Allocator) -> [BindingOp<'_>; 14] {
    let expr = expression(allocator);
    [
        bind(allocator),
        event(allocator),
        BindingOp::Model(Box::new_in(
            ModelOp {
                contract: BindingContract {
                    read: expr,
                    write: expr,
                },
                argument: None,
                attributes: Vec::new_in(&allocator),
                span: SPAN,
            },
            &allocator,
        )),
        BindingOp::SlotContent(Box::new_in(
            SlotContentOp {
                name: None,
                modifiers: Vec::new_in(&allocator),
                params: Some(expr),
                span: SPAN,
            },
            &allocator,
        )),
        BindingOp::VueDirective(Box::new_in(
            VueDirectiveOp {
                name: "custom",
                argument: None,
                modifiers: Vec::new_in(&allocator),
                value: Some(expr),
                span: SPAN,
            },
            &allocator,
        )),
        BindingOp::VueCssBind(Box::new_in(
            VueCssBindOp {
                value: expr,
                span: SPAN,
            },
            &allocator,
        )),
        BindingOp::VueSync(Box::new_in(
            VueSyncOp {
                name: "value",
                modifiers: Vec::new_in(&allocator),
                value: expr,
                span: SPAN,
            },
            &allocator,
        )),
        BindingOp::VueSlotScope(Box::new_in(
            VueSlotScopeOp {
                name: None,
                params: Some(expr),
                span: SPAN,
            },
            &allocator,
        )),
        BindingOp::VueOnce(Box::new_in(VueOnceOp { span: SPAN }, &allocator)),
        BindingOp::VueMemo(Box::new_in(
            VueMemoOp {
                value: expr,
                span: SPAN,
            },
            &allocator,
        )),
        BindingOp::VueShow(Box::new_in(
            VueShowOp {
                value: expr,
                span: SPAN,
            },
            &allocator,
        )),
        BindingOp::VueHtml(Box::new_in(
            VueHtmlOp {
                value: Some(expr),
                span: SPAN,
            },
            &allocator,
        )),
        BindingOp::VueText(Box::new_in(
            VueTextOp {
                value: Some(expr),
                span: SPAN,
            },
            &allocator,
        )),
        cloak(allocator),
    ]
}
