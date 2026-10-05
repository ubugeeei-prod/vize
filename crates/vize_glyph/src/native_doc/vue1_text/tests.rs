//! Independent whole Doc trees and original authored framing.

use super::vue1_text_document;
use crate::native_doc::{Doc, Line, PrintOptions, document::Kind};
use vize_l0::Allocator;
use vize_l1::dialect::vue1::surface;

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

#[test]
fn whole_doc_tree_retains_original_delimiters_trim_and_entity_operator() {
    for (source, operator) in [("{{ a+b }}", "+"), ("{{ a&#43;b }}", "&#43;")] {
        let arena = Allocator::default();
        let owner = surface::parse_component(&arena, source).unwrap();
        let view = owner.text_for(owner.children().next().unwrap()).unwrap();
        let document = vue1_text_document(view, &arena).unwrap();
        assert_eq!(
            tree(&document.document),
            concat(vec![
                Tree::Text("{{ "),
                concat(vec![
                    Tree::Text(""),
                    Tree::Group(std::boxed::Box::new(concat(vec![
                        Tree::Text("a"),
                        Tree::Text(" "),
                        Tree::Text(operator),
                        Tree::Indent(
                            1,
                            std::boxed::Box::new(concat(vec![
                                Tree::Line(Line::Space),
                                Tree::Text("b"),
                            ])),
                        ),
                    ]))),
                    Tree::Text(""),
                ]),
                Tree::Text(" }}"),
            ])
        );
        let Kind::Concat(parts) = &document.document.kind else {
            panic!("whole original callback")
        };
        let Kind::Text(prefix) = parts[0].kind else {
            panic!("authored opening delimiter")
        };
        let Kind::Text(suffix) = parts[2].kind else {
            panic!("authored closing delimiter")
        };
        assert_eq!(prefix.as_ptr(), source.as_ptr());
        assert_eq!(suffix.as_ptr(), source[source.len() - 3..].as_ptr());
    }
}

#[test]
fn complete_comment_and_trim_bytes_survive_without_vue2_comment_policy() {
    for (source, expected) in [
        ("{{ a/*keep*/+b }}", "{{ a/*keep*/+ b }}"),
        ("{{ /*keep*/ a + b }}", "{{ /*keep*/ a + b }}"),
        ("{{\n a + b\n}}", "{{\n a + b\n}}"),
        (
            "{{\u{00a0}\u{feff}a\u{3000}}}",
            "{{\u{00a0}\u{feff}a\u{3000}}}",
        ),
    ] {
        let arena = Allocator::default();
        let owner = surface::parse_component(&arena, source).unwrap();
        let syntax = owner.bindings()[0].syntax().unwrap();
        let comments = syntax.comments().count();
        let root = core::ptr::from_ref(syntax.expression().unwrap());
        let document = vue1_text_document(
            owner.text_for(owner.children().next().unwrap()).unwrap(),
            &arena,
        )
        .unwrap();
        assert_eq!(document.print(&PrintOptions::default()).unwrap(), expected);
        assert_eq!(syntax.comments().count(), comments);
        assert_eq!(core::ptr::from_ref(syntax.expression().unwrap()), root);
        assert_eq!(syntax.hole(), None);
    }
}
