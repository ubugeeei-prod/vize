//! A new compiler child role must not change ordinary template reference discovery.
use vize_patina::Linter;
#[test]
fn ordinary_template_and_v_text_keep_the_complete_undefined_reference_diagnostic() {
    let source = include_str!(
        "../../../tests/_fixtures/differential/linter/jsx-child-expression-control-6890/Layout.vue.txt"
    );
    let result = Linter::new()
        .with_enabled_rules(Some(vec!["vue/no-undefined-refs".into()]))
        .lint_sfc(source, "Layout.vue");
    assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
    let diagnostic = &result.diagnostics[0];
    assert_eq!(diagnostic.rule_name, "vue/no-undefined-refs");
    assert_eq!(diagnostic.message, "Variable 'missing' is not defined");
    let start = source.find("missing").unwrap() as u32;
    assert_eq!((diagnostic.start, diagnostic.end), (start, start + 7));
    assert_eq!(result.error_count + result.warning_count, 1);
}
