//! Exact Art native display successor from Full41129427590/job124316268671.
//! Preserve the historical corpus and compare the complete current vector.
use super::super::fix_history_contract::{Case, Diagnostic, Pack};
use super::super::fix_history_observation::digest;
use std::path::Path;

pub(super) fn expected(fixtures: &Path, frozen_pack: &[u8], case: &Case) -> Vec<Diagnostic> {
    assert_eq!(
        digest(frozen_pack),
        "bdf1c89cf693d30a0c5778ae10f7fb207f1a867242b7c1f490d40eac4c692834",
        "the complete historical Art pack stays frozen"
    );
    for (name, expected) in [
        (
            "tsconfig.json.txt",
            "e36b280dce185f4be85cbff1838c0ee06d8d7a46b38b3e63df995c0decd7b13b",
        ),
        (
            "src/MyCard.vue.txt",
            "455de116ba8d563f320c6b90d5a655a7c9cef4d8305e00e6705adf289420df35",
        ),
        (
            "src/MyCard.art.vue.txt",
            "8244d93074da638cd1c9f7a93845de4e7cf146f4da11b7c170ac7eff39ae3eea",
        ),
        (
            "src/MyCardExplicit.art.vue.txt",
            "224008e13f14210912b95eadfed63f36f18ffc1ee8108261f418d0f11bc9600a",
        ),
        (
            "src/MyCardValid.art.vue.txt",
            "766c26dee563a4eb211aded1d2f54a97d5dae4e0a40dfbd3bad655eb3d1bf834",
        ),
    ] {
        let bytes = std::fs::read(fixtures.join(name)).expect("the frozen Art input exists");
        assert_eq!(
            digest(&bytes),
            expected,
            "the original {name} bytes stay frozen"
        );
    }
    let bytes = std::fs::read(fixtures.join("native-display-current.json"))
        .expect("the separately recorded Art current vector exists");
    assert_eq!(
        digest(&bytes),
        "5963f62b417b944dadb54d0d4ead587714e9bf2e4725b2773e611fe887eb5c94",
        "the complete native current vector stays exact"
    );
    let current: Pack = serde_json::from_slice(&bytes).expect("the current Art pack is valid");
    assert_eq!(
        current.source_revision,
        "c4ee353b79611126138a82fb68ae187dd171ef70"
    );
    assert_eq!(
        current
            .cases
            .iter()
            .map(|case| case.id.as_str())
            .collect::<Vec<_>>(),
        [
            "implicit-and-explicit-props",
            "implicit-subject-avoids-unknown-global-diagnostics"
        ]
    );
    let selected = current
        .cases
        .into_iter()
        .find(|entry| entry.id == case.id)
        .expect("only the two recorded Art cases select this successor");
    assert_eq!(
        selected
            .inputs
            .iter()
            .map(|input| (&input.file, &input.source))
            .collect::<Vec<_>>(),
        case.inputs
            .iter()
            .map(|input| (&input.file, &input.source))
            .collect::<Vec<_>>()
    );
    assert_eq!(selected.diagnostics.len(), case.diagnostics.len());
    for (index, (before, after)) in case
        .diagnostics
        .iter()
        .zip(&selected.diagnostics)
        .enumerate()
    {
        assert_eq!(
            (
                &before.file,
                before.line,
                before.column,
                before.severity,
                before.code
            ),
            (
                &after.file,
                after.line,
                after.column,
                after.severity,
                after.code
            ),
            "Art diagnostic order, authored positions, severity and codes stay exact"
        );
        if index == 1 || index == 3 {
            let suffix = concat!(
                "'.\nProperty 'title' is missing in type '{ elevated: true; }' but required in type ",
                "'{ readonly title: string; readonly elevated?: boolean | undefined; }'."
            );
            assert_eq!(
                &before.message[before.message.len() - suffix.len()..],
                suffix
            );
            assert_eq!(&after.message[after.message.len() - suffix.len()..], suffix);
        } else {
            assert_eq!(
                before, after,
                "the complete wrong-value diagnostics stay exact"
            );
        }
    }
    selected.diagnostics
}
