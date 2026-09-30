use std::path::Path;

use super::load_raw_config_with_source;
use crate::config::LanguageServerUnstableFlags;

/// Load the `languageServer` switches that are not stable model fields
/// (currently `signatureHelp`); defaults when unset.
///
/// These live off [`crate::config::LanguageServerConfig`] so the public model
/// stays additively stable for `cargo-semver-checks`, mirroring how
/// `linter.ruleOptions` rides on the internal raw linter config.
pub fn load_language_server_unstable_flags(path: Option<&Path>) -> LanguageServerUnstableFlags {
    let loaded = load_raw_config_with_source(path);
    loaded.config.language_server_unstable_flags()
}

#[cfg(test)]
mod tests {
    use super::load_language_server_unstable_flags;

    fn flags_for(config: &str) -> Option<bool> {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("vize.config.json"), config).unwrap();
        load_language_server_unstable_flags(Some(dir.path())).signature_help
    }

    #[test]
    fn signature_help_defaults_to_unset() {
        assert_eq!(flags_for("{}"), None);
        assert_eq!(
            flags_for(r#"{ "languageServer": { "hover": false } }"#),
            None
        );
    }

    #[test]
    fn signature_help_reads_the_language_server_section() {
        assert_eq!(
            flags_for(r#"{ "languageServer": { "signatureHelp": false } }"#),
            Some(false)
        );
        assert_eq!(
            flags_for(r#"{ "languageServer": { "signatureHelp": true } }"#),
            Some(true)
        );
    }

    #[test]
    fn signature_help_falls_back_to_the_legacy_lsp_section() {
        assert_eq!(
            flags_for(r#"{ "lsp": { "signatureHelp": false } }"#),
            Some(false)
        );
    }

    #[test]
    fn language_server_section_wins_over_the_legacy_lsp_section() {
        assert_eq!(
            flags_for(
                r#"{
                    "languageServer": { "editor": true, "signatureHelp": true },
                    "lsp": { "signatureHelp": false }
                }"#
            ),
            Some(true)
        );
    }
}
