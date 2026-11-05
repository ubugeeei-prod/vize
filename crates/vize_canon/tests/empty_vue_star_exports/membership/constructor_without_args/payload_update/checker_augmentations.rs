//! Invert only the #11630 move of checker-only augmentations before #11591.
use super::support;
use serde_json::{Value, json};

const PATH: &str = "npm/cli/virtual-ts-globals.d.ts";
const BEFORE: &str = "\n  // ImportMeta augmentation (reference existing framework types)\n  // Extend ImportMeta with Nuxt-specific properties not covered by vite/client\n  interface ImportMeta {\n    client: boolean;\n    server: boolean;\n    dev: boolean;\n    prod: boolean;\n    ssr: boolean;\n  }\n\n  namespace JSX {\n    interface IntrinsicAttributes {\n      class?: unknown;\n      style?: unknown;\n    }\n  }\n\n  // Shared type helpers used by generated virtual modules\n";
const AFTER: &str = "\n  // Shared type helpers used by generated virtual modules\n";
const BEFORE_BYTES: usize = 19261;
const BEFORE_SHA: &str = "1077ed40b76672245658ad66ded95531ccfbe41405b713b44126c1704f6c1cd2";
const AFTER_SHA: &str = "b75a5ae2d5e020e819088ebc68eaf0a8cd9a5b57de900b700cd3759b936a9fb0";

pub(super) fn source_before(path: &str, current: String) -> String {
    if path != PATH {
        return current;
    }
    assert_eq!(current.len() + BEFORE.len(), BEFORE_BYTES + AFTER.len());
    assert_eq!(support::hash(current.as_bytes()), AFTER_SHA);
    assert_eq!(current.matches(AFTER).count(), 1);
    assert_eq!(current.matches(BEFORE).count(), 0);
    let previous = current.replacen(AFTER, BEFORE, 1);
    assert_eq!(previous.len(), BEFORE_BYTES);
    assert_eq!(support::hash(previous.as_bytes()), BEFORE_SHA);
    previous
}

pub(super) fn retain_provenance(manifest: &mut Value) {
    manifest["provenance"]["currentCheckerAugmentationsUpdate"] = json!({
        "kind": "exact-current-checker-augmentations-inverse",
        "issue": 11630,
        "nativeAcceptanceCredit": false,
        "files": [{ "path": PATH, "beforeBytes": BEFORE_BYTES,
            "afterBytes": BEFORE_BYTES + AFTER.len() - BEFORE.len(),
            "beforeSha256": BEFORE_SHA, "afterSha256": AFTER_SHA }],
        "compiledPreamble": "unchanged"
    });
}

#[cfg(test)]
mod tests {
    use super::{AFTER, BEFORE, PATH, retain_provenance, source_before};
    use serde_json::json;

    fn current(path: &str) -> String {
        std::fs::read_to_string(super::support::repository().join(path)).unwrap()
    }

    #[test]
    fn current_globals_reconstruct_only_the_checker_augmentations_layer() {
        let body = current(PATH);
        assert_eq!(
            source_before(PATH, body.clone()).replacen(BEFORE, AFTER, 1),
            body
        );
        let other = "crates/vize_canon/src/virtual_ts/helpers/declarations/core-types.d.ts.txt";
        assert_eq!(source_before(other, current(other)), current(other));
        let mut manifest = json!({ "provenance": { "existing": "unchanged" } });
        retain_provenance(&mut manifest);
        assert_eq!(manifest["provenance"]["existing"], "unchanged");
    }

    #[test]
    fn missing_duplicate_stale_and_extra_bytes_refuse() {
        let body = current(PATH);
        for mutated in [
            body.replacen(AFTER, "", 1),
            format!("{body}{AFTER}"),
            body.replacen(AFTER, BEFORE, 1),
            format!("{body}\n"),
        ] {
            assert!(std::panic::catch_unwind(|| source_before(PATH, mutated.clone())).is_err());
        }
    }
}
