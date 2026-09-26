use crate::ir::{L1InputDialect, LintDocumentKind};

#[test]
fn kind_maps_onto_l1_input_dialects() {
    assert_eq!(
        LintDocumentKind::VueSfc.l1_input_dialect(),
        Some(L1InputDialect::VueTemplate)
    );
    assert_eq!(
        LintDocumentKind::MarkupFragment.l1_input_dialect(),
        Some(L1InputDialect::VueTemplate)
    );
    assert_eq!(
        LintDocumentKind::ScriptModule.l1_input_dialect(),
        Some(L1InputDialect::Jsx)
    );
    assert_eq!(LintDocumentKind::AstroComponent.l1_input_dialect(), None);
    assert_eq!(LintDocumentKind::SvelteComponent.l1_input_dialect(), None);
    assert_eq!(
        L1InputDialect::from_template_lang(Some("pug")),
        L1InputDialect::Pug
    );
    assert_eq!(
        L1InputDialect::from_template_lang(Some(" HTML ")),
        L1InputDialect::VueTemplate
    );
    assert_eq!(
        L1InputDialect::from_template_lang(None),
        L1InputDialect::VueTemplate
    );
}
