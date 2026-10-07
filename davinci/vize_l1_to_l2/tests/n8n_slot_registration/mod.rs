//! Complete pinned stock packets, separate from legacy helper registration.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "complete authored and independently pinned compiler packets"
)]
use serde_json::Value;
use sha2::{Digest, Sha256};

const STOCK: &str = include_str!(
    "../../../../tests/_fixtures/differential/compiler/n8n-helper-props/slot-registration-controls.json"
);
const NAMES: [&str; 17] = [
    "earlier-branch-named-carrier-no-outlet-later-wrapped-outlet",
    "wrapped-outlet-no-carrier",
    "nested-conditional-outlet-no-carrier",
    "carrier-before-wrapped-vnode",
    "wrapped-vnode-before-carrier",
    "earlier-branch-vnode-before-carrier",
    "conditional-carrier-before-wrapped-outlet",
    "wrapped-outlet-before-conditional-carrier",
    "component-root-slot-content",
    "ordinary-template-without-slot-content",
    "slot-carrier-with-wrapped-outlet",
    "n8n-FormInput",
    "airi-animated-content",
    "existing-nested-conditional-slot-original",
    "existing-nested-slot-first",
    "existing-preceding-vnode-stays-first",
    "existing-preceding-sibling-stays-first",
];

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn authored_slot_carriers_preserve_complete_legacy_helper_registration() {
    assert_eq!(
        digest(STOCK.as_bytes()),
        "9d3bf28ef2a02ab0dcf3647b1ec17372eadd83f44b235286ef0648211eea69a0"
    );
    let stock: Value = serde_json::from_str(STOCK).expect("whole independent stock packets");
    assert_eq!(stock["compiler"]["version"], "3.5.26");
    assert_eq!(
        stock["compiler"]["sha256"],
        "4bd5cbcf9bdae4e4264be4ddb14e416074ad96909c56ba9b604e93d5b76b06ec"
    );
    let records = stock["records"]
        .as_array()
        .expect("exact full packet inventory");
    assert_eq!(records.len(), NAMES.len());
    let originals = stock["originals"]
        .as_array()
        .expect("complete original SFC and license bytes");
    assert_eq!(originals.len(), 2);
    let mut cases = Vec::new();
    for (record, name) in records.iter().zip(NAMES) {
        assert_eq!(record["name"], name);
        let template = record["source"].as_str().expect("authored template");
        assert_eq!(digest(template.as_bytes()), record["sourceSha256"]);
        let module = record["official"]["code"]
            .as_str()
            .expect("whole stock module");
        assert_eq!(digest(module.as_bytes()), record["officialModuleSha256"]);
        assert_eq!(record["official"]["errors"], serde_json::json!([]));
        let source = if let Some(original) = originals.iter().find(|row| row["name"] == name) {
            let original_source = original["source"].as_str().expect("whole original SFC");
            assert_eq!(digest(original_source.as_bytes()), original["sourceSha256"]);
            assert_eq!(original["templateSha256"], record["sourceSha256"]);
            let license = original["license"].as_str().expect("complete license");
            assert_eq!(digest(license.as_bytes()), original["licenseSha256"]);
            original_source.to_owned()
        } else {
            format!("<template>{template}</template>")
        };
        cases.push((name, source));
    }
    let inputs = cases
        .iter()
        .map(|(name, source)| (*name, source.as_str()))
        .collect::<Vec<_>>();
    // This compares complete native and forced-legacy output in all three
    // shipped option lanes. Stock import ordering is retained independently;
    // it is not substituted for Vize's legacy byte contract.
    super::assert_all_lanes(&inputs);
}
