//! Invert the focused entry package CLI exports before reading frozen history.

use super::support;
use crate::package_task_history;
use serde_json::{Value, json};

const MANIFEST: &str = "npm/cli/package.json";
const BEFORE_BYTES: usize = 4_522;
const BEFORE_SHA: &str = "54f3c69dcae1a2a2263bd599fa3057ddf2eea6adab125c66ecbb59476130e87b";
const AFTER_BYTES: usize = 4_747;
const AFTER_SHA: &str = "835515952fb317caaa9403f566efee0bc66d44dc6150170595237de2192919a8";
const ADDED: &str = concat!(
    "    \"./cli\": {\n",
    "      \"import\": \"./dist/cli.mjs\",\n",
    "      \"default\": \"./dist/cli.mjs\"\n",
    "    },\n",
    "    \"./content-mapper-cli\": {\n",
    "      \"import\": \"./dist/content-mapper-cli.mjs\",\n",
    "      \"default\": \"./dist/content-mapper-cli.mjs\"\n",
    "    },\n",
);

pub(super) fn source_before(path: &str, current: String) -> String {
    let current = super::optional_jsx_witness::source_before(path, current);
    if path != MANIFEST {
        return current;
    }
    let current = if current.contains("node ./scripts/package-tasks.cjs ") {
        package_task_history::restore(MANIFEST, current)
    } else {
        current
    };
    let additions = current.matches(ADDED).count();
    if additions == 0 {
        return current;
    }
    assert_eq!(additions, 1);
    assert_eq!(current.len(), AFTER_BYTES);
    assert_eq!(support::hash(current.as_bytes()), AFTER_SHA);
    let previous = current.replacen(ADDED, "", 1);
    assert_eq!(previous.len(), BEFORE_BYTES);
    assert_eq!(support::hash(previous.as_bytes()), BEFORE_SHA);
    previous
}

pub(super) fn retain_provenance(manifest: &mut Value) {
    let current = package_task_history::read(MANIFEST).unwrap();
    let previous = source_before(MANIFEST, current.clone());
    manifest["provenance"]["currentEntryPackageCliExports"] = json!({
        "kind": "exact-entry-package-cli-export-source-inverse",
        "nativeAcceptanceCredit": false,
        "path": MANIFEST,
        "beforeBytes": previous.len(),
        "beforeSha256": BEFORE_SHA,
        "afterBytes": current.len(),
        "afterSha256": AFTER_SHA,
        "addition": ADDED,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_cli_manifest_reconstructs_the_exact_previous_source() {
        let current = package_task_history::read(MANIFEST).unwrap();
        let previous = source_before(MANIFEST, current.clone());
        assert_eq!(previous.matches(ADDED).count(), 0);
        const ANCHOR: &str = "    \"./virtual-ts\": {";
        assert_eq!(
            previous.replacen(ANCHOR, &format!("{ADDED}{ANCHOR}"), 1),
            current
        );
        assert_eq!(source_before(MANIFEST, previous.clone()), previous);
        for changed in [format!("{current}\n"), format!("{current}{current}")] {
            assert!(std::panic::catch_unwind(|| source_before(MANIFEST, changed)).is_err());
        }
    }
}
