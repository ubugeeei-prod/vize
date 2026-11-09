use vize_atelier_core::TemplateSyntaxMode;
use vize_atelier_dom::{DomCompilerOptions, compile_template_with_template_syntax};
use vize_l0::Allocator;

#[test]
fn authored_table_section_cells_are_not_wrapped_in_an_implicit_row() {
    let source = "<table><thead><th>Name</th><th>Grade</th></thead></table>";

    for syntax in [TemplateSyntaxMode::Standard, TemplateSyntaxMode::Quirks] {
        let allocator = Allocator::new();
        let (_, errors, result) = compile_template_with_template_syntax(
            &allocator,
            source,
            DomCompilerOptions::default(),
            syntax,
        );

        assert!(errors.is_empty(), "{syntax:?}: {errors:?}");
        assert!(
            result.code.contains("\"thead\", null, ["),
            "{syntax:?}: {}",
            result.code
        );
        assert_eq!(
            result.code.matches("\"tr\"").count(),
            0,
            "{syntax:?}: {}",
            result.code
        );
        assert_eq!(
            result.code.matches("\"th\"").count(),
            2,
            "{syntax:?}: {}",
            result.code
        );
    }
}

#[test]
fn quirks_preserves_adjacent_table_control_flow() {
    let allocator = Allocator::new();
    let source = r#"<table>
  <template v-if="hasRows"><tr><td>row</td></tr></template>
  <tr v-else><td>empty</td></tr>
</table>"#;
    let (_, errors, result) = compile_template_with_template_syntax(
        &allocator,
        source,
        DomCompilerOptions::default(),
        TemplateSyntaxMode::Quirks,
    );

    assert!(errors.is_empty(), "{errors:?}");
    assert!(result.code.contains("hasRows"));
    assert!(result.code.contains("? (_openBlock()"));
    assert!(result.code.contains(": (_openBlock()"));
    assert!(result.code.contains("\"tr\", { key: 1 }"));
}
