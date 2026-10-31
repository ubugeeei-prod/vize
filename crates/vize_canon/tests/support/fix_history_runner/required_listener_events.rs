//! Exact no-emits listener successor from Full41137319424/job124342873996.
//! Only the frozen false Event diagnostics change; actual vectors stay intact.
use super::super::fix_history_contract::{Case, Diagnostic, Pack};
use super::super::fix_history_observation::digest;
use std::path::Path;

pub(super) fn expected(fixtures: &Path, frozen_pack: &[u8], case: &Case) -> Vec<Diagnostic> {
    assert_eq!(
        digest(frozen_pack),
        "7b2175e25db8e5924133ff31d4ade064341d10bff923eeb002256d63277c210c",
        "the complete original listener pack stays frozen"
    );
    for (name, expected) in [
        (
            "tsconfig.json.txt",
            "20607bc6729ba818893549d0567ca341f57cdf1eb56e34e176d2b2205e7002e4",
        ),
        (
            "src/Child.vue.txt",
            "c06aaefcc168971c448e95ac6354ea792a7e5e7227de5f52d7372c9f354821fb",
        ),
        (
            "src/App.vue.txt",
            "f74d671621c4509482b2d9dfcf499be3b193b2ff840dd5a254e16a8629a0b2b7",
        ),
    ] {
        let bytes = std::fs::read(fixtures.join(name)).expect("the original listener input exists");
        assert_eq!(
            digest(&bytes),
            expected,
            "the original {name} bytes stay frozen"
        );
    }
    let bytes = std::fs::read(fixtures.join("native-event-current.json"))
        .expect("the separately recorded native listener vector exists");
    assert_eq!(
        digest(&bytes),
        "6ba0345ffe593207a8e114c8834fd7fc8da97758d13a7c7068203713b65bf7df",
        "the complete native listener vector stays exact"
    );
    let current: Pack = serde_json::from_slice(&bytes).expect("the current listener pack is valid");
    assert_eq!(
        current.source_revision,
        "90800904bcfbf3ad073ba95349852ee71ffb37e2"
    );
    assert_eq!(current.cases.len(), 1);
    let selected = current
        .cases
        .into_iter()
        .next()
        .expect("the recorded listener case exists");
    assert_eq!(selected.id, "required-boolean-with-listener");
    assert_eq!(selected.id, case.id);
    assert_eq!(
        selected
            .inputs
            .iter()
            .map(|i| (&i.file, &i.source))
            .collect::<Vec<_>>(),
        case.inputs
            .iter()
            .map(|i| (&i.file, &i.source))
            .collect::<Vec<_>>()
    );
    assert_eq!(case.diagnostics.len(), 4);
    assert_eq!(
        selected.diagnostics.as_slice(),
        &case.diagnostics[2..],
        "both complete authored prop errors stay unchanged; only the two frozen no-emits Event errors are obsolete"
    );
    selected.diagnostics
}
