#![cfg(test)]

use serde_json::{Value, json};
use vize_armature::{
    ElementNode, ParserOptions, TemplateChildNode, WhitespaceStrategy, parse_document_with_options,
};
use vize_l0::{Allocator, is_native_tag};

const CORPUS: &str = include_str!("fixtures/document/html-select.json");

fn packet(children: &[TemplateChildNode<'_>]) -> (Vec<Value>, Vec<Value>) {
    fn walk(
        children: &[TemplateChildNode<'_>],
        parent: Option<usize>,
        elements: &mut Vec<Value>,
        nodes: &mut Vec<Value>,
    ) {
        for child in children {
            let TemplateChildNode::Element(element) = child else {
                continue;
            };
            let index = elements.len();
            elements.push(json!([element.tag.to_ascii_lowercase(), parent]));
            if !element.tag.eq_ignore_ascii_case("html")
                && !element.tag.eq_ignore_ascii_case("head")
            {
                nodes.push(json!([
                    element.tag.to_ascii_lowercase(),
                    direct_nodes(element)
                ]));
            }
            walk(&element.children, Some(index), elements, nodes);
        }
    }

    fn direct_nodes(element: &ElementNode<'_>) -> Vec<Value> {
        element
            .children
            .iter()
            .map(|child| match child {
                TemplateChildNode::Element(child) => {
                    json!([1, child.tag.to_ascii_lowercase()])
                }
                TemplateChildNode::Text(child) => json!([3, child.content]),
                TemplateChildNode::Comment(child) => json!([8, child.content]),
                unexpected => panic!("unexpected select child: {unexpected:?}"),
            })
            .collect()
    }

    let mut elements = Vec::new();
    let mut nodes = Vec::new();
    walk(children, None, &mut elements, &mut nodes);
    (elements, nodes)
}

#[test]
fn admitted_select_packets_match_the_shipped_document_parser() {
    let cases: Vec<Value> = serde_json::from_str(CORPUS).unwrap();
    let admitted: Vec<_> = cases.iter().filter(|case| case["legacy"] == true).collect();
    assert_eq!(admitted.len(), 4);
    for case in admitted {
        let label = case["name"].as_str().unwrap();
        let source = case["source"].as_str().unwrap();
        let allocator = Allocator::default();
        let options = ParserOptions {
            is_native_tag: Some(is_native_tag),
            whitespace: WhitespaceStrategy::Preserve,
            ..ParserOptions::default()
        };
        let (root, errors) = parse_document_with_options(&allocator, source, options);
        assert!(errors.is_empty(), "{label}: {errors:?}");
        let (elements, nodes) = packet(&root.children);
        let expected_elements: Vec<_> = case["elements"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| json!([row[0], row[1]]))
            .collect();
        assert_eq!(elements, expected_elements, "elements: {label}");
        assert_eq!(json!(nodes), case["nodes"], "nodes: {label}");
    }
}
