use super::*;
use crate::{
    AllocBox, AllocVec, Allocator, ElementNode, ExpressionNode, ForNode, ForParseResult, Position,
    SimpleExpressionNode, SourceLocation, TextNode,
};

fn loc(source: &str, start: u32, end: u32) -> SourceLocation {
    SourceLocation::new(
        Position::new(start, 1, start + 1),
        Position::new(end, 1, end + 1),
        source,
    )
}

fn simple<'a>(bump: &'a vize_carton::Bump, text: &'a str) -> ExpressionNode<'a> {
    ExpressionNode::Simple(AllocBox::new_in(
        SimpleExpressionNode::new(text, false, loc(text, 0, text.len() as u32)),
        bump,
    ))
}

#[test]
fn classifies_relief_template_children_as_rendu_ops() {
    let allocator = Allocator::default();
    let bump = allocator.as_bump();

    let element = TemplateChildNode::Element(AllocBox::new_in(
        ElementNode::new(bump, "MyButton", loc("<MyButton />", 0, 12)),
        bump,
    ));
    let text = TemplateChildNode::Text(AllocBox::new_in(
        TextNode::new("Save", loc("Save", 13, 17)),
        bump,
    ));

    assert_eq!(
        RenduOp::from_template_child(&element),
        RenduOp::Element {
            tag: "MyButton",
            kind: RenduElementKind::Element,
            span: RenduSpan::new(Position::new(0, 1, 1), Position::new(12, 1, 13)),
        }
    );
    assert_eq!(
        RenduOp::from_template_child(&text),
        RenduOp::Text {
            content: "Save",
            span: RenduSpan::new(Position::new(13, 1, 14), Position::new(17, 1, 18)),
        }
    );
}

#[test]
fn roots_carry_target_and_dialect_capability_facts() {
    let op = RenduOp::HoistRef { index: 1 };
    let ops = [op];
    let root = RenduRoot::new(
        RenduSource::named("App.vue", "<div />"),
        RenduBlock::new(&ops),
        RenduCapabilities::empty()
            .with_target(SourceAtlasTarget::Vdom)
            .with_target(SourceAtlasTarget::Ssr)
            .with_coordinate(SourceAtlasCoordinate::Vapor),
    );

    assert_eq!(root.source.filename, Some("App.vue"));
    assert!(!root.entry.is_empty());
    assert!(root.capabilities.targets.contains(SourceAtlasTarget::Vdom));
    assert!(root.capabilities.targets.contains(SourceAtlasTarget::Ssr));
    assert_eq!(
        root.capabilities.coordinate,
        Some(SourceAtlasCoordinate::Vapor)
    );
}

#[test]
fn expression_refs_borrow_relief_expression_text() {
    let allocator = Allocator::default();
    let expression = simple(allocator.as_bump(), "count + 1");

    let expr_ref = RenduExprRef::from_expression(&expression);
    // Equality is by source text, so the borrowed node and a string ref naming
    // the same expression compare equal.
    assert_eq!(expr_ref, RenduExprRef::Relief("count + 1"));
    // The text accessor returns the borrowed source regardless of origin.
    assert_eq!(expr_ref.text(), "count + 1");
    assert_eq!(RenduExprRef::Oxc("a.b").text(), "a.b");
    // `from_expression` carries the real Relief node, reachable via `node()`.
    assert!(expr_ref.node().is_some());
    assert!(RenduExprRef::Relief("count + 1").node().is_none());
}

#[test]
fn for_ops_carry_value_key_and_index_aliases_losslessly() {
    let allocator = Allocator::default();
    let bump = allocator.as_bump();

    // v-for="(item, key, index) in items"
    let node = ForNode {
        source: simple(bump, "items"),
        value_alias: Some(simple(bump, "item")),
        key_alias: Some(simple(bump, "key")),
        object_index_alias: Some(simple(bump, "index")),
        parse_result: ForParseResult {
            source: simple(bump, "items"),
            value: None,
            key: None,
            index: None,
            finalized: true,
        },
        children: AllocVec::new_in(bump),
        loc: loc("v-for", 0, 5),
    };
    let child = TemplateChildNode::For(AllocBox::new_in(node, bump));

    match RenduOp::from_template_child(&child) {
        RenduOp::For {
            source,
            value,
            key,
            index,
            ..
        } => {
            assert_eq!(source.text(), "items");
            assert_eq!(value.map(RenduExprRef::text), Some("item"));
            assert_eq!(key.map(RenduExprRef::text), Some("key"));
            assert_eq!(index.map(RenduExprRef::text), Some("index"));
        }
        other => panic!("expected RenduOp::For, got {other:?}"),
    }
}
