//! P5-3 — the project summary.
//!
//! `cargo test -p vize_davinci --test global_summary`
//!
//! Adding a global component invalidates exactly the files that resolve it.

#![expect(clippy::expect_used, reason = "tests assert by panicking")]

use vize_davinci::summary::{
    AlphaEntry, AlphaPages, Facet, GlobalEntry, GlobalError, GlobalFacet, GlobalFacts,
    GlobalResolution, GlobalSummary, SfcSummary, Signature,
};
use vize_s0::String;

fn entry(name: &str, contract: &str) -> GlobalEntry {
    GlobalEntry {
        name: String::from(name),
        contract: String::from(contract),
    }
}

fn facts(
    components: &[(&str, &str)],
    provides: &[(&str, &str)],
    directives: &[(&str, &str)],
) -> GlobalSummary {
    GlobalSummary::from_facts(GlobalFacts {
        components: components
            .iter()
            .map(|(name, contract)| entry(name, contract))
            .collect(),
        provides: provides
            .iter()
            .map(|(name, contract)| entry(name, contract))
            .collect(),
        directives: directives
            .iter()
            .map(|(name, contract)| entry(name, contract))
            .collect(),
    })
    .expect("project facts")
}

fn resolutions(summary: &GlobalSummary, files: &[(&str, &[&str])]) -> Vec<GlobalResolution> {
    files
        .iter()
        .map(|(file, names)| {
            let resolved = names
                .iter()
                .map(|name| (GlobalFacet::Component, *name))
                .collect::<Vec<_>>();
            GlobalResolution::record(file, summary, &resolved).expect("resolution")
        })
        .collect()
}

#[test]
fn adding_a_global_component_invalidates_exactly_its_resolvers() {
    let before = facts(&[("RouterLink", "vue-router")], &[], &[]);
    let files = [
        ("app.vue", &[][..]),
        ("host.vue", &["AppDialog"][..]),
        ("nav.vue", &["RouterLink"][..]),
        ("page.vue", &["AppDialog"][..]),
    ];
    let recorded = resolutions(&before, &files);
    assert!(before.invalidated(&recorded).is_empty());
    assert_eq!(
        before.fingerprint(GlobalFacet::Component, "AppDialog"),
        None
    );

    let router = before
        .fingerprint(GlobalFacet::Component, "RouterLink")
        .expect("router link");
    let after = facts(
        &[("AppDialog", "local"), ("RouterLink", "vue-router")],
        &[],
        &[],
    );
    assert_eq!(
        after.fingerprint(GlobalFacet::Component, "RouterLink"),
        Some(router)
    );
    assert_eq!(after.invalidated(&recorded), ["host.vue", "page.vue"]);

    let rerecorded = resolutions(&after, &files);
    let with_unrelated = facts(
        &[
            ("AppDialog", "local"),
            ("AppToast", "local"),
            ("RouterLink", "vue-router"),
        ],
        &[("theme", "string")],
        &[("focus", "void")],
    );
    assert!(with_unrelated.invalidated(&rerecorded).is_empty());

    let page = SfcSummary::from_alpha(AlphaPages {
        signature: Signature {
            name: String::from("Page"),
            params: String::from(""),
        },
        props: vec![AlphaEntry {
            name: String::from("label"),
            contract: String::from("string"),
        }],
        emits: Vec::new(),
        slots: Vec::new(),
        reactivity: Vec::new(),
        components: vec![AlphaEntry {
            name: String::from("AppDialog"),
            contract: String::from("local"),
        }],
    })
    .expect("page summary");
    let label = page.fingerprint(Facet::Prop, "label").expect("label");
    assert_ne!(
        page.fingerprint(Facet::Component, "AppDialog"),
        after.fingerprint(GlobalFacet::Component, "AppDialog")
    );
    assert_eq!(page.fingerprint(Facet::Prop, "label"), Some(label));
}

#[test]
fn a_duplicate_global_component_is_refused() {
    let error = GlobalSummary::from_facts(GlobalFacts {
        components: vec![entry("AppDialog", "local"), entry("AppDialog", "other")],
        provides: Vec::new(),
        directives: Vec::new(),
    });
    assert_eq!(
        error,
        Err(GlobalError::Duplicate {
            facet: GlobalFacet::Component,
            name: String::from("AppDialog"),
        })
    );
}
