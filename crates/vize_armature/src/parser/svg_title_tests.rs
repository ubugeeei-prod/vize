//! The production compatibility parser consumes the same independent corpus.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "independent complete authored parser corpus"
)]

use super::parse;
use vize_l0::Allocator;
use vize_relief::TemplateChildNode;

fn observe(children: &[TemplateChildNode<'_>], values: &mut Vec<String>) {
    for child in children {
        match child {
            TemplateChildNode::Element(node) => {
                values.push(format!("+{}", node.tag));
                observe(&node.children, values);
                values.push(format!("-{}", node.tag));
            }
            TemplateChildNode::Text(node) => values.push(format!("={}", node.content)),
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
    assert_eq!(count, 11, "all authored controls execute");
}
