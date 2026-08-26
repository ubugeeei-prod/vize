use sha2::{Digest, Sha256};
use vize_s0::String;

use crate::ApplicationContract;

/// Failure to serialize a validated contract canonically.
#[derive(Debug)]
pub struct CanonicalContractError(serde_json::Error);

impl std::fmt::Display for CanonicalContractError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("failed to serialize the application marquette")
    }
}

impl std::error::Error for CanonicalContractError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

/// Serializes a contract after sorting all identifier-addressed collections.
///
/// Canonical output is intended for cache keys, compatibility fixtures, and
/// generated artifact inputs. Call [`ApplicationContract::validate`] before
/// execution; canonicalization does not make an invalid graph valid.
pub fn canonical_json(contract: &ApplicationContract) -> Result<Vec<u8>, CanonicalContractError> {
    let mut canonical = contract.clone();
    canonical
        .environments
        .sort_by(|left, right| left.id.cmp(&right.id));
    canonical
        .backends
        .sort_by(|left, right| left.id.cmp(&right.id));
    canonical
        .protocols
        .sort_by(|left, right| left.id.cmp(&right.id));
    canonical
        .routes
        .sort_by(|left, right| left.id.cmp(&right.id));
    serde_json::to_vec(&canonical).map_err(CanonicalContractError)
}

/// Returns a lowercase SHA-256 fingerprint of the canonical contract.
pub fn contract_fingerprint(
    contract: &ApplicationContract,
) -> Result<String, CanonicalContractError> {
    let bytes = canonical_json(contract)?;
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
    use crate::{ApplicationContract, Environment, EnvironmentConsumer, RuntimeFamily, Target};

    use super::*;

    #[test]
    fn collection_order_does_not_change_the_fingerprint() {
        let mut left = ApplicationContract::new("example");
        left.targets.insert(Target::Web);
        left.environments.push(Environment::new(
            "server",
            Target::Web,
            EnvironmentConsumer::Server,
            RuntimeFamily::Rust,
        ));
        left.environments.push(Environment::new(
            "client",
            Target::Web,
            EnvironmentConsumer::Client,
            RuntimeFamily::Browser,
        ));
        let mut right = left.clone();
        right.environments.reverse();

        assert_eq!(
            contract_fingerprint(&left).unwrap(),
            contract_fingerprint(&right).unwrap()
        );
        assert_eq!(
            canonical_json(&left).unwrap(),
            canonical_json(&right).unwrap()
        );
    }
}
