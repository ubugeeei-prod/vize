//! Independent complete Doc trees and genuine private borrowed-origin controls.

use super::*;
use crate::native_doc::{Line, PrintOptions, document::Kind, print};
use vize_l1::dialect::vue2::surface;

#[derive(Debug, PartialEq, Eq)]
enum Tree<'a> {
    Text(&'a str),
    Line(Line),
    HardLine,
    Concat(std::vec::Vec<Self>),
    Group(std::boxed::Box<Self>),
    Indent(usize, std::boxed::Box<Self>),
}
fn tree<'a>(doc: &Doc<'a>) -> Tree<'a> {
    match &doc.kind {
        Kind::Text(text) => Tree::Text(text),
        Kind::Line(line) => Tree::Line(*line),
        Kind::HardLine => Tree::HardLine,
        Kind::Concat(parts) => Tree::Concat(parts.iter().map(tree).collect()),
        Kind::Group(inner) => Tree::Group(std::boxed::Box::new(tree(inner))),
        Kind::Indent(levels, inner) => Tree::Indent(*levels, std::boxed::Box::new(tree(inner))),
    }
}
fn concat(parts: std::vec::Vec<Tree<'_>>) -> Tree<'_> {
    Tree::Concat(parts)
}
fn atom(text: &str) -> Tree<'_> {
    concat(vec![Tree::Text(""), Tree::Text(text), Tree::Text("")])
}
fn infix<'a>(left: &'a str, operator: &'a str, right: &'a str) -> Tree<'a> {
    concat(vec![
        Tree::Text(""),
        Tree::Group(std::boxed::Box::new(concat(vec![
            Tree::Text(left),
            Tree::Text(" "),
            Tree::Text(operator),
            Tree::Indent(
                1,
                std::boxed::Box::new(concat(vec![Tree::Line(Line::Space), Tree::Text(right)])),
            ),
        ]))),
        Tree::Text(""),
    ])
}

#[test]
fn complete_doc_trees_keep_every_authored_gap_and_original_operand_structure() {
    for (source, expected) in [
        (
            "{{ a | f( b, ) }}",
            concat(vec![
                Tree::Text("{{ "),
                atom("a"),
                Tree::Text(" | f( "),
                atom("b"),
                Tree::Text(", ) }}"),
            ]),
        ),
        (
            "{{ a+b | f( c*d, ) }}",
            concat(vec![
                Tree::Text("{{ "),
                infix("a", "+", "b"),
                Tree::Text(" | f( "),
                infix("c", "*", "d"),
                Tree::Text(", ) }}"),
            ]),
        ),
        (
            "{{ a&#43;b &#124; f() }}",
            concat(vec![
                Tree::Text("{{ "),
                infix("a", "&#43;", "b"),
                Tree::Text(" &#124; f() }}"),
            ]),
        ),
        (
            "{{ 日本 | upper () }}",
            concat(vec![
                Tree::Text("{{ "),
                atom("日本"),
                Tree::Text(" | upper () }}"),
            ]),
        ),
    ] {
        let arena = Allocator::default();
        let owner = surface::parse_component(&arena, source).unwrap();
        let view = owner.text_for(owner.children().next().unwrap()).unwrap();
        let document = vue2_text_document(view, &arena).unwrap();
        assert_eq!(tree(document.document()), expected, "{source}");
        let Kind::Concat(parts) = &document.document().kind else {
            panic!("complete root")
        };
        let Kind::Text(prefix) = parts[0].kind else {
            panic!("complete prefix")
        };
        assert_eq!(prefix.as_ptr(), source.as_ptr());
        let Kind::Text(tail) = parts.last().unwrap().kind else {
            panic!("complete tail")
        };
        assert_eq!(
            tail.as_ptr(),
            source.get(source.len() - tail.len()..).unwrap().as_ptr()
        );
    }
}
