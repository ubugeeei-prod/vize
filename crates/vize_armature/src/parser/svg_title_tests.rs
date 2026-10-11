//! The production compatibility parser consumes the same independent corpus.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "independent complete authored parser corpus"
)]

use super::parse;
use vize_l0::Allocator;
use vize_relief::{ExpressionNode, TemplateChildNode};

fn observe(children: &[TemplateChildNode<'_>], values: &mut Vec<String>) {
    for child in children {
        match child {
            TemplateChildNode::Element(node) => {
                values.push(format!("+{}", node.tag));
                observe(&node.children, values);
                values.push(format!("-{}", node.tag));
            }
            TemplateChildNode::Text(node) => values.push(format!("={}", node.content)),
            TemplateChildNode::Interpolation(node) => match &node.content {
                ExpressionNode::Simple(expression) => {
                    values.push(format!("?{}", expression.content))
                }
                _ => panic!("unexpected compound authored interpolation"),
            },
            _ => panic!("unexpected authored parser structure"),
        }
    }
}

#[test]
fn svg_title_children_preserve_complete_html_rcdata_controls() {
    let corpus =
        include_str!("../../../../tests/_fixtures/differential/compiler/vapor-svg-title-8624.tsv");
    let mut count = 0;
    for row in corpus.lines() {
        let mut fields = row.split('\t');
        let name = fields.next().unwrap();
        let source = fields.next().unwrap();
        let expected: Vec<_> = fields.next().unwrap().split('|').collect();
        assert!(fields.next().is_none(), "{name}: exact corpus row");
        let allocator = Allocator::new();
        let (root, errors) = parse(&allocator, source);
        assert!(errors.is_empty(), "{name}: {errors:?}");
        let mut actual = Vec::new();
        observe(&root.children, &mut actual);
        assert_eq!(actual, expected, "{name}: entire ordered structure");
        count += 1;
    }
    assert_eq!(count, 12, "all authored controls execute");
}

#[test]
fn svg_title_matches_pinned_vue36_scope() {
    let corpus = include_str!(
        "../../../../tests/_fixtures/differential/compiler/vapor-svg-title-scope-8624/controls.tsv"
    );
    let mut snapshots = Vec::new();
    let mut expected = Vec::new();
    for row in corpus.lines() {
        let fields: Vec<_> = row.split('\t').collect();
        assert_eq!(fields.len(), 3);
        let allocator = Allocator::new();
        let (root, errors) = parse(&allocator, fields[1]);
        assert!(errors.is_empty(), "{}: {errors:?}", fields[0]);
        let mut actual = Vec::new();
        observe(&root.children, &mut actual);
        snapshots.push((fields[0], actual));
        expected.push((fields[0], fields[2].split('|').map(str::to_owned).collect()));
    }
    assert_eq!(snapshots.len(), 11, "all additional whole controls execute");
    assert_eq!(snapshots, expected, "complete pinned Vue 3.6 scope corpus");
}
