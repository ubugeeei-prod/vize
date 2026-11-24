//! Invert the #13113 authored component binding (`$AuthoredComponent extends
//! infer A extends object ? … & A & …`) before older custody.
use super::support;
use serde_json::{Value, json};
use std::fs;

const RECORD: &str = "tests/_fixtures/tooling/authored-component-infer-13113.json";
const RECORD_SHA: &str = "0698d1a514fab517c63ec516202a2dffbe33467107b8b0062437a8f3f64a795f";

fn record() -> Value {
    let bytes = fs::read(support::repository().join(RECORD)).unwrap();
    assert_eq!(support::hash(&bytes), RECORD_SHA);
    let record: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(record["schema"], "vize.authoredComponentInferSource.v1");
    assert_eq!(record["issue"], 13113);
    assert_eq!(
        record["credit"],
        json!({ "compiled": false, "native": false, "performance": false })
    );
    record
}

fn read_source_record(bytes: &[u8], hash: &str, schema: &str, issue: u64) -> Value {
    assert_eq!(support::hash(bytes), hash);
    let record: Value = serde_json::from_slice(bytes).unwrap();
    assert_eq!(record["schema"], schema);
    assert_eq!(record["issue"], issue);
    assert_eq!(
        record["credit"],
        json!({ "compiled": false, "native": false, "performance": false })
    );
    record
}

fn script_consumed_slots_record() -> &'static Value {
    static RECORD: std::sync::OnceLock<Value> = std::sync::OnceLock::new();
    RECORD.get_or_init(|| {
        read_source_record(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../tests/_fixtures/tooling/script-consumed-slots-13578.json"
            )),
            "8b6794567259acb6b220fb84ea4b622c16b5674f837c751adbeb4010d6ffb848",
            "vize.scriptConsumedSlotsSource.v1",
            13578,
        )
    })
}

pub(super) fn before_metadata(path: &str, current: String) -> String {
    let current = restore_producer(path, current, script_consumed_slots_record());
    static RECORDS: std::sync::OnceLock<(Value, Value)> = std::sync::OnceLock::new();
    let (metadata, default_expression) = RECORDS.get_or_init(|| {
        (
            read_source_record(
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../tests/_fixtures/tooling/component-metadata-13430.json"
                )),
                "24efda8a8e69229347612becd4f5503d1158d824a755f2f5b9d3b5eff5f23b95",
                "vize.componentMetadataSource.v1",
                13430,
            ),
            read_source_record(
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../tests/_fixtures/tooling/default-export-expression-mapping-12985.json"
                )),
                "c7747859a9100742ac100f965c4e981005381e65c3bf12ed1986a699e459e8d2",
                "vize.defaultExportExpressionMappingSource.v1",
                12985,
            ),
        )
    });
    restore_producer(
        path,
        restore_producer(path, current, default_expression),
        metadata,
    )
}

fn is_after(current: &str, transition: &Value) -> bool {
    json!(current.len()) == transition["after"]["bytes"]
        && support::hash(current.as_bytes()) == transition["after"]["sha256"]
}

fn restore(current: &str, transition: &Value) -> String {
    assert!(is_after(current, transition));
    let mut previous = current.to_owned();
    for hunk in transition["hunks"].as_array().unwrap() {
        let after = hunk["after"].as_str().unwrap();
        assert_eq!(previous.matches(after).count(), 1);
        previous = previous.replacen(after, hunk["before"].as_str().unwrap(), 1);
    }
    assert_eq!(json!(previous.len()), transition["before"]["bytes"]);
    assert_eq!(
        support::hash(previous.as_bytes()),
        transition["before"]["sha256"]
    );
    previous
}

/// The pre-#13113 bytes of a recorded producer's current image. Older images
/// pass through; the layers below authenticate them.
pub(super) fn source_before(path: &str, current: String) -> String {
    let current =
        super::with_defaults_undefined::source_before(path, before_metadata(path, current));
    restore_producer(path, current, &record())
}

fn restore_producer(path: &str, current: String, record: &Value) -> String {
    match record["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|file| file["path"] == path)
    {
        Some(file) if is_after(&current, file) => restore(&current, file),
        _ => current,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_consumed_slots_restore_only_the_exact_recorded_producers() {
        let record = script_consumed_slots_record();
        for file in record["files"].as_array().unwrap() {
            let path = file["path"].as_str().unwrap();
            let current = fs::read_to_string(support::repository().join(path)).unwrap();
            assert!(is_after(&current, file), "{path}");
            let previous = restore_producer(path, current.clone(), record);
            assert_eq!(support::hash(previous.as_bytes()), file["before"]["sha256"]);
            assert_eq!(restore_producer(path, previous.clone(), record), previous);
            assert_eq!(
                restore_producer("other.rs", current.clone(), record),
                current
            );
            for mutated in [format!("{current}{current}"), format!("{current}\n")] {
                assert_eq!(restore_producer(path, mutated.clone(), record), mutated);
                assert!(std::panic::catch_unwind(|| restore(&mutated, file)).is_err());
            }
            for hunk in file["hunks"].as_array().unwrap() {
                let after = hunk["after"].as_str().unwrap();
                let mutated = current.replacen(after, "", 1);
                assert_eq!(restore_producer(path, mutated.clone(), record), mutated);
                assert!(std::panic::catch_unwind(|| restore(&mutated, file)).is_err());
            }
        }
    }

    #[test]
    fn recorded_producer_restores_its_exact_predecessor() {
        let record = record();
        for file in record["files"].as_array().unwrap() {
            let path = file["path"].as_str().unwrap();
            let current = fs::read_to_string(support::repository().join(path)).unwrap();
            let current = before_metadata(path, current);
            assert!(is_after(&current, file), "{path}");
            let previous = source_before(path, current.clone());
            assert_eq!(support::hash(previous.as_bytes()), file["before"]["sha256"]);
            assert_eq!(source_before(path, previous.clone()), previous);
            let first = file["hunks"][0]["after"].as_str().unwrap();
            for mutated in [
                format!("{current}{current}"),
                current.replacen(first, "", 1),
            ] {
                assert_eq!(source_before(path, mutated.clone()), mutated);
                assert!(std::panic::catch_unwind(|| restore(&mutated, file)).is_err());
            }
        }
    }
}
