//! Native arena owners for the independently captured Vue reference inputs.

use alloc::vec::Vec as OwnedVec;
use vize_l0::{Allocator, Box, Span, String, Vec, id::NodeId, side_table::SideTable};
use vize_l2::artifact::{Artifact, ArtifactParts};
use vize_l2::expr::{ExprRef, JsExpr};
use vize_l2::op::{
    Attribute, BindOp, BindingOp, CommentOp, DynamicName, ElementOp, InterpolationOp, Namespace,
    OnOp, Op, Region, TextOp,
};
use vize_l2::provenance::ProvenanceRecord;

mod extra;

fn region<'a>(arena: &'a Allocator, ops: impl IntoIterator<Item = Op<'a>>) -> Region<'a> {
    let mut result = Region {
        ops: Vec::new_in(&arena),
    };
    result.ops.extend(ops);
    result
}

fn span(source: &str, needle: &str) -> Span {
    let start = source.find(needle).unwrap();
    Span::new(start as u32, (start + needle.len()) as u32)
}

fn expression<'a>(arena: &'a Allocator, source: &'a str, authored: &'a str) -> ExprRef<'a> {
    let at = span(source, authored);
    ExprRef::Js(JsExpr::parse_in(arena, authored, at).unwrap())
}

fn text<'a>(arena: &'a Allocator, source: &str, content: &'a str) -> Op<'a> {
    Op::Text(Box::new_in(
        TextOp {
            content,
            span: span(source, content),
        },
        &arena,
    ))
}

fn interpolation<'a>(arena: &'a Allocator, source: &'a str, authored: &'a str) -> Op<'a> {
    let expression = expression(arena, source, authored);
    let expression_span = expression.span();
    Op::Interpolation(Box::new_in(
        InterpolationOp {
            expression,
            span: Span::new(expression_span.start - 2, expression_span.end + 2),
        },
        &arena,
    ))
}

fn attribute<'a>(source: &str, spelling: &str, name: &'a str, value: &'a str) -> Attribute<'a> {
    Attribute {
        name,
        value: Some(value),
        span: span(source, spelling),
    }
}

fn bind<'a>(
    arena: &'a Allocator,
    source: &'a str,
    spelling: &str,
    name: &'a str,
    value: &'a str,
) -> BindingOp<'a> {
    BindingOp::Bind(Box::new_in(
        BindOp {
            name: Some(DynamicName::Static(name)),
            modifiers: Vec::new_in(&arena),
            value: Some(expression(arena, source, value)),
            span: span(source, spelling),
        },
        &arena,
    ))
}

fn element<'a>(
    arena: &'a Allocator,
    source: &str,
    spelling: &str,
    tag: &'a str,
    attributes: impl IntoIterator<Item = Attribute<'a>>,
    bindings: impl IntoIterator<Item = BindingOp<'a>>,
    children: impl IntoIterator<Item = Op<'a>>,
) -> Op<'a> {
    let mut attrs = Vec::new_in(&arena);
    attrs.extend(attributes);
    let mut attached = Vec::new_in(&arena);
    attached.extend(bindings);
    Op::Element(Box::new_in(
        ElementOp {
            tag,
            namespace: Namespace::Html,
            attributes: attrs,
            bindings: attached,
            children: region(arena, children),
            span: span(source, spelling),
        },
        &arena,
    ))
}

/// Each node is constructed with its real authored span. This target test
/// receives the sealed native owner directly, not a legacy parser artifact.
/// Expressions parse once into retained ASTs; emission never calls a parser.
pub(super) fn build<'a>(arena: &'a Allocator, id: &str, source: &'a str) -> Artifact<'a> {
    let root = match id {
        "empty" => region(arena, []),
        "empty-element" => region(arena, [element(arena, source, source, "div", [], [], [])]),
        "static-prop-text" => region(
            arena,
            [element(
                arena,
                source,
                source,
                "p",
                [attribute(source, "title=\"hi\"", "title", "hi")],
                [],
                [text(arena, source, "Hello")],
            )],
        ),
        "nested-comment" => {
            let child = element(
                arena,
                source,
                "<span>x</span>",
                "span",
                [],
                [],
                [text(arena, source, "x")],
            );
            let comment = Op::Comment(Box::new_in(
                CommentOp {
                    content: "ok",
                    span: span(source, "<!--ok-->"),
                },
                &arena,
            ));
            region(
                arena,
                [element(
                    arena,
                    source,
                    source,
                    "div",
                    [attribute(source, "id=\"a\"", "id", "a")],
                    [],
                    [child, comment],
                )],
            )
        }
        "mixed-text-runs" => {
            let child = element(arena, source, "<span/>", "span", [], [], []);
            region(
                arena,
                [element(
                    arena,
                    source,
                    source,
                    "div",
                    [],
                    [],
                    [
                        text(arena, source, "a"),
                        interpolation(arena, source, "x"),
                        child,
                        text(arena, source, "b"),
                        interpolation(arena, source, "y"),
                    ],
                )],
            )
        }
        "dynamic-prop-text" => region(
            arena,
            [element(
                arena,
                source,
                source,
                "p",
                [],
                [bind(arena, source, ":title=\"tip\"", "title", "tip")],
                [interpolation(arena, source, "count + offset")],
            )],
        ),
        "class-style" => region(
            arena,
            [element(
                arena,
                source,
                source,
                "div",
                [],
                [
                    bind(arena, source, ":class=\"classes\"", "class", "classes"),
                    bind(arena, source, ":style=\"styles\"", "style", "styles"),
                ],
                [],
            )],
        ),
        "root-fragment" => region(
            arena,
            [
                element(
                    arena,
                    source,
                    "<p>A</p>",
                    "p",
                    [],
                    [],
                    [text(arena, source, "A")],
                ),
                element(
                    arena,
                    source,
                    "<p>B</p>",
                    "p",
                    [],
                    [],
                    [text(arena, source, "B")],
                ),
            ],
        ),
        "root-text-run" => region(
            arena,
            [
                text(arena, source, "hello "),
                interpolation(arena, source, "msg"),
            ],
        ),
        _ => extra::build(arena, id, source)
            .unwrap_or_else(|| panic!("unknown native DOM reference {id}")),
    };
    Artifact::try_new(ArtifactParts {
        source,
        root,
        provenance: OwnedVec::new(),
        scopes: SideTable::new(),
    })
    .unwrap()
}

pub(super) fn property_value<'a>(
    arena: &'a Allocator,
    source: &'a str,
    value: &'a str,
) -> Artifact<'a> {
    Artifact::try_new(ArtifactParts {
        source,
        root: region(
            arena,
            [element(
                arena,
                source,
                source,
                "p",
                [],
                [bind(arena, source, source, "title", value)],
                [],
            )],
        ),
        provenance: OwnedVec::new(),
        scopes: SideTable::new(),
    })
    .unwrap()
}

pub(super) fn rejected_event(arena: &Allocator) -> Artifact<'_> {
    let source = "<button @click=\"handler\"/>";
    let at = span(source, "@click=\"handler\"");
    let binding = BindingOp::On(Box::new_in(
        OnOp {
            name: Some(DynamicName::Static("click")),
            modifiers: Vec::new_in(&arena),
            handler: Some(expression(arena, source, "handler").into()),
            span: at,
        },
        &arena,
    ));
    let provenance = OwnedVec::from([ProvenanceRecord {
        rule: String::from("native.fixture"),
        node: Some(NodeId::from_index(1).unwrap()),
        before: String::from("@click=\"handler\""),
        after: String::from("unsupported-event"),
        span: at,
    }]);
    Artifact::try_new(ArtifactParts {
        source,
        root: region(
            arena,
            [element(arena, source, source, "button", [], [binding], [])],
        ),
        provenance,
        scopes: SideTable::new(),
    })
    .unwrap()
}
