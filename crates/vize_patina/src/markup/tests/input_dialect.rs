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
    for (lang, expected) in [
        (None, Some(L1InputDialect::VueTemplate)),
        (Some("HTML"), Some(L1InputDialect::VueTemplate)),
        (Some("pug"), Some(L1InputDialect::Pug)),
        (Some("JADE"), Some(L1InputDialect::Pug)),
        // The registry matches an enumerated value: padding is no language,
        // and an unknown lang is opaque rather than the Vue template.
        (Some(" JADE "), None),
        (Some("haml"), None),
    ] {
        assert_eq!(
            L1InputDialect::from_template_lang(lang),
            expected,
            "{lang:?}"
        );
    }
}
