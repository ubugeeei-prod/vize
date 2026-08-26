use sha2::{Digest, Sha256};
use vize_s0::String;

use super::model::TestRunEvidence;

/// Failure to serialize a test-run evidence record canonically.
#[derive(Debug)]
pub struct CanonicalTestRunError(serde_json::Error);

impl std::fmt::Display for CanonicalTestRunError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("failed to serialize the test-run evidence")
    }
}

impl std::error::Error for CanonicalTestRunError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

/// Serializes a test-run evidence record after sorting recorded executions.
///
/// Targets sort by id and suites by id then shard index, so equivalent records
/// produce identical bytes for cache keys, fixtures, and fingerprints. Call
/// validation before trusting the record; canonicalization does not make an
/// invalid record valid.
pub fn canonical_test_run_json(
    evidence: &TestRunEvidence,
) -> Result<Vec<u8>, CanonicalTestRunError> {
    let mut canonical = evidence.clone();
    canonical
        .targets
        .sort_by(|left, right| left.id.cmp(&right.id));
    canonical
        .suites
        .sort_by(|left, right| (&left.id, left.shard_index).cmp(&(&right.id, right.shard_index)));
    serde_json::to_vec(&canonical).map_err(CanonicalTestRunError)
}

/// Returns a lowercase SHA-256 fingerprint of the canonical record.
///
/// The fingerprint is the exact value admitted as `test-run:<sha256>` by
/// deployment gates.
pub fn test_run_fingerprint(evidence: &TestRunEvidence) -> Result<String, CanonicalTestRunError> {
    let bytes = canonical_test_run_json(evidence)?;
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in digest {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use crate::test_run::model_tests::{example_evidence, filled};

    use super::*;

    #[test]
    fn execution_order_does_not_change_the_fingerprint() {
        let left = example_evidence();
        let mut right = left.clone();
        right.targets.reverse();
        right.suites.reverse();

        assert_eq!(
            canonical_test_run_json(&left).unwrap(),
            canonical_test_run_json(&right).unwrap()
        );
        assert_eq!(
            test_run_fingerprint(&left).unwrap(),
            test_run_fingerprint(&right).unwrap()
        );
    }

    #[test]
    fn any_bound_fact_changes_the_fingerprint() {
        let base = example_evidence();
        let baseline = test_run_fingerprint(&base).unwrap();

        let mut artifact = base.clone();
        artifact.artifact.fingerprint = filled('9', 64);
        let mut revision = base.clone();
        revision.source_revision = filled('b', 40);
        let mut counts = base;
        counts.suites[0].passed += 1;

        for changed in [&artifact, &revision, &counts] {
            assert_ne!(baseline, test_run_fingerprint(changed).unwrap());
        }
    }
}
