use vize_l0::{Allocator, Box, Span, Vec};
use vize_l2::{
    artifact::{Artifact, ArtifactParts},
    expr::ExprRef,
    op::{
        BindOp, BindingOp, CommentOp, DynamicName, ElementOp, InterpolationOp, Namespace, Op,
        Region, TextOp,
    },
};

pub const SOURCE: &str = "x + y 1 'a' id class style";
pub const SPAN: Span = Span::new(0, SOURCE.len() as u32);

pub fn expression<'a>(allocator: &'a Allocator, text: &'a str) -> ExprRef<'a> {
    let start = SOURCE
        .find(text)
        .expect("expression exists in fixture source") as u32;
    ExprRef::parse_js_in(allocator, text, Span::new(start, start + text.len() as u32))
}

pub fn region<'a>(allocator: &'a Allocator, ops: impl IntoIterator<Item = Op<'a>>) -> Region<'a> {
    Region {
        ops: Vec::from_iter_in(ops, &allocator),
    }
}

pub fn seal(root: Region<'_>) -> Artifact<'_> {
    Artifact::try_new(ArtifactParts {
        source: SOURCE,
        root,
        scopes: Default::default(),
        provenance: Default::default(),
    })
    .expect("canonical DOM fixture")
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

pub fn text(allocator: &Allocator) -> Op<'_> {
    Op::Text(Box::new_in(
        TextOp {
            content: "hello",
            span: SPAN,
        },
        &allocator,
    ))
}

pub fn interpolation<'a>(allocator: &'a Allocator, value: ExprRef<'a>) -> Op<'a> {
    Op::Interpolation(Box::new_in(
        InterpolationOp {
            expression: value,
            span: SPAN,
        },
        &allocator,
    ))
}

pub fn comment(allocator: &Allocator) -> Op<'_> {
    Op::Comment(Box::new_in(
        CommentOp {
            content: "comment",
            span: SPAN,
        },
        &allocator,
    ))
}

pub fn bind<'a>(allocator: &'a Allocator, name: &'a str, value: ExprRef<'a>) -> BindingOp<'a> {
    BindingOp::Bind(Box::new_in(
        BindOp {
            name: Some(DynamicName::Static(name)),
            modifiers: Vec::new_in(&allocator),
            value: Some(value),
            span: SPAN,
        },
        &allocator,
    ))
}
