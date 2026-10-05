//! Complete independent Doc topology and actual crossed-envelope controls.

use super::*;
use crate::native_doc::{Doc, Line, document::Kind};
use vize_l0::{Allocator, SourceFrameError, Span};
use vize_l1::container::Vue;

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
fn whole_doc_topology_retains_full_original_frame_and_borrowed_binary() {
    let arena = Allocator::default();
    let source = "<!--前--><template>{{a+b}}</template><!--尾-->";
    let owner = observe_native_vue2_sfc_in(&arena, source, NativeVue2SfcOptions::default());
    let binary = concat(vec![
        Tree::Text(""),
        Tree::Group(std::boxed::Box::new(concat(vec![
            Tree::Text("a"),
            Tree::Text(" "),
            Tree::Text("+"),
            Tree::Indent(
                1,
                std::boxed::Box::new(concat(vec![Tree::Line(Line::Space), Tree::Text("b")])),
            ),
        ]))),
        Tree::Text(""),
    ]);
    assert_eq!(
        tree(owner.document().unwrap()),
        concat(vec![
            Tree::Text("<!--前--><template>"),
            concat(vec![
                Tree::Text(""),
                Tree::Indent(
                    0,
                    std::boxed::Box::new(concat(vec![Tree::Text("{{"), binary, Tree::Text("}}"),]))
                )
            ]),
            Tree::Text("</template><!--尾-->"),
        ])
    );
    let Kind::Concat(parts) = &owner.document().unwrap().kind else {
        panic!("complete root")
    };
    let Kind::Text(prefix) = parts[0].kind else {
        panic!("physical prefix")
    };
    let Kind::Text(suffix) = parts[2].kind else {
        panic!("physical suffix")
    };
    assert!(core::ptr::eq(prefix.as_ptr(), source.as_ptr()));
    assert!(core::ptr::eq(
        suffix.as_ptr(),
        source[source.find("</template>").unwrap()..].as_ptr()
    ));
    assert_eq!(owner.document().unwrap().flat_width, Some(46));
}

#[test]
fn original_envelope_join_refuses_real_same_buffer_and_equal_byte_descriptors() {
    let arena = Allocator::default();
    let source = std::string::String::from("<!--前--><template>{{a}}</template>");
    let copied = source.clone();
    assert_ne!(source.as_ptr(), copied.as_ptr());
    let options = NativeVue2SfcOptions::default();
    let descriptor = Vue.observe_vue2_descriptor(&arena, &source, options.descriptor);
    let selected = descriptor.selected().unwrap();
    let content = selected.block().span();
    for foreign in [
        Vue.observe_vue2_descriptor(&arena, &source, options.descriptor),
        Vue.observe_vue2_descriptor(&arena, &copied, options.descriptor),
    ] {
        let original_blocks = foreign.container().blocks.as_ptr();
        let view = foreign.selected().unwrap();
        assert_eq!(
            build::frame(&descriptor, &view, Doc::text("{{a}}"), &arena).unwrap_err(),
            NativeVue2SfcRefusal::Frame {
                span: content,
                error: SourceFrameError::BlockNotRootSlice
            }
        );
        assert_eq!(foreign.container().blocks.as_ptr(), original_blocks);
        assert!(core::ptr::eq(view.observation(), &foreign));
    }
    assert_eq!(content, Span::new(20, 25));
    assert!(build::frame(&descriptor, &selected, Doc::text("{{a}}"), &arena).is_ok());
}
