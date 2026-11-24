//! Rows and cells written directly in a `<table>`: a component template keeps
//! them as authored (Vue's compiler emits `table > tr`); a document and an
//! explicit `html_tree_construction` parse get the HTML parser's owners.

use vize_l0::Allocator;
use vize_relief::{ElementNode, TemplateChildNode, options::ParserOptions};

use super::{find_element, parse, parse_document, parse_with_options};

const DIRECT_ROWS: &str = "<table><tr><td>x</td></tr><tr><td>y</td></tr></table>";
const DIRECT_CELLS: &str = "<table><td>x</td><th>y</th></table>";

fn table<'a, 'b>(children: &'b [TemplateChildNode<'a>]) -> &'b ElementNode<'a> {
    find_element(children, "table").expect("authored table")
}

fn child_tags<'a>(element: &ElementNode<'a>) -> Vec<&'a str> {
    element
        .children
        .iter()
        .filter_map(|child| match child {
            TemplateChildNode::Element(element) => Some(element.tag),
            _ => None,
        })
        .collect()
}

#[test]
fn component_parse_keeps_direct_rows() {
    let allocator = Allocator::new();
    let (root, errors) = parse(&allocator, DIRECT_ROWS);

    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    let table = table(&root.children);
    assert_eq!(child_tags(table), ["tr", "tr"]);
    let TemplateChildNode::Element(row) = &table.children[0] else {
        panic!("Expected authored row");
    };
    let TemplateChildNode::Element(cell) = &row.children[0] else {
        panic!("Expected td");
    };
    assert!(matches!(&cell.children[0], TemplateChildNode::Text(text) if text.content == "x"));
}

#[test]
fn component_parse_keeps_direct_cells() {
    let allocator = Allocator::new();
    let (root, errors) = parse(&allocator, DIRECT_CELLS);

    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    assert_eq!(child_tags(table(&root.children)), ["td", "th"]);
}

#[test]
fn document_parse_inserts_implicit_tbody() {
    let allocator = Allocator::new();
    let (root, errors) = parse_document(&allocator, DIRECT_ROWS);

    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    let table = table(&root.children);
    assert_eq!(child_tags(table), ["tbody"]);
    let TemplateChildNode::Element(tbody) = &table.children[0] else {
        panic!("Expected implicit tbody");
    };
    assert_eq!(child_tags(tbody), ["tr", "tr"]);
}

#[test]
fn html_tree_construction_restore_the_html_owners() {
    let options = || ParserOptions {
        html_tree_construction: true,
        ..ParserOptions::default()
    };
    let allocator = Allocator::new();

    let (root, errors) = parse_with_options(&allocator, DIRECT_ROWS, options());
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    let table_node = table(&root.children);
    assert_eq!(child_tags(table_node), ["tbody"]);
    let TemplateChildNode::Element(tbody) = &table_node.children[0] else {
        panic!("Expected implicit tbody");
    };
    assert_eq!(child_tags(tbody), ["tr", "tr"]);

    let (root, errors) = parse_with_options(&allocator, DIRECT_CELLS, options());
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    let tbody = find_element(&root.children, "tbody").expect("implicit tbody");
    assert_eq!(child_tags(tbody), ["tr"]);
    let row = find_element(&tbody.children, "tr").expect("implicit row");
    assert_eq!(child_tags(row), ["td", "th"]);

    // Cells under an authored section stay direct.
    let (root, errors) = parse_with_options(
        &allocator,
        "<table><thead><th>Name</th></thead></table>",
        options(),
    );
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
    let thead = find_element(&root.children, "thead").expect("authored thead");
    assert_eq!(child_tags(thead), ["th"]);
}
