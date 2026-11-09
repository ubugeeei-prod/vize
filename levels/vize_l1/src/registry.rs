//! L1 syntax-policy registry.
//!
//! Concrete Vue versions are named only here. Parser/container code asks this
//! registry for a capability table or a checked component profile, keeping
//! variant selection out of generic L1 owners.

use vize_l0::{FileDialect, TemplateSyntaxMode, config::VueVersion, dialect::VueDialect};

use crate::dialect::{LegacyDialectCapabilities, LegacyVueVersion};

/// A component profile accepted by one version-owned L1 provider.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ComponentProfile {
    version: VueVersion,
}

impl ComponentProfile {
    #[must_use]
    pub(crate) fn accepts_version(self, version: VueVersion) -> bool {
        self.version == version
    }

    #[must_use]
    pub(crate) const fn accepts_framework(self, framework: VueDialect) -> bool {
        matches!(framework, VueDialect::Vue)
    }

    #[must_use]
    pub(crate) const fn accepts_syntax(self, syntax: TemplateSyntaxMode) -> bool {
        matches!(syntax, TemplateSyntaxMode::Standard)
    }
}

pub(crate) const VUE3_COMPONENT: ComponentProfile = ComponentProfile {
    version: VueVersion::V3,
};
pub(crate) const VUE2_COMPONENT: ComponentProfile = ComponentProfile {
    version: VueVersion::V2,
};
pub(crate) const VUE1_COMPONENT: ComponentProfile = ComponentProfile {
    version: VueVersion::V1,
};

/// Resolve a configured version into the historical public legacy line.
#[must_use]
pub const fn legacy_version(version: VueVersion) -> Option<LegacyVueVersion> {
    match version {
        VueVersion::V3 => None,
        VueVersion::V2_7 | VueVersion::V2 => Some(LegacyVueVersion::V2),
        VueVersion::V1 => Some(LegacyVueVersion::V1),
        VueVersion::V0_11 => Some(LegacyVueVersion::V0_11),
        VueVersion::V0_10 => Some(LegacyVueVersion::V0_10),
    }
}

/// Resolve a legacy line into its version-composed feature table.
#[must_use]
pub const fn legacy_capabilities(version: LegacyVueVersion) -> LegacyDialectCapabilities {
    match version {
        LegacyVueVersion::V0_10 => crate::dialect::vue0::V0_10,
        LegacyVueVersion::V0_11 => crate::dialect::vue0::V0_11,
        LegacyVueVersion::V1 => crate::dialect::vue1::CAPABILITIES,
        LegacyVueVersion::V2 => crate::dialect::vue2::CAPABILITIES,
    }
}

/// Resolve the complete syntax capability table once per file.
#[must_use]
pub const fn capabilities(version: VueVersion) -> LegacyDialectCapabilities {
    match legacy_version(version) {
        None => crate::dialect::vue3::CAPABILITIES,
        Some(version) => legacy_capabilities(version),
    }
}

/// Whether an L1 owner may expose the standard Vue 3 component profile.
#[must_use]
pub fn is_standard_vue3_component(dialect: FileDialect) -> bool {
    vize_l0::registry::is_standard_vue_component(dialect, VueVersion::V3)
}

/// Whether an L1 owner may expose the standard Vue 2 component profile.
#[must_use]
pub fn is_standard_vue2_component(dialect: FileDialect) -> bool {
    vize_l0::registry::is_standard_vue_component(dialect, VueVersion::V2)
}

/// Whether an L1 owner may expose the standard Vue 1 component profile.
#[must_use]
pub fn is_standard_vue1_component(dialect: FileDialect) -> bool {
    vize_l0::registry::is_standard_vue_component(dialect, VueVersion::V1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vize_l0::{Lang, ProfileKind};

    #[test]
    fn every_configured_version_resolves_to_the_existing_table() {
        let cases = [
            (
                VueVersion::V0_10,
                Some(LegacyVueVersion::V0_10),
                crate::dialect::vue0::V0_10,
            ),
            (
                VueVersion::V0_11,
                Some(LegacyVueVersion::V0_11),
                crate::dialect::vue0::V0_11,
            ),
            (
                VueVersion::V1,
                Some(LegacyVueVersion::V1),
                crate::dialect::vue1::CAPABILITIES,
            ),
            (
                VueVersion::V2,
                Some(LegacyVueVersion::V2),
                crate::dialect::vue2::CAPABILITIES,
            ),
            (
                VueVersion::V2_7,
                Some(LegacyVueVersion::V2),
                crate::dialect::vue2::CAPABILITIES,
            ),
            (VueVersion::V3, None, crate::dialect::vue3::CAPABILITIES),
        ];
        for (version, expected_legacy, expected_capabilities) in cases {
            assert_eq!(legacy_version(version), expected_legacy, "{version:?}");
            assert_eq!(capabilities(version), expected_capabilities, "{version:?}");
        }
    }

    #[test]
    fn component_profiles_keep_version_and_recovery_orthogonal() {
        let vue2 = FileDialect::component(
            VueVersion::V2,
            VueDialect::Vue,
            TemplateSyntaxMode::Standard,
            Lang::Js,
        );
        assert!(is_standard_vue2_component(vue2));
        assert!(!is_standard_vue3_component(vue2));
        assert_eq!(vue2.profile(), ProfileKind::Component);
        assert_eq!(vue2.lang(), Lang::Js);
    }
}
