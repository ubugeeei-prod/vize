use std::path::Path;

use super::load_raw_config_with_source;

/// Load the configured `compiler.jsxCompat` JSX semantics (#3391).
///
/// Returns `None` when the key is absent, which the JSX entry points treat as
/// `native` — Vize's own semantics. `babel` opts into `@vue/babel-plugin-jsx`
/// semantics for projects migrating off the babel plugin.
pub fn load_compiler_jsx_compat(path: Option<&Path>) -> Option<crate::config::JsxCompat> {
    let loaded = load_raw_config_with_source(path);
    let (_, features) = loaded.config.into_config_and_features();
    features.jsx_compat
}

#[cfg(test)]
mod tests {
    use super::load_compiler_jsx_compat;
    use crate::config::loader::load_compiler_jsx_mode;
    use crate::config::{JsxCompat, JsxMode};

    #[test]
    fn load_compiler_jsx_mode_reads_jsx_mode_key() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("vize.config.json");
        std::fs::write(&config_path, r#"{ "compiler": { "jsxMode": "vapor" } }"#).unwrap();

        assert_eq!(
            load_compiler_jsx_mode(Some(&config_path)),
            Some(JsxMode::Vapor)
        );
        assert_eq!(
            load_compiler_jsx_mode(Some(&config_path)).map(JsxMode::as_str),
            Some("vapor")
        );
    }
    #[test]
    fn load_compiler_jsx_mode_reads_vdom() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("vize.config.json");
        std::fs::write(&config_path, r#"{ "compiler": { "jsxMode": "vdom" } }"#).unwrap();

        assert_eq!(
            load_compiler_jsx_mode(Some(&config_path)),
            Some(JsxMode::Vdom)
        );
    }
    #[test]
    fn load_compiler_jsx_mode_defaults_to_unset() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("vize.config.json");
        std::fs::write(&config_path, r#"{ "formatter": { "singleQuote": true } }"#).unwrap();

        // No `compiler.jsxMode` key → absent (the JSX entry points treat this as VDOM).
        assert_eq!(load_compiler_jsx_mode(Some(&config_path)), None);
    }
    #[test]
    fn load_compiler_jsx_compat_reads_babel_key() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("vize.config.json");
        std::fs::write(&config_path, r#"{ "compiler": { "jsxCompat": "babel" } }"#).unwrap();

        assert_eq!(
            load_compiler_jsx_compat(Some(&config_path)),
            Some(JsxCompat::Babel)
        );
        assert_eq!(
            load_compiler_jsx_compat(Some(&config_path)).map(JsxCompat::as_str),
            Some("babel")
        );
    }
    #[test]
    fn load_compiler_jsx_compat_reads_native() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("vize.config.json");
        std::fs::write(&config_path, r#"{ "compiler": { "jsxCompat": "native" } }"#).unwrap();

        assert_eq!(
            load_compiler_jsx_compat(Some(&config_path)),
            Some(JsxCompat::Native)
        );
    }
    #[test]
    fn load_compiler_jsx_compat_defaults_to_unset() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("vize.config.json");
        std::fs::write(&config_path, r#"{ "compiler": { "jsxMode": "vdom" } }"#).unwrap();

        // No `compiler.jsxCompat` key → absent, which the JSX entry points treat as
        // `native`. Setting only `jsxMode` must not imply a compatibility mode.
        assert_eq!(load_compiler_jsx_compat(Some(&config_path)), None);
    }
}
