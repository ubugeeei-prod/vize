use std::collections::BTreeSet;

use vize_s0::{String, ToCompactString};

use super::{
    ADAPTER_CAPABILITY_FORMAT_VERSION, AdapterCapabilityDiagnostic,
    AdapterCapabilityDiagnosticCode, AdapterCapabilityManifest,
};

/// Validates an adapter capability manifest without mutating it.
pub fn validate_adapter_capability_manifest(
    manifest: &AdapterCapabilityManifest,
) -> Vec<AdapterCapabilityDiagnostic> {
    let mut diagnostics = Vec::new();
    if manifest.format_version != ADAPTER_CAPABILITY_FORMAT_VERSION {
        diagnostics.push(diagnostic(
            AdapterCapabilityDiagnosticCode::InvalidFormatVersion,
            "formatVersion",
            "unsupported adapter capability manifest format version",
        ));
    }
    if !is_identifier(&manifest.adapter) {
        diagnostics.push(diagnostic(
            AdapterCapabilityDiagnosticCode::InvalidAdapterId,
            "adapter",
            "adapter must be a lowercase portable identifier",
        ));
    }

    let mut seen = BTreeSet::new();
    for (index, capability) in manifest.capabilities.iter().enumerate() {
        let path = capability_path(index);
        if !is_identifier(&capability.id) {
            diagnostics.push(diagnostic(
                AdapterCapabilityDiagnosticCode::InvalidCapabilityId,
                &format_path(&path, "id"),
                "capability id must be a lowercase portable identifier",
            ));
        }
        if !seen.insert(capability.id.as_str()) {
            diagnostics.push(diagnostic(
                AdapterCapabilityDiagnosticCode::DuplicateCapability,
                &format_path(&path, "id"),
                "capability id must be unique within the adapter manifest",
            ));
        }
        if capability.min_version == 0 {
            diagnostics.push(diagnostic(
                AdapterCapabilityDiagnosticCode::InvalidVersion,
                &format_path(&path, "minVersion"),
                "minimum supported version must be greater than zero",
            ));
        }
        if capability.max_version == 0 {
            diagnostics.push(diagnostic(
                AdapterCapabilityDiagnosticCode::InvalidVersion,
                &format_path(&path, "maxVersion"),
                "maximum supported version must be greater than zero",
            ));
        }
        if capability.min_version > capability.max_version {
            diagnostics.push(diagnostic(
                AdapterCapabilityDiagnosticCode::InvalidVersionRange,
                &path,
                "minimum supported version must not exceed maximum supported version",
            ));
        }
    }

    diagnostics.sort_by(|left, right| {
        (&left.path, left.code, &left.message).cmp(&(&right.path, right.code, &right.message))
    });
    diagnostics
}

fn diagnostic(
    code: AdapterCapabilityDiagnosticCode,
    path: &str,
    message: &str,
) -> AdapterCapabilityDiagnostic {
    AdapterCapabilityDiagnostic {
        code,
        path: path.into(),
        message: message.into(),
    }
}

fn capability_path(index: usize) -> String {
    let mut path = "capabilities.".to_compact_string();
    path.push_str(&index.to_compact_string());
    path
}

fn format_path(parent: &str, field: &str) -> String {
    let mut path = parent.to_compact_string();
    path.push('.');
    path.push_str(field);
    path
}

fn is_identifier(value: &str) -> bool {
    let mut characters = value.bytes();
    matches!(characters.next(), Some(b'a'..=b'z' | b'0'..=b'9'))
        && characters
            .all(|character| matches!(character, b'a'..=b'z' | b'0'..=b'9' | b'.' | b'_' | b'-'))
}
