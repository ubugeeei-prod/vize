//! Reverse the CLI-owned development dependencies before reading older SDK history.
use super::support;
use serde_json::{Value, json};

const MANIFEST: &str = "npm/cli/package.json";
const AFTER_BYTES: usize = 4_362;
const AFTER_SHA: &str = "11abb0e379b919dcff38cc4aed99e1a8934aab4a55d9000115cf54dc1608f5c9";
const BEFORE_BYTES: usize = 4_273;
const BEFORE_SHA: &str = "2e430bc9bbb14cbd07ae076459bf0a932d1558bb9a698b67527b4086d8988e6f";
const ADDITIONS: &[&str] = &[
    "    \"json-schema-to-typescript\": \"catalog:repo-tooling\",\n",
    "    \"oxfmt\": \"catalog:linting\",\n",
];

fn authenticate(text: &str, bytes: usize, sha: &str) {
    assert_eq!(text.len(), bytes);
    assert_eq!(support::hash(text.as_bytes()), sha);
}

pub(super) fn source_before(path: &str, current: String) -> String {
    if path != MANIFEST {
        return current;
    }
    authenticate(&current, AFTER_BYTES, AFTER_SHA);
    let mut previous = current;
    for addition in ADDITIONS {
        assert_eq!(previous.matches(addition).count(), 1);
        previous = previous.replacen(addition, "", 1);
    }
    authenticate(&previous, BEFORE_BYTES, BEFORE_SHA);
    previous
}

pub(super) fn retain_provenance(manifest: &mut Value) {
    manifest["provenance"]["currentCliDependencyOwnership"] = json!({
        "kind": "exact-cli-development-dependency-source-inverse",
        "pullRequest": 12123,
        "nativeAcceptanceCredit": false,
        "path": MANIFEST,
        "beforeBytes": BEFORE_BYTES,
        "beforeSha256": BEFORE_SHA,
        "afterBytes": AFTER_BYTES,
        "afterSha256": AFTER_SHA,
        "additions": ADDITIONS,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn current_cli_manifest_reconstructs_the_exact_dependency_predecessor() {
        let current = fs::read_to_string(support::repository().join(MANIFEST)).unwrap();
        let previous = source_before(MANIFEST, current.clone());
        authenticate(&previous, BEFORE_BYTES, BEFORE_SHA);
        for changed in [format!("{current}\n"), format!("{current}{current}")] {
            assert!(std::panic::catch_unwind(|| source_before(MANIFEST, changed)).is_err());
        }
        for addition in ADDITIONS {
            let changed = current.replacen(addition, "", 1);
            assert!(std::panic::catch_unwind(|| source_before(MANIFEST, changed)).is_err());
        }
    }
}
