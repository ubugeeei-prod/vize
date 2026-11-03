//! Consume the complete machine-readable report; terminal styling is not a test protocol.

use serde_json::Value;
use std::path::Path;

pub(super) fn diagnostic_sites(bytes: &[u8]) -> Vec<(u32, u32, u32)> {
    let report: Value = serde_json::from_slice(bytes).unwrap_or_else(|error| {
        panic!(
            "invalid check JSON: {error}\n{}",
            std::str::from_utf8(bytes).unwrap_or("<non-UTF-8 output>")
        )
    });
    let files = report["files"].as_array().expect("a complete file report");
    assert_eq!(
        files.len(),
        1,
        "the authored probe must be checked: {report}"
    );
    assert_eq!(report["fileCount"], 1, "{report}");
    assert_eq!(report["warningCount"], 0, "{report}");
    assert_eq!(
        Path::new(files[0]["file"].as_str().expect("a diagnostic file path")),
        Path::new("src/Probe.vue"),
        "diagnostics must belong to the authored probe: {report}"
    );
    let diagnostics = files[0]["diagnostics"]
        .as_array()
        .expect("the complete diagnostic list");
    assert_eq!(
        report["errorCount"].as_u64().expect("an error count"),
        diagnostics.len() as u64,
        "the summary must account for every diagnostic: {report}"
    );
    diagnostics
        .iter()
        .map(|value| {
            let text = value.as_str().expect("a diagnostic string");
            let (site, rest) = text
                .strip_prefix("error:")
                .and_then(|text| text.split_once(" [TS"))
                .expect("an unstyled TypeScript error diagnostic");
            let (line, column) = site.split_once(':').expect("an authored line and column");
            let (code, message) = rest
                .split_once("] ")
                .expect("a diagnostic code and message");
            assert!(!message.is_empty(), "a diagnostic must retain its message");
            (
                line.parse().expect("a numeric authored line"),
                column.parse().expect("a numeric authored column"),
                code.parse().expect("a numeric TypeScript diagnostic code"),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn packet() -> Value {
        json!({
            "files": [{
                "file": "src/Probe.vue",
                "diagnostics": ["error:3:7 [TS2322] Type 'number' is not assignable to type 'string'."]
            }],
            "errorCount": 1,
            "warningCount": 0,
            "fileCount": 1
        })
    }

    #[test]
    fn complete_clean_and_error_packets_preserve_the_authored_sites() {
        let mut value = packet();
        assert_eq!(
            diagnostic_sites(&serde_json::to_vec(&value).unwrap()),
            [(3, 7, 2322)]
        );
        value["files"][0]["diagnostics"] = json!([]);
        value["errorCount"] = json!(0);
        assert!(diagnostic_sites(&serde_json::to_vec(&value).unwrap()).is_empty());
    }

    #[test]
    fn incomplete_foreign_or_unparsed_diagnostics_cannot_become_clean() {
        for (field, changed) in [
            ("fileCount", json!(0)),
            ("errorCount", json!(0)),
            ("warningCount", json!(1)),
            ("files", json!([])),
        ] {
            let mut value = packet();
            value[field] = changed;
            assert!(
                std::panic::catch_unwind(|| {
                    diagnostic_sites(&serde_json::to_vec(&value).unwrap())
                })
                .is_err()
            );
        }
        for changed in [json!("src/Other.vue"), Value::Null] {
            let mut value = packet();
            value["files"][0]["file"] = changed;
            assert!(
                std::panic::catch_unwind(|| {
                    diagnostic_sites(&serde_json::to_vec(&value).unwrap())
                })
                .is_err()
            );
        }
        for changed in [
            json!("noise"),
            json!("warning:3:7 [TS2322] wrong severity"),
            json!("\u{1b}[31merror:3:7 [TS2322] styled text\u{1b}[0m"),
            json!("error:three:7 [TS2322] malformed location"),
            Value::Null,
        ] {
            let mut value = packet();
            value["files"][0]["diagnostics"][0] = changed;
            assert!(
                std::panic::catch_unwind(|| {
                    diagnostic_sites(&serde_json::to_vec(&value).unwrap())
                })
                .is_err()
            );
        }
        assert!(std::panic::catch_unwind(|| diagnostic_sites(b"not JSON")).is_err());
    }
}
