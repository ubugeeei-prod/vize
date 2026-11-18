//! Document mode: the HTML tree construction closes these elements
//! implicitly. A component template keeps them as authored
//! (`tests::authored_structure`).
#![expect(clippy::disallowed_macros, reason = "fixtures are built with format!")]

use super::parse_document;
use vize_l0::Allocator;
use vize_relief::TemplateChildNode;

#[test]
fn test_parse_in_body_omits_p_and_li_end_tags() {
    let allocator = Allocator::new();
    let (root, errors) = parse_document(&allocator, "<p>a<p>b<ul><li>c<li>d</ul>");

    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(root.children.len(), 3);
    assert!(matches!(&root.children[0], TemplateChildNode::Element(p) if p.tag == "p"));
    assert!(matches!(&root.children[1], TemplateChildNode::Element(p) if p.tag == "p"));
    let TemplateChildNode::Element(ul) = &root.children[2] else {
        panic!("Expected ul");
    };
    assert_eq!(ul.children.len(), 2);
    assert!(matches!(&ul.children[0], TemplateChildNode::Element(li) if li.tag == "li"));
    assert!(matches!(&ul.children[1], TemplateChildNode::Element(li) if li.tag == "li"));
}

#[test]
fn test_parse_nested_list_items_respect_list_item_scope() {
    for list_tag in ["ol", "ul"] {
        let allocator = Allocator::new();
        let source = format!(
            "<{list_tag}><li><span>outer</span><{list_tag}><li>inner</li></{list_tag}></li></{list_tag}>"
        );
        let (root, errors) = parse_document(&allocator, &source);

        assert!(
            errors.is_empty(),
            "unexpected errors for {list_tag}: {errors:?}"
        );
        assert_eq!(root.children.len(), 1);
        let TemplateChildNode::Element(list) = &root.children[0] else {
            panic!("Expected {list_tag}");
        };
        assert_eq!(list.tag, list_tag);
        assert_eq!(list.children.len(), 1);

        let TemplateChildNode::Element(outer_li) = &list.children[0] else {
            panic!("Expected outer li");
        };
        assert_eq!(outer_li.tag, "li");
        assert_eq!(outer_li.children.len(), 2);
        assert!(
            matches!(&outer_li.children[0], TemplateChildNode::Element(span) if span.tag == "span")
        );

        let TemplateChildNode::Element(inner_list) = &outer_li.children[1] else {
            panic!("Expected nested {list_tag}");
        };
        assert_eq!(inner_list.tag, list_tag);
        assert_eq!(inner_list.children.len(), 1);
        assert!(
            matches!(&inner_list.children[0], TemplateChildNode::Element(li) if li.tag == "li")
        );
    }
}
