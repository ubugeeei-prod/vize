use crate::ir::{LintDocumentKind, S1InputDialect};

#[test]
fn kind_maps_onto_s1_input_dialects() {
    assert_eq!(
        LintDocumentKind::VueSfc.s1_input_dialect(),
        Some(S1InputDialect::VueTemplate)
    );
    assert_eq!(
        LintDocumentKind::MarkupFragment.s1_input_dialect(),
        Some(S1InputDialect::VueTemplate)
    );
    assert_eq!(
        LintDocumentKind::ScriptModule.s1_input_dialect(),
        Some(S1InputDialect::Jsx)
    );
    assert_eq!(LintDocumentKind::AstroComponent.s1_input_dialect(), None);
    assert_eq!(LintDocumentKind::SvelteComponent.s1_input_dialect(), None);
    assert_eq!(
        S1InputDialect::from_template_lang(Some("pug")),
        S1InputDialect::Pug
    );
    assert_eq!(
        S1InputDialect::from_template_lang(Some(" HTML ")),
        S1InputDialect::VueTemplate
    );
    assert_eq!(
        S1InputDialect::from_template_lang(None),
        S1InputDialect::VueTemplate
    );
}
