use std::collections::{BTreeMap, BTreeSet};

use vize_s0::String;

use crate::ApplicationContract;

use super::{
    AdapterCapabilityManifest, AdapterCapabilityMismatch, AdapterCapabilityMismatchCode,
    AdapterCapabilityNegotiation, AdapterCapabilitySupport, validate_adapter_capability_manifest,
};

/// Negotiates application capability requirements against one adapter.
///
/// Requirement identifiers are deduplicated and sorted. Unknown application
/// capability identifiers fail closed instead of being treated as adapter
/// omissions. An invalid manifest never produces a compatible result.
pub fn negotiate_adapter_capabilities<'a>(
    contract: &ApplicationContract,
    required_capabilities: impl IntoIterator<Item = &'a str>,
    manifest: &AdapterCapabilityManifest,
) -> AdapterCapabilityNegotiation {
    let diagnostics = validate_adapter_capability_manifest(manifest);
    let support = manifest
        .capabilities
        .iter()
        .map(|capability| (capability.id.as_str(), capability))
        .collect::<BTreeMap<_, _>>();
    let requirements = required_capabilities.into_iter().collect::<BTreeSet<_>>();
    let mut mismatches = requirements
        .into_iter()
        .filter_map(|id| {
            mismatch(
                contract,
                id,
                support.get(id).copied(),
                diagnostics.is_empty(),
            )
        })
        .collect::<Vec<_>>();

    mismatches
        .sort_by(|left, right| (&left.capability, left.code).cmp(&(&right.capability, right.code)));
    AdapterCapabilityNegotiation {
        adapter: manifest.adapter.clone(),
        compatible: diagnostics.is_empty() && mismatches.is_empty(),
        diagnostics,
        mismatches,
    }
}

fn mismatch(
    contract: &ApplicationContract,
    id: &str,
    support: Option<&AdapterCapabilitySupport>,
    manifest_is_valid: bool,
) -> Option<AdapterCapabilityMismatch> {
    let Some(requirement) = contract.capabilities.get(id) else {
        return Some(problem(
            AdapterCapabilityMismatchCode::UnknownRequirement,
            id,
            None,
            None,
        ));
    };
    if !manifest_is_valid {
        return None;
    }
    let Some(support) = support else {
        return Some(problem(
            AdapterCapabilityMismatchCode::MissingCapability,
            id,
            Some(requirement.version),
            None,
        ));
    };
    let code = if requirement.version < support.min_version {
        Some(AdapterCapabilityMismatchCode::VersionBelowMinimum)
    } else if requirement.version > support.max_version {
        Some(AdapterCapabilityMismatchCode::VersionAboveMaximum)
    } else {
        None
    };
    code.map(|code| problem(code, id, Some(requirement.version), Some(support)))
}

fn problem(
    code: AdapterCapabilityMismatchCode,
    capability: &str,
    required_version: Option<u32>,
    support: Option<&AdapterCapabilitySupport>,
) -> AdapterCapabilityMismatch {
    AdapterCapabilityMismatch {
        code,
        capability: String::from(capability),
        path: capability_path(capability),
        message: mismatch_message(code).into(),
        required_version,
        min_version: support.map(|value| value.min_version),
        max_version: support.map(|value| value.max_version),
    }
}

fn capability_path(capability: &str) -> String {
    let mut path = String::from("capabilities.");
    path.push_str(capability);
    path
}

const fn mismatch_message(code: AdapterCapabilityMismatchCode) -> &'static str {
    match code {
        AdapterCapabilityMismatchCode::UnknownRequirement => {
            "application references an undeclared capability requirement"
        }
        AdapterCapabilityMismatchCode::MissingCapability => {
            "adapter does not support the required capability"
        }
        AdapterCapabilityMismatchCode::VersionBelowMinimum => {
            "application requires a capability version below the adapter minimum"
        }
        AdapterCapabilityMismatchCode::VersionAboveMaximum => {
            "application requires a capability version above the adapter maximum"
        }
    }
}
