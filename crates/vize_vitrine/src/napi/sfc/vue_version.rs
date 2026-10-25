//! Resolve the requested runtime before parsing files or allocating workers.
//!
//! A syntax dialect is not a runtime backend. The current SFC emitter imports
//! Vue 3 helpers, so an explicit legacy selection must never fall through to it.

use napi::{Error, Result, Status};
use vize_l0::config::VueVersion;

pub(super) fn resolve_sfc_vue_version(value: Option<&str>) -> Result<VueVersion> {
    let version = value
        .map(VueVersion::from_config_str)
        .transpose()
        .map_err(|error| Error::new(Status::InvalidArg, error.to_string()))?
        .unwrap_or_default();
    match version {
        VueVersion::V3 => Ok(version),
        VueVersion::V2
        | VueVersion::V2_7
        | VueVersion::V1
        | VueVersion::V0_11
        | VueVersion::V0_10 => Err(Error::new(
            Status::InvalidArg,
            format!(
                "SFC code generation for vueVersion \"{}\" is not available: the native SFC compiler currently targets the Vue 3 runtime. Keep this project's Vue {} framework compiler enabled.",
                version.as_str(),
                version.as_str(),
            ),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_sfc_vue_version;
    use vize_l0::config::VueVersion;

    #[test]
    fn default_and_explicit_modern_select_the_same_runtime() {
        for value in [None, Some("3"), Some("v3"), Some(" V3 ")] {
            assert_eq!(resolve_sfc_vue_version(value).unwrap(), VueVersion::V3);
        }
    }

    #[test]
    fn known_legacy_dialects_require_a_real_runtime_backend() {
        for (version, canonical) in [
            ("2", "2"),
            ("2.7", "2.7"),
            ("v2", "2"),
            (" V2.7 ", "2.7"),
            ("1", "1"),
            ("0.11", "0.11"),
            ("0.10", "0.10"),
        ] {
            let error = resolve_sfc_vue_version(Some(version)).unwrap_err();
            assert_eq!(error.status, napi::Status::InvalidArg);
            assert_eq!(
                error.reason,
                format!(
                    "SFC code generation for vueVersion \"{canonical}\" is not available: the native SFC compiler currently targets the Vue 3 runtime. Keep this project's Vue {canonical} framework compiler enabled."
                )
            );
        }
    }

    #[test]
    fn unknown_or_ambiguous_dialects_never_default_to_modern() {
        for version in [
            "", "2.6", "2.6.14", "2.7.16", "3.5", "4", "0", "Vue2", "2.7-beta",
        ] {
            let error = resolve_sfc_vue_version(Some(version)).unwrap_err();
            assert_eq!(error.status, napi::Status::InvalidArg);
            let reference = VueVersion::from_config_str(version).unwrap_err();
            assert_eq!(error.reason, reference.to_string());
        }
    }
}
