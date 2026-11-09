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
