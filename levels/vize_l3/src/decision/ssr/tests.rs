use super::{SsrPart, SsrUnsupported, build_ssr_file_decisions};
use crate::decision::{DecisionBuildError, build_decisions, policy::TargetPolicy};
use vize_l0::{Allocator, Span, Vec, id::NodeId};
use vize_l2::{
    artifact::{Builder, ComponentFactory},
    file::{TemplatePolicy, TemplateScope},
    lang::js::FileProducer,
    op::Namespace,
};

#[test]
fn sole_walk_retains_original_nodes_root_inheritance_and_void_content() {
    let arena = Allocator::default();
    let source = "<!--x--><div><br>ok</div>";
    let mut builder = Builder::new(&arena, source).unwrap();
    builder.comment("x", Span::new(0, 8)).unwrap();
    let element = builder
        .element(
            "div",
            Namespace::Html,
            Vec::new_in(&&arena),
            Span::new(8, 25),
            |body, _| {
                body.element(
                    "br",
                    Namespace::Html,
                    Vec::new_in(&&arena),
                    Span::new(13, 17),
                    |_, _| {},
                )
                .unwrap();
                body.text("ok", Span::new(17, 19)).unwrap();
            },
        )
        .unwrap();
    let artifact = builder.finish().unwrap();
    let result = build_decisions(&artifact, TargetPolicy::Ssr).unwrap();
    let facts = result.ssr().unwrap();
    assert!(facts.unsupported().is_empty());
    assert!(facts.fragment());
    assert_eq!(facts.inherit_attrs(), Some(element));
    assert_eq!(result.tables().nodes.len(), artifact.node_count() as usize);
    assert_eq!(facts.parts().len(), 5);
    let SsrPart::Open {
        node,
        element: admitted,
        void,
    } = facts.parts()[1]
    else {
        panic!("original element")
    };
    assert_eq!(node, element);
    assert!(!void);
    let vize_l2::op::Op::Element(original) = &artifact.root().ops[1] else {
        panic!("original element")
    };
    assert!(core::ptr::eq(admitted, original.as_ref()));
    assert!(matches!(facts.parts()[2], SsrPart::Open { void: true, .. }));
    assert!(
        build_decisions(&artifact, TargetPolicy::Dom)
            .unwrap()
            .ssr()
            .is_none()
    );
    assert!(
        build_decisions(&artifact, TargetPolicy::Vapor)
            .unwrap()
            .ssr()
            .is_none()
    );
}

#[test]
fn every_authored_root_controls_fragment_boundaries_and_text_has_no_fallthrough() {
    let arena = Allocator::default();
    let mut builder = Builder::new(&arena, "a<!--b-->c").unwrap();
    builder.text("a", Span::new(0, 1)).unwrap();
    builder.comment("b", Span::new(1, 9)).unwrap();
    builder.text("c", Span::new(9, 10)).unwrap();
    let artifact = builder.finish().unwrap();
    let analysis = build_decisions(&artifact, TargetPolicy::Ssr).unwrap();
    let facts = analysis.ssr().unwrap();
    assert!(facts.fragment());
    assert_eq!(facts.inherit_attrs(), None);
    assert_eq!(facts.parts().len(), 3);
    let empty = Builder::new(&arena, "").unwrap().finish().unwrap();
    let analysis = build_decisions(&empty, TargetPolicy::Ssr).unwrap();
    assert!(analysis.ssr().unwrap().parts().is_empty());
    assert!(!analysis.ssr().unwrap().fragment());
}

#[test]
fn unsafe_comments_raw_text_namespaces_and_void_children_are_typed_refusals() {
    let arena = Allocator::default();
    let mut builder = Builder::new(&arena, "<!--->-->").unwrap();
    builder.comment("->", Span::new(0, 9)).unwrap();
    let artifact = builder.finish().unwrap();
    let analysis = build_decisions(&artifact, TargetPolicy::Ssr).unwrap();
    assert_eq!(
        analysis.ssr().unwrap().unsupported()[0].reason,
        SsrUnsupported::UnsafeComment
    );
    for (tag, namespace, expected) in [
        ("script", Namespace::Html, SsrUnsupported::ElementSemantics),
        ("search", Namespace::Html, SsrUnsupported::ElementSemantics),
        ("svg", Namespace::Svg, SsrUnsupported::Namespace),
        ("IMG", Namespace::Html, SsrUnsupported::ElementName),
    ] {
        let mut builder = Builder::new(&arena, "<script></script>").unwrap();
        builder
            .element(
                tag,
                namespace,
                Vec::new_in(&&arena),
                Span::new(0, 17),
                |_, _| {},
            )
            .unwrap();
        let artifact = builder.finish().unwrap();
        let analysis = build_decisions(&artifact, TargetPolicy::Ssr).unwrap();
        assert_eq!(analysis.ssr().unwrap().unsupported()[0].reason, expected);
    }
    let mut builder = Builder::new(&arena, "<br>x").unwrap();
    builder
        .element(
            "br",
            Namespace::Html,
            Vec::new_in(&&arena),
            Span::new(0, 5),
            |body, _| {
                body.text("x", Span::new(4, 5)).unwrap();
            },
        )
        .unwrap();
    let artifact = builder.finish().unwrap();
    let analysis = build_decisions(&artifact, TargetPolicy::Ssr).unwrap();
    assert_eq!(analysis.ssr().unwrap().unsupported()[0].node, NodeId::FIRST);
    assert_eq!(
        analysis.ssr().unwrap().unsupported()[0].reason,
        SsrUnsupported::VoidChildren
    );
}

#[test]
fn file_entry_refuses_interrupted_template_and_keeps_sole_owner() {
    #[derive(Clone, Copy)]
    struct Visible;
    impl TemplatePolicy for Visible {
        fn visible(self, _: &vize_l2::file::Declaration) -> bool {
            false
        }
    }
    let arena = Allocator::default();
    let mut producer = FileProducer::new(&arena, "hello").unwrap();
    {
        let mut template = producer
            .template_region(TemplateScope::Root, Visible)
            .unwrap();
        let mut walk = template.walk(Span::new(0, 5)).unwrap();
        walk.text("hello", Span::new(0, 5)).unwrap();
    }
    let file = producer.finish().unwrap();
    assert!(matches!(
        build_ssr_file_decisions(&file),
        Err(DecisionBuildError::IncompleteFile)
    ));
    assert_eq!(file.artifact().source(), "hello");
    assert!(file.template_interruption().is_some());
    assert!(!file.is_complete());
}
