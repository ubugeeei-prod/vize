use super::Linter;

#[test]
fn direct_table_section_cells_do_not_report_foster_parenting() {
    let linter = Linter::new().with_enabled_rules(Some(Vec::new()));
    let source = "<table><thead><th>Name</th><th>Grade</th></thead></table>";
    let result = linter.lint_template(source, "test.vue");

    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name != "parser/template"),
        "direct cells remain under the authored table section: {:?}",
        result.diagnostics
    );
}

fn element_tags<'a>(ops: &[vize_l2::op::Op<'a>], out: &mut Vec<&'a str>) {
    for op in ops {
        if let vize_l2::op::Op::Element(element) = op {
            out.push(element.tag);
            element_tags(&element.children.ops, out);
        }
    }
}

#[test]
fn sfc_templates_keep_authored_rows_and_html_documents_keep_parser_owners() {
    let source = "<table><tr><td>row</td></tr></table>";
    let allocator = vize_l0::Allocator::new();

    let mut component = Vec::new();
    let template = crate::markup::L2Template::lower(&allocator, source);
    element_tags(&template.lowered().artifact.root.ops, &mut component);
    assert_eq!(component, ["table", "tr", "td"]);

    let mut document = Vec::new();
    let page = crate::markup::L2Template::lower_in_dom(&allocator, source);
    element_tags(&page.lowered().artifact.root.ops, &mut document);
    assert_eq!(document, ["table", "tbody", "tr", "td"]);

    let linter = Linter::new().with_enabled_rules(Some(Vec::new()));
    for result in [
        linter.lint_template(source, "test.vue"),
        linter.lint_standalone_html(source, "index.html"),
    ] {
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    }
}
