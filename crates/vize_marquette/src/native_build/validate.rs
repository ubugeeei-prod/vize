use std::collections::BTreeSet;

use vize_l0::{String, ToCompactString};

use super::{
    NATIVE_BUILD_PROFILE_FORMAT, NATIVE_BUILD_PROFILE_FORMAT_VERSION, NativeBuildConfiguration,
    NativeBuildDestination, NativeBuildProfile, NativeBuildProfileDiagnostic,
    NativeBuildProfileDiagnosticCode, NativeBuildSigning,
};

/// Validates a native build profile without mutating it.
pub fn validate_native_build_profile(
    profile: &NativeBuildProfile,
) -> Vec<NativeBuildProfileDiagnostic> {
    let mut diagnostics = Vec::new();
    if profile.format != NATIVE_BUILD_PROFILE_FORMAT {
        diagnostics.push(diagnostic(
            NativeBuildProfileDiagnosticCode::InvalidFormat,
            "format",
            "unsupported native build profile format",
        ));
    }
    if profile.format_version != NATIVE_BUILD_PROFILE_FORMAT_VERSION {
        diagnostics.push(diagnostic(
            NativeBuildProfileDiagnosticCode::InvalidFormatVersion,
            "formatVersion",
            "unsupported native build profile format version",
        ));
    }
    if !is_identifier(&profile.profile) {
        diagnostics.push(diagnostic(
            NativeBuildProfileDiagnosticCode::InvalidProfileId,
            "profile",
            "profile must be a lowercase portable identifier",
        ));
    }
    if !is_identifier(&profile.environment) {
        diagnostics.push(diagnostic(
            NativeBuildProfileDiagnosticCode::InvalidEnvironmentId,
            "environment",
            "environment must be a lowercase portable identifier",
        ));
    }
    if profile.targets.is_empty() {
        diagnostics.push(diagnostic(
            NativeBuildProfileDiagnosticCode::MissingTarget,
            "targets",
            "native build profile must declare a target",
        ));
    }

    let mut seen_targets = BTreeSet::new();
    for (index, target) in profile.targets.iter().enumerate() {
        let path = indexed_path("targets", index);
        if !seen_targets.insert((target.platform, target.destination)) {
            diagnostics.push(diagnostic(
                NativeBuildProfileDiagnosticCode::DuplicateTarget,
                &path,
                "platform and destination must be unique within the native build profile",
            ));
        }
        if target.destination == NativeBuildDestination::Store
            && profile.configuration != NativeBuildConfiguration::Production
        {
            diagnostics.push(diagnostic(
                NativeBuildProfileDiagnosticCode::StoreRequiresProduction,
                &path,
                "store targets require the production configuration",
            ));
        }
        if target.destination == NativeBuildDestination::Store
            && matches!(target.signing, None | Some(NativeBuildSigning::None))
        {
            diagnostics.push(diagnostic(
                NativeBuildProfileDiagnosticCode::StoreRequiresSigning,
                &field_path(&path, "signing"),
                "store targets require automatic or manual signing",
            ));
        }
    }

    if let Some(updates) = &profile.updates {
        if !is_identifier(&updates.channel) {
            diagnostics.push(diagnostic(
                NativeBuildProfileDiagnosticCode::InvalidUpdateChannel,
                "updates.channel",
                "update channel must be a lowercase portable identifier",
            ));
        }
        if !is_runtime_version(&updates.runtime_version) {
            diagnostics.push(diagnostic(
                NativeBuildProfileDiagnosticCode::InvalidRuntimeVersion,
                "updates.runtimeVersion",
                "runtime version must be a portable non-empty version token",
            ));
        }
    }

    let mut seen_capabilities = BTreeSet::new();
    for (index, capability) in profile.capabilities.iter().enumerate() {
        let path = indexed_path("capabilities", index);
        if !is_identifier(capability) {
            diagnostics.push(diagnostic(
                NativeBuildProfileDiagnosticCode::InvalidCapabilityId,
                &path,
                "capability id must be a lowercase portable identifier",
            ));
        }
        if !seen_capabilities.insert(capability.as_str()) {
            diagnostics.push(diagnostic(
                NativeBuildProfileDiagnosticCode::DuplicateCapability,
                &path,
                "capability id must be unique within the native build profile",
            ));
        }
    }

    diagnostics.sort_by(|left, right| {
        (&left.path, left.code, &left.message).cmp(&(&right.path, right.code, &right.message))
    });
    diagnostics
}

fn diagnostic(
    code: NativeBuildProfileDiagnosticCode,
    path: &str,
    message: &str,
) -> NativeBuildProfileDiagnostic {
    NativeBuildProfileDiagnostic {
        code,
        path: path.into(),
        message: message.into(),
    }
}

fn indexed_path(parent: &str, index: usize) -> String {
    let mut path = parent.to_compact_string();
    path.push('.');
    path.push_str(&index.to_compact_string());
    path
}

fn field_path(parent: &str, field: &str) -> String {
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

fn is_runtime_version(value: &str) -> bool {
    let mut characters = value.bytes();
    matches!(
        characters.next(),
        Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9')
    ) && characters.all(|character| {
        matches!(
            character,
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'.' | b'_' | b'+' | b'-'
        )
    })
}
