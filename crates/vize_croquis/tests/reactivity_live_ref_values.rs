//! Legacy deep-ref and DOM mutation corpus, retaining shallow/raw/scalar controls.

use serde::Deserialize;
use vize_croquis::{reactivity::ReactivityLossKind, script_parser::parse_script_setup};
use vize_l0::String;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    source: String,
    mutations: Vec<String>,
}

#[test]
fn member_mutations_preserve_live_ref_values() {
    let cases: Vec<Case> = serde_json::from_str(include_str!(
        "fixtures/reactivity-live-ref-values/cases.json"
    ))
    .expect("valid lexical provenance corpus");
    assert_eq!(cases.len(), 15);
    for case in cases {
        let parsed = parse_script_setup(&case.source);
        let mutations: Vec<_> = parsed
            .reactivity
            .losses()
            .iter()
            .filter_map(|loss| match &loss.kind {
                ReactivityLossKind::PlainValueAlias {
                    alias_name,
                    target_name,
                    ..
                } if alias_name == "<mutation>" => Some(target_name.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(
            mutations,
            case.mutations
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            "{}",
            case.id
        );
    }
}
