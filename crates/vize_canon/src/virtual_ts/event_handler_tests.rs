use super::{
    VirtualTsCheckOptions, VirtualTsGenerationOptions, VirtualTsOptions,
    generate_virtual_ts_with_offsets_and_checks,
};
use vize_croquis::{Analyzer, AnalyzerOptions};

fn generate_unchecked_emit_handler(script: &str, template: &str) -> vize_carton::String {
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);

    generate_virtual_ts_with_offsets_and_checks(
        &analyzer.finish(),
        Some(script),
        Some(&root),
        0,
        0,
        &VirtualTsOptions::default(),
        VirtualTsGenerationOptions {
            check_options: VirtualTsCheckOptions {
                check_props: false,
                check_template_bindings: true,
                check_emits: false,
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .code
}

fn assert_parseable(code: &str) {
    let allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, code, oxc_span::SourceType::ts()).parse();
    assert!(
        !parsed.panicked && parsed.diagnostics.is_empty(),
        "event handlers must generate parseable TypeScript: {:#?}\n{code}",
        parsed.diagnostics
    );
}

#[test]
fn asi_separated_statement_event_handler_uses_handler_scope() {
    let script = r#"const emit = defineEmits<{
  create: []
  close: []
}>()
"#;
    let template = r#"<Child @create="
  emit('create')
  emit('close')
" />"#;
    let code = generate_unchecked_emit_handler(script, template);

    assert!(
        code.contains("// @create handler"),
        "ASI-separated handlers should get an event handler scope:\n{code}"
    );
    assert!(
        code.contains("emit('create')\n  emit('close')"),
        "both authored statements should be preserved:\n{code}"
    );
    assert!(
        !code.contains("void (\n  emit('create')"),
        "ASI-separated statements must not be parenthesized as one expression:\n{code}"
    );
    assert!(
        code.contains("($event: any)") && !code.contains("_listener"),
        "disabled emit checks should keep the handler without checking its payload:\n{code}"
    );
    assert_parseable(&code);
}

#[test]
fn disabled_emit_checks_preserve_expression_shaped_handlers() {
    let code = generate_unchecked_emit_handler(
        "function handler() {}",
        r#"<button
  @click="handler"
  @focus="function () {}"
/>"#,
    );

    assert!(
        code.contains("void (handler);  // handler expression (emit checks disabled)")
            && code
                .contains("void (function () {});  // handler expression (emit checks disabled)"),
        "expression-shaped handlers should remain expressions when emit checks are disabled:\n{code}"
    );
    assert!(
        !code.contains("handler($event)") && !code.contains("_listener"),
        "disabling emit checks must not invoke or assign the handler:\n{code}"
    );
    assert_parseable(&code);
}
