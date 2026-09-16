use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor_with_diagnostics};
use vize_carton::Allocator;

#[test]
fn disabled_patterned_template_reports_transform_errors_without_code() {
    let allocator = Allocator::default();
    let source = "<template v-match=\"status\"><p v-when=\"1\">Ready</p><p v-when=\"_\">Waiting</p></template>";
    let (result, diagnostics) =
        compile_vapor_with_diagnostics(&allocator, source, VaporCompilerOptions::default());
    assert_eq!(result.code, "");
    assert_eq!(result.templates, Vec::<vize_carton::String>::new());
    assert_eq!(
        result.error_messages,
        [
            "`v-match` / `v-when` patterned templates require `experimentals.patternedTemplate`.",
            "`v-match` / `v-when` patterned templates require `experimentals.patternedTemplate`.",
            "`v-match` / `v-when` patterned templates require `experimentals.patternedTemplate`.",
        ]
    );
    assert_eq!(
        diagnostics
            .iter()
            .map(|error| error.message.as_str())
            .collect::<Vec<_>>(),
        result
            .error_messages
            .iter()
            .map(|message| message.as_str())
            .collect::<Vec<_>>()
    );

    let (result, diagnostics) = compile_vapor_with_diagnostics(
        &allocator,
        source,
        VaporCompilerOptions {
            experimental_patterned_template: true,
            ..Default::default()
        },
    );
    assert_eq!(result.error_messages, Vec::<vize_carton::String>::new());
    assert_eq!(diagnostics.len(), 0);
    assert!(!result.code.is_empty());
}
