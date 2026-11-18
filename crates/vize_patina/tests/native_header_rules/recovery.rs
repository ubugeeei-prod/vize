use super::support::{
    HeaderRule, LOCALES, NeverLookup, complete, expected, registered, selected, span, warning,
};
use vize_l0::Allocator;
use vize_patina::native::{NativeLintRefusal, NativeSyntaxLint};

#[test]
fn nested_form_refusal_keeps_complete_facade_warnings_without_a_parser_error() {
    // Vue keeps the nested form, so its attributes are linted and no parser
    // error is reported.
    let source = "<template><form><form autofocus accesskey='a' /></form></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let outer = owner.children().next().unwrap().into_element().unwrap();
    let form = outer.children().next().unwrap().into_element().unwrap();
    let receipt = form.tag_facts().unwrap();
    assert!(receipt.in_recovery_context());
    for rule in HeaderRule::ALL {
        assert_eq!(
            rule.check(&lint, &form, &NeverLookup).err(),
            Some(NativeLintRefusal::RecoveryContext {
                span: receipt.span()
            })
        );
        for locale in LOCALES {
            let mut diagnostics = Vec::new();
            if !matches!(rule, HeaderRule::Distracting) {
                let range = if matches!(rule, HeaderRule::AccessKey) {
                    span(source, "accesskey='a'")
                } else {
                    span(source, "autofocus")
                };
                diagnostics.push(warning(rule, locale, range, "form"));
            }
            assert_eq!(
                complete(&registered(source, locale, rule)),
                expected(diagnostics)
            );
        }
    }
}
