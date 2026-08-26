use vize_atelier_core::TemplateSyntaxMode;
use vize_atelier_dom::{DomCompilerOptions, compile_template_with_template_syntax};
use vize_s0::Allocator;

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
