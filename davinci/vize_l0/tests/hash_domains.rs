//! Exact hash v2 vectors; expected values require actual producer + independent capture.
#![expect(
    clippy::expect_used,
    clippy::disallowed_types,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::string_slice,
    reason = "fixture transport uses std strings and regressions fail closed"
)]
use serde_json::{Value, json};
use vize_davinci::key::{CachedArtifact, KeyManifest, source_block_key};
use vize_l0::hash::StableHasher128;
use vize_l2::summary::{
    AlphaEntry, AlphaPages, Facet, GlobalEntry, GlobalFacet, GlobalFacts, GlobalSummary,
    SfcSummary, Signature,
};

fn fixture() -> Value {
    serde_json::from_str(include_str!("fixtures/keys/hash-domains-v2.json")).unwrap()
}

fn hex(bytes: [u8; 16]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn preimage(row: &Value) -> Vec<u8> {
    let text = row["preimageHex"].as_str().unwrap();
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}

fn preimage_digest(row: &Value) -> String {
    let mut hash = StableHasher128::new();
    hash.update(&preimage(row));
    hex(hash.digest())
}

fn sfc() -> SfcSummary {
    let entry = || {
        vec![AlphaEntry {
            name: "Item".into(),
            contract: "string".into(),
        }]
    };
    SfcSummary::from_alpha(AlphaPages {
        signature: Signature {
            name: "Item".into(),
            params: "string".into(),
        },
        props: entry(),
        emits: entry(),
        slots: entry(),
        reactivity: entry(),
        components: entry(),
    })
    .unwrap()
}

fn global() -> GlobalSummary {
    let entry = || {
        vec![GlobalEntry {
            name: "Item".into(),
            contract: "string".into(),
        }]
    };
    GlobalSummary::from_facts(GlobalFacts {
        components: entry(),
        provides: entry(),
        directives: entry(),
    })
    .unwrap()
}

fn production(row: &Value) -> Option<String> {
    let producer = row["production"].as_str()?;
    Some(match producer {
        "source-block" => hex(source_block_key(
            "script",
            &[("setup", None), ("lang", Some("ts"))],
            "\nconst café = 1;\n",
        )
        .hash()),
        "manifest-corsa" => {
            let mut manifest = KeyManifest::new();
            for input in CachedArtifact::CorsaSession.inputs() {
                manifest.set(*input, "");
            }
            hex(manifest.fingerprint(CachedArtifact::CorsaSession).unwrap())
        }
        "sfc-signature" => hex(sfc().fingerprint(Facet::Signature, "Item").unwrap().bytes()),
        "sfc-prop" => hex(sfc().fingerprint(Facet::Prop, "Item").unwrap().bytes()),
        "sfc-emit" => hex(sfc().fingerprint(Facet::Emit, "Item").unwrap().bytes()),
        "sfc-slot" => hex(sfc().fingerprint(Facet::Slot, "Item").unwrap().bytes()),
        "sfc-reactivity" => hex(sfc()
            .fingerprint(Facet::Reactivity, "Item")
            .unwrap()
            .bytes()),
        "sfc-component" => hex(sfc().fingerprint(Facet::Component, "Item").unwrap().bytes()),
        "global-component" => hex(global()
            .fingerprint(GlobalFacet::Component, "Item")
            .unwrap()
            .bytes()),
        "global-provide" => hex(global()
            .fingerprint(GlobalFacet::Provide, "Item")
            .unwrap()
            .bytes()),
        "global-directive" => hex(global()
            .fingerprint(GlobalFacet::Directive, "Item")
            .unwrap()
            .bytes()),
        other => panic!("unknown vector producer {other}"),
    })
}

#[test]
fn exact_v2_digests_match_fixed_preimages_and_real_producers() {
    let fixture = fixture();
    assert_eq!(fixture["state"], "captured-source-and-independent-one-shot");
    let rows = fixture["vectors"].as_array().unwrap();
    for row in rows {
        let expected = row["digestHex"]
            .as_str()
            .expect("actual digest capture required");
        assert_eq!(preimage_digest(row), expected, "{}", row["id"]);
        if let Some(actual) = production(row) {
            assert_eq!(actual, expected, "{}", row["id"]);
        }
        if let Some(id) = row["mustDifferFrom"].as_str() {
            let original = rows.iter().find(|r| r["id"] == id).unwrap();
            assert_ne!(
                expected,
                original["digestHex"].as_str().unwrap(),
                "{}",
                row["id"]
            );
        }
    }
}

#[test]
#[ignore = "explicit capture only; this is not an acceptance gate"]
fn observe_pending_hash_domain_vectors() {
    for row in fixture()["vectors"].as_array().unwrap() {
        let computed = preimage_digest(row);
        if let Some(actual) = production(row) {
            assert_eq!(actual, computed, "preimage mismatch {}", row["id"]);
        }
        println!(
            "HASH_DOMAIN_OBSERVATION {}",
            json!({ "id": row["id"], "preimageHex": row["preimageHex"], "digestHex": computed, "productionDigestHex": production(row) })
        );
    }
}
