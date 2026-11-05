//! Authenticate the TSX-only exact-key change before the older input-channel layer.
use super::support;
use serde_json::{Value, json};

const RECORD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/tsx-single-required-exact-keys/source-transition.json"
));

fn load() -> Value {
    assert_eq!(
        support::hash(RECORD.as_bytes()),
        "c647fda7816e1684a864d015efde1fe8a4ada2f447e38f9930fce5e6833c6bcd"
    );
    let record: Value = serde_json::from_str(RECORD).unwrap();
    assert_eq!(record["schema"], "vize.tsx-exact-keys-source-transition.v1");
    assert_eq!(
        record["sourceBefore"],
        "727db3b16a8eb5151d326c73725d275498f9d54d"
    );
    assert_eq!(record["nativeAcceptanceCredit"], false);
    assert_eq!(record["files"].as_array().unwrap().len(), 3);
    record
}

fn invert(current: String, record: &Value) -> String {
    assert_eq!(json!(current.len()), record["afterBytes"]);
    assert_eq!(
        json!(support::hash(current.as_bytes())),
        record["afterSha256"]
    );
    let mut previous = current;
    for edit in record["edits"].as_array().unwrap() {
        let before = edit["before"].as_str().unwrap();
        let after = edit["after"].as_str().unwrap();
        assert_ne!(before, after);
        assert_eq!(previous.matches(after).count(), 1);
        assert_eq!(previous.matches(before).count(), 0);
        previous = previous.replacen(after, before, 1);
    }
    assert_eq!(json!(previous.len()), record["beforeBytes"]);
    assert_eq!(
        json!(support::hash(previous.as_bytes())),
        record["beforeSha256"]
    );
    previous
}

pub(super) fn source_before(path: &str, current: String) -> String {
    let record = load();
    match record["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|file| file["path"] == path)
    {
        Some(file) => invert(current, file),
        None => current,
    }
}

pub(super) fn preamble_before(current: &str) -> String {
    invert(current.to_owned(), &load()["preamble"])
}

pub(super) fn retain_provenance(manifest: &mut Value) {
    manifest["provenance"]["currentTsxExactKeysUpdate"] = load();
}

#[cfg(test)]
mod tests {
    use super::{invert, load, preamble_before, retain_provenance, source_before};
    use serde_json::{Value, json};

    fn forward(mut previous: String, record: &Value) -> String {
        for edit in record["edits"].as_array().unwrap().iter().rev() {
            previous = previous.replacen(
                edit["before"].as_str().unwrap(),
                edit["after"].as_str().unwrap(),
                1,
            );
        }
        previous
    }

    #[test]
    fn all_current_assets_and_the_compiled_preamble_have_exact_inverses() {
        let record = load();
        for file in record["files"].as_array().unwrap() {
            let path = file["path"].as_str().unwrap();
            let body = std::fs::read_to_string(super::support::repository().join(path)).unwrap();
            assert_eq!(forward(source_before(path, body.clone()), file), body);
        }
        let preamble = vize_canon::virtual_ts::SHARED_PREAMBLE_DTS;
        assert_eq!(
            forward(preamble_before(preamble), &record["preamble"]),
            preamble
        );
        assert_eq!(
            source_before("unrelated/source.ts", "unchanged".to_owned()),
            "unchanged"
        );
        let mut manifest = json!({ "provenance": { "existing": "unchanged" } });
        retain_provenance(&mut manifest);
        assert_eq!(manifest["provenance"]["existing"], "unchanged");
        assert_eq!(manifest["provenance"]["currentTsxExactKeysUpdate"], record);
    }

    #[test]
    fn missing_duplicate_stale_foreign_and_extra_bytes_refuse_at_this_layer() {
        let record = load();
        let bodies = record["files"]
            .as_array()
            .unwrap()
            .iter()
            .map(|file| {
                (
                    std::fs::read_to_string(
                        super::support::repository().join(file["path"].as_str().unwrap()),
                    )
                    .unwrap(),
                    file,
                )
            })
            .chain(std::iter::once((
                vize_canon::virtual_ts::SHARED_PREAMBLE_DTS.to_owned(),
                &record["preamble"],
            )));
        for (body, file) in bodies {
            let before = file["edits"][0]["before"].as_str().unwrap();
            let after = file["edits"][0]["after"].as_str().unwrap();
            for mutated in [
                body.replacen(after, "", 1),
                format!("{body}{after}"),
                body.replacen(after, before, 1),
                body.replacen("type ", "xxxx ", 1),
                format!("{body}\n"),
            ] {
                assert!(std::panic::catch_unwind(|| invert(mutated.clone(), file)).is_err());
            }
        }
    }
}
