//! Exact new loop-source reads are additive to the original 64 witnesses.
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

const REGISTRY: &str = include_str!("fixtures/public-instance-loop-sources/additions.json");
const ORIGINAL: &str =
    include_str!("fixtures/public-instance-loop-sources/original-comparisons.json");

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Registry {
    version: u8,
    entries: Vec<Entry>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Entry {
    path: String,
    template_base: usize,
    authored_span: [usize; 2],
    authored_text: String,
    diagnostic: Value,
    source: String,
    source_bytes: usize,
    source_sha256: String,
}

impl Entry {
    fn observation(&self) -> Value {
        serde_json::json!({
            "path": self.path,
            "templateBase": self.template_base,
            "authoredSpan": self.authored_span,
            "authoredText": self.authored_text,
            "diagnostic": self.diagnostic,
        })
    }
}

pub struct Additions {
    entries: BTreeMap<PathBuf, Entry>,
    observed: BTreeSet<PathBuf>,
}

impl Additions {
    pub fn new(root: &Path) -> Self {
        let registry: Registry =
            serde_json::from_str(REGISTRY).expect("fixed loop-source registry");
        assert_eq!(registry.version, 1);
        assert_eq!(registry.entries.len(), 2);
        let mut entries = BTreeMap::new();
        for entry in registry.entries {
            assert_eq!(entry.source.len(), entry.source_bytes);
            assert_eq!(sha256(&entry.source), entry.source_sha256);
            assert!(entries.insert(root.join(&entry.path), entry).is_none());
        }
        Self {
            entries,
            observed: BTreeSet::new(),
        }
    }

    pub fn accept(&mut self, path: &Path, source: &str, observation: &Value) -> bool {
        let Some(entry) = self.entries.get(path) else {
            return false;
        };
        // Other diagnostics in the same source remain original witnesses.
        if observation != &entry.observation() {
            return false;
        }
        assert_eq!(source.len(), entry.source_bytes, "loop-source input length");
        assert_eq!(
            sha256(source),
            entry.source_sha256,
            "loop-source input body"
        );
        assert!(
            self.observed.insert(path.to_path_buf()),
            "duplicate loop-source witness"
        );
        eprintln!("projection loop-source addition={observation}");
        true
    }

    pub fn finish(&self) {
        assert_eq!(
            self.observed,
            self.entries.keys().cloned().collect(),
            "both exact loop-source comparisons must remain additive"
        );
    }
}

pub fn assert_original_comparisons(mut observations: Vec<Value>) {
    let mut expected: Vec<Value> = serde_json::from_str(ORIGINAL).expect("original 64 comparisons");
    assert_eq!(expected.len(), 64);
    expected.sort_by_cached_key(|value| value.to_string());
    observations.sort_by_cached_key(|value| value.to_string());
    assert_eq!(
        observations, expected,
        "original comparison identities changed"
    );
}

fn sha256(source: &str) -> String {
    Sha256::digest(source.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod controls {
    use super::*;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    #[test]
    fn frozen_baseline_observer_remains_authenticated() {
        assert_eq!(
            sha256(include_str!(
                "fixtures/public-instance-loop-sources/baseline-observer.rs.txt"
            )),
            "a973e915ed4a453491ca0ce4c3d1bcacbef0e45f3a728854f773abaec300788b"
        );
    }

    #[test]
    fn exact_sources_pass_but_missing_duplicate_changed_body_and_changed_span_refuse() {
        let root = Path::new("private-fixtures");
        let registry: Registry = serde_json::from_str(REGISTRY).expect("fixed registry");
        let mut additions = Additions::new(root);
        for entry in &registry.entries {
            assert!(additions.accept(&root.join(&entry.path), &entry.source, &entry.observation()));
        }
        additions.finish();
        assert!(catch_unwind(|| Additions::new(root).finish()).is_err());
        let entry = registry.entries.first().expect("first complete input");
        let path = root.join(&entry.path);
        let observation = entry.observation();
        assert!(
            catch_unwind(AssertUnwindSafe(|| additions.accept(
                &path,
                &entry.source,
                &observation
            )))
            .is_err()
        );
        let mut changed = entry.source.clone().into_bytes();
        *changed.first_mut().expect("nonempty input") ^= 1;
        let changed = String::from_utf8(changed).expect("same-size source mutation");
        assert!(
            catch_unwind(AssertUnwindSafe(|| Additions::new(root).accept(
                &path,
                &changed,
                &observation
            )))
            .is_err()
        );
        let mut changed = observation.clone();
        *changed.get_mut("authoredSpan").expect("authored span") = serde_json::json!([0, 1]);
        assert!(!Additions::new(root).accept(&path, &entry.source, &changed));
        assert!(!Additions::new(root).accept(&root.join("Other.vue"), &entry.source, &observation));
    }

    #[test]
    fn retained_before_native_run_matches_every_original_comparison() {
        let receipt: Value = serde_json::from_str(include_str!(
            "fixtures/public-instance-loop-sources/baseline-proof/receipt.json"
        ))
        .expect("before producer receipt");
        assert_eq!(
            receipt["before"],
            "481c94bd55a7276339adc277edd93e30313f6ea9"
        );
        assert_eq!(receipt["run"], "40494413713");
        assert_eq!(receipt["artifact"], 12730062824_u64);
        assert_eq!(receipt["comparisons"], 64);
        let stdout =
            include_str!("fixtures/public-instance-loop-sources/baseline-proof/stdout.txt");
        let stderr =
            include_str!("fixtures/public-instance-loop-sources/baseline-proof/stderr.txt");
        let patch = include_str!(
            "fixtures/public-instance-loop-sources/baseline-proof/observer-only.patch.txt"
        );
        assert_eq!(sha256(stdout), receipt["beforeStdoutSha256"]);
        assert_eq!(sha256(stderr), receipt["beforeStderrSha256"]);
        assert_eq!(sha256(patch), receipt["observerPatchSha256"]);
        assert_eq!(
            stdout,
            concat!(
                "\nrunning 1 test\n",
                "test check_diagnostics_land_inside_the_projection ... ok\n\n",
                "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; ",
                "5 filtered out; finished in 0.41s\n\n"
            )
        );
        let observations = stderr
            .lines()
            .filter_map(|line| {
                line.strip_prefix("projection baseline comparison=")
                    .map(|json| {
                        serde_json::from_str(json).expect("complete captured before observation")
                    })
            })
            .collect();
        assert_original_comparisons(observations);
    }

    #[test]
    fn original_same_count_diagnostic_mutation_refuses() {
        let original: Vec<Value> = serde_json::from_str(ORIGINAL).expect("original comparisons");
        assert_original_comparisons(original.clone());
        let mut changed = original;
        *changed.first_mut().expect("nonempty original census") =
            serde_json::json!({"changed": true});
        assert!(catch_unwind(|| assert_original_comparisons(changed)).is_err());
    }
}
