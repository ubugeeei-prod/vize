//! One executable documentation gate; all historical manifest bytes remain bound.
use crate::support;

const MANIFEST: &str = "docs/package.json";
const CURRENT_BYTES: usize = 1_195;
const CURRENT_SHA: &str = "79b7263bdaa9216ea566a819ec22c9dc6b0ff62fa8b8b08f257d10c7ce6c36fb";
const PREVIOUS_BYTES: usize = 1_128;
const PREVIOUS_SHA: &str = "9de99893b0877b0b8e7480c9847483c59d8b01277d35d2d9774b4d6b7fa4b077";
const ADDED: &str = "    \"check:contracts\": \"node ./scripts/check-source-contracts.ts\",\n";

pub(crate) fn before_docs_source_contracts(path: &str, body: &[u8]) -> Option<Vec<u8>> {
    if path != MANIFEST || body.len() != CURRENT_BYTES || support::hash(body) != CURRENT_SHA {
        return None;
    }
    let current = std::str::from_utf8(body).ok()?;
    if current.matches(ADDED).count() != 1 {
        return None;
    }
    let previous = current.replacen(ADDED, "", 1).into_bytes();
    (previous.len() == PREVIOUS_BYTES && support::hash(&previous) == PREVIOUS_SHA)
        .then_some(previous)
}

#[cfg(test)]
mod tests {
    use super::{ADDED, MANIFEST, before_docs_source_contracts, support};
    use std::fs;

    #[test]
    fn exact_docs_gate_inverse_preserves_every_historical_manifest_byte() {
        let current = fs::read_to_string(support::repository().join(MANIFEST)).unwrap();
        let previous = before_docs_source_contracts(MANIFEST, current.as_bytes()).unwrap();
        assert_eq!(current.len() - previous.len(), ADDED.len());
        let previous = String::from_utf8(previous).unwrap();
        let manifest: serde_json::Value = serde_json::from_str(&previous).unwrap();
        assert_eq!(
            manifest["scripts"]["check:contracts"],
            serde_json::Value::Null
        );
        let anchor =
            "    \"generate:metadata\": \"node ./scripts/generate-contract-metadata.ts\",\n";
        assert_eq!(previous.matches(anchor).count(), 1);
        assert_eq!(
            previous.replacen(anchor, &format!("{anchor}{ADDED}"), 1),
            current
        );
        assert!(before_docs_source_contracts(MANIFEST, previous.as_bytes()).is_none());
        assert_eq!(
            super::super::restore(MANIFEST, current.clone()),
            super::super::restore(MANIFEST, previous.clone()),
        );
        assert_eq!(
            super::super::before_translation_worker(MANIFEST, current.as_bytes()),
            super::super::before_translation_worker(MANIFEST, previous.as_bytes()),
        );
        assert!(super::super::before_translation_worker(MANIFEST, current.as_bytes()).is_some());
    }

    #[test]
    fn missing_duplicate_foreign_and_unrelated_docs_bytes_cannot_use_the_successor() {
        let current = fs::read_to_string(support::repository().join(MANIFEST)).unwrap();
        assert!(before_docs_source_contracts("foreign-package.json", current.as_bytes()).is_none());
        for changed in [
            current.replacen(ADDED, "", 1),
            current.replacen(ADDED, &format!("{ADDED}{ADDED}"), 1),
            current.replacen("check-source-contracts.ts", "unchecked-contracts.ts", 1),
            current.replacen("generate-contract-metadata.ts", "foreign-metadata.ts", 1),
            current.replacen("\"1.2.6\"", "\"0.0.0\"", 1),
            current.replacen("vize-docs", "foreign-docs", 1),
            format!("{current}\n"),
        ] {
            assert_ne!(changed, current);
            assert!(before_docs_source_contracts(MANIFEST, changed.as_bytes()).is_none());
        }
    }
}
