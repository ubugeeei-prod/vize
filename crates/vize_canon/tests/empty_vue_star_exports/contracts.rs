//! Exact vectors for known semantics; refusal boundaries have separate credit.

use super::{execution, projection, support};
use serde_json::{Value, json};

const CASES: &[&str] = &[
    "positive-authentic",
    "wrong-prop-authentic",
    "authored-shared-duplicate",
    "concrete-direct-types",
    "classic-default-only",
    "namespace-typed",
    "authored-named-refusal",
    "authored-reserved-props-refusal",
    "config-strict",
    "config-permissive",
    "malformed-refusal",
    "script-src-refusal",
    "indirect-refusal",
    "alias-refusal",
    "extensionless-refusal",
    "missing-refusal",
];

fn authentic_packets() -> Vec<Value> {
    support::value("authentic/receipt.json")["packets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|path| support::value(path.as_str().unwrap()))
        .collect()
}

pub(super) fn authentic_custody() {
    let receipt = support::value("authentic/receipt.json");
    assert_eq!(
        receipt["sourcePacketSHA256"],
        "a5e32916175feeb304fc88744cb5fcd94ca5f39ee385dad5ed573022d893129e"
    );
    let raw = support::body(&receipt["sourcePacketWholeBytes"]);
    assert_eq!(
        support::hash(raw.as_bytes()),
        receipt["sourcePacketSHA256"].as_str().unwrap()
    );
    let original: Value = serde_json::from_str(&raw).unwrap();
    let mut restored = receipt["metadata"].clone();
    let providers = receipt["providers"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(path, record)| (path.clone(), support::blob(record)))
        .collect::<serde_json::Map<_, _>>();
    restored["vueProviderWholeBodies"] = Value::Object(providers);
    let packets = authentic_packets();
    assert_eq!(packets.len(), 6);
    let names = [
        "positive-before",
        "positive-counterfactual",
        "wrong-prop-before",
        "wrong-prop-counterfactual",
        "authored-duplicate-before",
        "authored-duplicate-counterfactual",
    ];
    let expected_codes = [
        vec![2308, 2308, 2308],
        vec![],
        vec![2308, 2308, 2308, 2322, 2322],
        vec![2322, 2322],
        vec![2308, 2308, 2308, 2308],
        vec![2308],
    ];
    let mut whole = Vec::new();
    for (index, mut packet) in packets.into_iter().enumerate() {
        assert_eq!(packet["name"], names[index]);
        let stdout = packet["wholeStdoutUtf8"].as_str().unwrap();
        assert_eq!(
            support::hash(stdout.as_bytes()),
            packet["stdoutSha256"].as_str().unwrap()
        );
        let stderr = packet["wholeStderrBytes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|byte| u8::try_from(byte.as_u64().unwrap()).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            support::hash(&stderr),
            packet["stderrSha256"].as_str().unwrap()
        );
        let codes = support::raw_rows(stdout)
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["code"].as_u64().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(codes, expected_codes[index]);
        let inputs = packet["inputs"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(path, record)| (path.clone(), support::blob(record)))
            .collect::<serde_json::Map<_, _>>();
        assert_eq!(inputs["A.vue"]["bytes"], 233);
        assert_eq!(inputs["A.vue"], inputs["B.vue"]);
        packet["inputs"] = Value::Object(inputs);
        whole.push(packet);
    }
    restored["rows"] = json!(whole);
    assert_eq!(
        restored, original,
        "every complete packet/input/provider/output field must survive losslessly"
    );
    let supplement = support::value("supplement.json");
    assert_eq!(supplement["cases"], json!(CASES));
    for credit in [
        "historical32Credit",
        "legacyAdapterCredit",
        "fullFixHistoryClosureCredit",
    ] {
        assert_eq!(supplement[credit], false);
    }
}

fn expected(case: &Value) -> Value {
    if let Some(name) = case["expectedAuthenticPacket"].as_str() {
        let packets = authentic_packets();
        let packet = packets
            .iter()
            .find(|packet| packet["name"] == name)
            .unwrap();
        support::raw_rows(packet["wholeStdoutUtf8"].as_str().unwrap())
    } else {
        case["diagnostics"].clone()
    }
}

pub(super) fn production(runtime: &support::Runtime, kind: &str) {
    authentic_custody();
    let mut observed = 0;
    for id in CASES {
        let case = support::value(&format!("production/{id}/case.json"));
        assert_eq!(case["id"], *id);
        if case["nativeContract"] != kind {
            continue;
        }
        observed += 1;
        let observation = execution::check(&case, runtime);
        let mut packet = observation.packet;
        projection::plain(&case, &observation.generated);
        if kind == "exact" {
            packet["wholeGeneratedModuleExportObservations"] =
                projection::exports(&case, &observation.generated);
            let expected = expected(&case);
            let authentic_raw = case["expectedAuthenticRawPacket"].as_str().map(|name| {
                authentic_packets()
                    .into_iter()
                    .find(|packet| packet["name"] == name)
                    .unwrap()
            });
            if let Some(authentic) = &authentic_raw {
                packet["rawDiagnosticCoordinateIdentityProof"] =
                    projection::raw_identity(&case, &observation.generated, authentic);
            }
            let actual = observation
                .rows
                .expect("normal native production must complete");
            assert_eq!(
                actual, expected,
                "{id}: complete authored diagnostic vector"
            );
            let cli = &packet["wholeSameConfigNativeCli"];
            assert_eq!(
                cli["wholeStderrBytes"],
                json!([]),
                "{id}: unexpected native stderr"
            );
            let raw = support::raw_rows(cli["wholeStdoutUtf8"].as_str().unwrap());
            let raw_status = if let Some(authentic) = authentic_raw {
                assert_eq!(
                    raw,
                    support::raw_rows(authentic["wholeStdoutUtf8"].as_str().unwrap()),
                    "{id}: complete authentic raw native oracle; filenames only normalized"
                );
                "exact authentic generated-coordinate vector with whole diagnostic-source body identity"
            } else if expected.as_array().unwrap().is_empty() {
                assert_eq!(cli["wholeStdoutUtf8"], "");
                "complete empty raw native vector"
            } else {
                "unrecorded generated-coordinate oracle; full raw vector observation only"
            };
            let success = expected.as_array().unwrap().is_empty();
            assert_eq!(packet["success"], success);
            assert_eq!(packet["exitCode"], if success { 0 } else { 1 });
            assert_eq!(cli["exitCode"], if success { 0 } else { 1 });
            packet["wholeRawGeneratedCoordinateDiagnosticVector"] = raw;
            packet["nativeRawOracleStatus"] = json!(raw_status);
            packet["nativeAcceptance"] = json!(
                "complete authored production vector accepted; raw oracle status is separate; no LSP/API/PID authority credit"
            );
            packet["phase"] = json!("EXACT_AUTHORED_AND_SCOPED_RAW_ORACLES_ACCEPTED");
        } else {
            // No fabricated diagnostic golden: every native result is retained,
            // but this law proves only refusal at the explicitly unproved edge.
            packet["nativeAcceptance"] =
                json!("projection refusal accepted; boundary semantic vector observation only");
            packet["phase"] =
                json!("EXACT_PROJECTION_REFUSAL_ACCEPTED_NATIVE_SEMANTICS_UNQUALIFIED");
        }
        support::capture(id, &packet);
    }
    assert_eq!(observed, if kind == "exact" { 10 } else { 6 });
}
