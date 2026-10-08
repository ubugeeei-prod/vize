//! Opt-in custody of existing queries; recording never performs a native query.

use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::{
        OnceLock,
        atomic::{AtomicU64, Ordering},
    },
};
use vize_l0::cstr;

const LIMIT: usize = 16 * 1024 * 1024;
static ROOT: OnceLock<Option<PathBuf>> = OnceLock::new();
static NEXT: AtomicU64 = AtomicU64::new(0);

pub(super) struct Capture {
    output: Option<(PathBuf, Vec<Value>)>,
}

impl Capture {
    pub(super) fn new() -> Self {
        let output = ROOT
            .get_or_init(|| std::env::var_os("VIZE_COMPONENT_TYPE_CAPTURE").map(PathBuf::from))
            .as_ref()
            .map(|root| {
                let id = NEXT.fetch_add(1, Ordering::Relaxed);
                (
                    root.join(cstr!("native-{}-{id}.json", std::process::id()).as_str()),
                    Vec::new(),
                )
            });
        Self { output }
    }

    pub(super) fn record(&mut self, stage: &str, value: impl FnOnce() -> Value) {
        if let Some((_, records)) = self.output.as_mut() {
            records.push(json!({"stage":stage,"value":value()}));
        }
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        let Some((path, records)) = &self.output else {
            return;
        };
        // A missing/oversized artifact is a custody failure, never a partial
        // native response or a reason to change the classification outcome.
        let _ = (|| -> Result<(), Box<dyn std::error::Error>> {
            let bytes = serde_json::to_vec(&json!({
                "representation":"existing decoded queries and guard outcomes",
                "pid":std::process::id(),"records":records
            }))?;
            if bytes.len() > LIMIT {
                return Err("component type capture exceeds the unchanged 16 MiB cap".into());
            }
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, bytes)?;
            Ok(())
        })();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_capture_never_constructs_query_copies() {
        let mut capture = Capture { output: None };
        capture.record("disabled", || panic!("disabled capture copied a response"));
        assert!(capture.output.is_none());
    }

    #[test]
    fn drop_preserves_whole_records_without_changing_the_result() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("whole.json");
        {
            let mut capture = Capture {
                output: Some((path.clone(), Vec::new())),
            };
            capture.record(
                "source",
                || json!({"text":"😀\r\ncomplete","positions":[1,4]}),
            );
            capture.record("refusal", || json!({"reason":"ProjectIdentity"}));
        }
        let value: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(
            value,
            json!({
                "representation":"existing decoded queries and guard outcomes",
                "pid":std::process::id(),"records":[
                    {"stage":"source","value":{"text":"😀\r\ncomplete","positions":[1,4]}},
                    {"stage":"refusal","value":{"reason":"ProjectIdentity"}}
                ]
            })
        );
    }
}
