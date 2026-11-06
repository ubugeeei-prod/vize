//! Exact generated successors after eliding the unused Vue 3 instance alias.
use super::support;
use serde_json::{Value, json};
use std::fs;

const BEFORE: &str = "type $ComponentInstance = {\n  $props: Props;\n  readonly __vizeRawProps?: Props;\n  $emit: $PublicEmit<Emits>;\n  $slots: $PublicSlots<$Slots>;\n} & $ComponentPublicBase;\ntype $ComponentConstructor = new <$AuthoredProps = unknown, $JsxProps = unknown>(props?: $AuthoredProps & $ComponentInput<Props, {}, $AuthoredProps>, ...args: any[]) => $ComponentInstanceFor<$ComponentInstance, Props, {}, $AuthoredProps, $JsxProps>;";
const AFTER: &str = "type $ComponentConstructor = new <$AuthoredProps = unknown, $JsxProps = unknown>(props?: $AuthoredProps & $ComponentInput<Props, {}, $AuthoredProps>, ...args: any[]) => $ComponentInstanceFor<{\n  $props: Props;\n  readonly __vizeRawProps?: Props;\n  $emit: $PublicEmit<Emits>;\n  $slots: $PublicSlots<$Slots>;\n}, Props, {}, $AuthoredProps, $JsxProps> & $ComponentPublicBase;";

fn load(path: &str, sha256: &str) -> Value {
    let bytes = fs::read(support::corpus().join(path)).unwrap();
    assert_eq!(support::hash(&bytes), sha256);
    serde_json::from_slice(&bytes).unwrap()
}

fn integrated(previous: &Value, record: &Value, entry: &Value) -> String {
    let historical = previous["afterWholeUtf8"].as_str().unwrap();
    let before = record["edit"]["before"].as_str().unwrap();
    let after = record["edit"]["after"].as_str().unwrap();
    assert_eq!(historical.matches(before).count(), 1);
    let current = historical.replacen(before, after, 1);
    assert_eq!(json!(current.len()), entry["after"][1]);
    assert_eq!(support::hash(current.as_bytes()), entry["after"][2]);
    current
}

pub(super) fn apply(manifest: &mut Value, members: &mut [Value]) {
    let owned = load(
        "malformed-membership/owned-constructor-20261105.json",
        "377ce83df86cc659c5e5e9dfa94ef4d3a7265d5048aaa37f399d48be911c619b",
    );
    let integrated_record = load(
        "malformed-membership/integrated-constructor-11964.json",
        "8b744fba21c4b1c1f12865cfafd6788bc06c06b8f0c765008fa3c97db9c85cd4",
    );
    let expected = [
        json!([
            "project/A.vue.ts",
            3804,
            "ef495b4f7e3f9d2b60dceba9f52d95bf25240de7011c611569b301a0edf239a7"
        ]),
        json!([
            "project/B.vue.ts",
            4024,
            "3bbde2cbdf1b2dc80f3140c8d080b0e0099854d0f63c7a5ce0103e857a8271a5"
        ]),
    ];
    for (((previous, entry), current), (index, path)) in owned["entries"]
        .as_array()
        .unwrap()
        .iter()
        .zip(integrated_record["entries"].as_array().unwrap())
        .zip(expected)
        .zip([(74, "project/A.vue.ts"), (75, "project/B.vue.ts")])
    {
        let historical = integrated(previous, &integrated_record, entry);
        assert_eq!(historical.matches(BEFORE).count(), 1);
        assert_eq!(historical.matches(AFTER).count(), 0);
        let successor = historical.replacen(BEFORE, AFTER, 1);
        assert_eq!(historical.len() - successor.len(), 46);
        assert_eq!(current[0], path);
        assert_eq!(json!(successor.len()), current[1]);
        assert_eq!(support::hash(successor.as_bytes()), current[2]);
        let member = members.get_mut(index).unwrap();
        assert_eq!(*member, entry["after"]);
        *member = current;
    }
    manifest["provenance"]["currentElidedInstance"] = json!({
        "pullRequest": 11984,
        "qualificationHead": "6f0a31207be9d22d0c9034d4672b9924c77040e1",
        "observedRun": 42184342147_i64,
        "observedJob": 127781275927_i64,
        "observedMismatchEntry": 74,
        "nativeAcceptanceCredit": false,
        "edit": {"before": BEFORE, "after": AFTER},
    });
}

#[test]
fn elided_instance_transition_rejects_missing_or_duplicate_edits() {
    for changed in [
        AFTER.replace("$props: Props;\n", ""),
        format!("{AFTER}\n{AFTER}"),
    ] {
        assert_ne!(changed.matches(AFTER).count(), 1);
    }
}
