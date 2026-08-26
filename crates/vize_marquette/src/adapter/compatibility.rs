use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use vize_s0::{String, ToCompactString};

use crate::{
    CompatibilityChange, CompatibilityChangeKind,
    adapter::{
        AdapterCapabilityDiagnostic, AdapterCapabilityManifest,
        validate_adapter_capability_manifest,
    },
};

/// Deterministic compatibility report between two adapter manifests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdapterCapabilityCompatibilityReport {
    /// Validation failures in the previous manifest.
    pub previous_diagnostics: Vec<AdapterCapabilityDiagnostic>,
    /// Validation failures in the next manifest.
    pub next_diagnostics: Vec<AdapterCapabilityDiagnostic>,
    /// Capability changes, empty when either input is invalid.
    pub changes: Vec<CompatibilityChange>,
}

/// Compares adapter capability support from older to newer.
///
/// Adding support or widening an inclusive version range is additive.
/// Removing support or narrowing either bound is breaking.
pub fn compare_adapter_capabilities(
    previous: &AdapterCapabilityManifest,
    next: &AdapterCapabilityManifest,
) -> AdapterCapabilityCompatibilityReport {
    let previous_diagnostics = validate_adapter_capability_manifest(previous);
    let next_diagnostics = validate_adapter_capability_manifest(next);
    if !previous_diagnostics.is_empty() || !next_diagnostics.is_empty() {
        return AdapterCapabilityCompatibilityReport {
            previous_diagnostics,
            next_diagnostics,
            changes: Vec::new(),
        };
    }

    let mut changes = Vec::new();
    if previous.adapter != next.adapter {
        changes.push(change(
            CompatibilityChangeKind::Breaking,
            "adapter",
            "adapter identity changed",
        ));
    }

    let previous = by_id(previous);
    let next = by_id(next);
    for id in previous.keys().filter(|id| !next.contains_key(*id)) {
        changes.push(capability_change(
            CompatibilityChangeKind::Breaking,
            id,
            "",
            "capability support was removed",
        ));
    }
    for id in next.keys().filter(|id| !previous.contains_key(*id)) {
        changes.push(capability_change(
            CompatibilityChangeKind::Additive,
            id,
            "",
            "capability support was added",
        ));
    }
    for (id, old) in &previous {
        let Some(new) = next.get(id) else {
            continue;
        };
        if old.min_version != new.min_version {
            changes.push(capability_change(
                if new.min_version < old.min_version {
                    CompatibilityChangeKind::Additive
                } else {
                    CompatibilityChangeKind::Breaking
                },
                id,
                ".minVersion",
                if new.min_version < old.min_version {
                    "minimum supported version decreased"
                } else {
                    "minimum supported version increased"
                },
            ));
        }
        if old.max_version != new.max_version {
            changes.push(capability_change(
                if new.max_version > old.max_version {
                    CompatibilityChangeKind::Additive
                } else {
                    CompatibilityChangeKind::Breaking
                },
                id,
                ".maxVersion",
                if new.max_version > old.max_version {
                    "maximum supported version increased"
                } else {
                    "maximum supported version decreased"
                },
            ));
        }
    }

    changes.sort_by(|left, right| (&left.path, left.kind).cmp(&(&right.path, right.kind)));
    AdapterCapabilityCompatibilityReport {
        previous_diagnostics,
        next_diagnostics,
        changes,
    }
}

fn by_id(manifest: &AdapterCapabilityManifest) -> BTreeMap<&str, &super::AdapterCapabilitySupport> {
    manifest
        .capabilities
        .iter()
        .map(|capability| (capability.id.as_str(), capability))
        .collect()
}

fn capability_change(
    kind: CompatibilityChangeKind,
    id: &str,
    suffix: &str,
    message: &str,
) -> CompatibilityChange {
    let mut path = "capabilities.".to_compact_string();
    path.push_str(id);
    path.push_str(suffix);
    change(kind, &path, message)
}

fn change(kind: CompatibilityChangeKind, path: &str, message: &str) -> CompatibilityChange {
    CompatibilityChange {
        kind,
        path: String::from(path),
        message: String::from(message),
    }
}
