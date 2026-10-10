use super::Parser;
use vize_l0::Allocator;

#[test]
fn opted_in_spans_bind_source_and_literal_classification_is_compiler_only() {
    let source = "<div v-pre><span><Child is=\"vue:Child\"></Child></span></div><Child></Child>";
    let arena = Allocator::new();
    let (root, errors, frozen) = Parser::new(&arena, source).parse_with_frozen_elements();
    let (control, control_errors) = Parser::new(&arena, source).parse();
    assert_ne!(vize_l0::cstr!("{root:?}"), vize_l0::cstr!("{control:?}"));
    let vize_relief::TemplateChildNode::Element(owner) = &root.children[0] else {
        panic!("owner")
    };
    let vize_relief::TemplateChildNode::Element(span) = &owner.children[0] else {
        panic!("span")
    };
    let vize_relief::TemplateChildNode::Element(child) = &span.children[0] else {
        panic!("Child")
    };
    assert_eq!(child.tag_type, vize_relief::ElementType::Element);
    let vize_relief::TemplateChildNode::Element(owner) = &control.children[0] else {
        panic!("owner")
    };
    let vize_relief::TemplateChildNode::Element(span) = &owner.children[0] else {
        panic!("span")
    };
    let vize_relief::TemplateChildNode::Element(child) = &span.children[0] else {
        panic!("Child")
    };
    assert_eq!(child.tag_type, vize_relief::ElementType::Component);
    assert_eq!(
        vize_l0::cstr!("{errors:?}"),
        vize_l0::cstr!("{control_errors:?}")
    );
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(
        frozen
            .spans
            .iter()
            .map(|span| span.slice(source))
            .collect::<std::vec::Vec<_>>(),
        ["<div v-pre>", "<span>", "<Child is=\"vue:Child\">"],
    );
    assert_eq!(frozen.spans_for(root.source, source), Some(frozen.spans));
    let copy = source.to_owned();
    assert_eq!(frozen.spans_for(&copy, source), None);
    assert_eq!(frozen.spans_for(root.source, &copy), None);
}

#[test]
fn opening_ownership_survives_html_scope_recovery_without_changing_recovery() {
    let source = "<p v-pre><div><Child></Child></div><p></p>";
    let arena = Allocator::new();
    let (root, errors, frozen) = Parser::new(&arena, source).parse_with_frozen_elements();
    let (control, control_errors) = Parser::new(&arena, source).parse();
    assert_eq!(vize_l0::cstr!("{root:?}"), vize_l0::cstr!("{control:?}"));
    assert_eq!(
        vize_l0::cstr!("{errors:?}"),
        vize_l0::cstr!("{control_errors:?}")
    );
    assert_eq!(
        frozen
            .spans
            .iter()
            .map(|span| span.slice(source))
            .collect::<std::vec::Vec<_>>(),
        ["<p v-pre>", "<div>"],
    );
    assert!(
        frozen
            .spans
            .windows(2)
            .all(|pair| pair[0].start < pair[1].start)
    );
}

#[test]
fn normal_opt_in_is_empty_and_default_parser_disables_collection() {
    let source = "<Child :id=\"active\"></Child>";
    let arena = Allocator::new();
    assert!(Parser::new(&arena, source).frozen_elements.is_none());
    let (root, errors, frozen) = Parser::new(&arena, source).parse_with_frozen_elements();
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(frozen.spans_for(root.source, source), Some(&[][..]));
}

#[test]
#[cfg(feature = "legacy")]
fn compiler_head_locations_do_not_change_the_original_default_v_pre_results() {
    use vize_relief::{PropNode, TemplateChildNode};

    macro_rules! before {
        ($name:literal) => {
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/_fixtures/v_pre_name_locations/",
                $name
            ))
        };
    }

    for (source, historical_ast, historical_errors, original_end, head_end) in [
        (
            "<p v-pre :title='value'>{{ literal }}<i @click='f'>{{nested}}</i></p>{{normal}}",
            before!("case-0.before.ast.txt"),
            before!("case-0.before.errors.txt"),
            23,
            15,
        ),
        (
            "<p v-pre :broken='unfinished",
            before!("case-7.before.ast.txt"),
            before!("case-7.before.errors.txt"),
            28,
            16,
        ),
    ] {
        let arena = Allocator::new();
        let (root, errors) = Parser::new(&arena, source).parse();
        assert_eq!(vize_l0::cstr!("{root:#?}"), historical_ast);
        assert_eq!(vize_l0::cstr!("{errors:#?}"), historical_errors);
        let (compiler_root, compiler_errors, frozen) =
            Parser::new(&arena, source).parse_with_frozen_elements();
        assert_eq!(vize_l0::cstr!("{compiler_errors:#?}"), historical_errors);
        assert_eq!(
            frozen.spans_for(compiler_root.source, source),
            Some(frozen.spans)
        );
        for (tree, end) in [(&root, original_end), (&compiler_root, head_end)] {
            let Some(TemplateChildNode::Element(element)) = tree.children.first() else {
                panic!("original literal p");
            };
            let Some(PropNode::Attribute(attribute)) = element.props.first() else {
                panic!("original frozen attribute");
            };
            assert_eq!(attribute.name_loc.span.start, 9);
            assert_eq!(attribute.name_loc.span.end, end);
            assert_eq!(attribute.loc.span.end, original_end);
        }
    }
}
