use super::{find_element, parse, parse_document};
use vize_l0::Allocator;
use vize_relief::TemplateChildNode;

const DIRECT_SECTION_CELLS: &str = "<table><thead><th>Name</th><th>Grade</th></thead></table>";

#[test]
fn template_parse_keeps_authored_cells_direct() {
    let allocator = Allocator::new();
    let (root, errors) = parse(&allocator, DIRECT_SECTION_CELLS);

    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    let TemplateChildNode::Element(table) = &root.children[0] else {
        panic!("Expected table");
    };
    let TemplateChildNode::Element(thead) = &table.children[0] else {
        panic!("Expected authored thead");
    };
    assert_eq!(thead.tag, "thead");
    assert_eq!(thead.children.len(), 2);
    assert!(
        thead
            .children
            .iter()
            .all(|child| matches!(child, TemplateChildNode::Element(cell) if cell.tag == "th"))
    );
}

#[test]
fn document_parse_keeps_html_implicit_row() {
    let allocator = Allocator::new();
    let (root, errors) = parse_document(&allocator, DIRECT_SECTION_CELLS);

    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    let thead = find_element(&root.children, "thead").expect("authored thead");
    let TemplateChildNode::Element(row) = &thead.children[0] else {
        panic!("Expected implicit row");
    };
    assert_eq!(row.tag, "tr");
    assert_eq!(row.children.len(), 2);
    assert!(
        row.children
            .iter()
            .all(|child| matches!(child, TemplateChildNode::Element(cell) if cell.tag == "th"))
    );
}
