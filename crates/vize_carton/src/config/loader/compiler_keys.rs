//! Loaders for individual `compiler.*` configuration keys.
//!
//! Each entry reads the raw config once and projects a single key, keeping the
//! loader entry points in `loader.rs` focused on whole-config loading.

use std::path::Path;

use super::load_raw_config_with_source;

/// Load the configured `compiler.templateSyntax` value from a directory or file path.
pub fn load_compiler_template_syntax(path: Option<&Path>) -> Option<&'static str> {
    load_raw_config_with_source(path)
        .config
        .compiler
        .template_syntax
        .map(|template_syntax| template_syntax.as_str())
}

/// Load the configured `vue.version` dialect from a directory or file path.
///
/// Returns `None` when the key is absent (modern Vue 3). Unknown or ambiguous
/// values fail config parsing earlier, so a returned value always names a valid
/// dialect. The build runner threads this into the per-file compile options so
/// it reaches the parser/transform layer.
pub fn load_compiler_vue_version(path: Option<&Path>) -> Option<crate::config::VueVersion> {
    let loaded = load_raw_config_with_source(path);
    let (_, features) = loaded.config.into_config_and_features();
    features.vue_version
}

/// Load `compiler.compatibility.hostCompiler` when explicitly configured.
pub fn load_compiler_host_compiler(path: Option<&Path>) -> Option<bool> {
    load_raw_config_with_source(path)
        .config
        .compiler
        .compatibility
        .host_compiler
}

/// Load the configured `compiler.jsxMode` default output mode (#1496).
///
/// Returns `None` when the key is absent (treated as VDOM by the JSX entry
/// points). The build runner and plugins thread this into the native
/// `compileJsx` mode-selection logic, where a per-component `"use vue:*"`
/// directive can still override it.
pub fn load_compiler_jsx_mode(path: Option<&Path>) -> Option<crate::config::JsxMode> {
    let loaded = load_raw_config_with_source(path);
    let (_, features) = loaded.config.into_config_and_features();
    features.jsx_mode
}

/// Load configured `compiler.customElements` tag patterns.
pub fn load_compiler_custom_elements(path: Option<&Path>) -> Vec<crate::String> {
    load_raw_config_with_source(path)
        .config
        .compiler
        .custom_elements
}

#[cfg(test)]
mod tests {
    use super::{
        load_compiler_custom_elements, load_compiler_host_compiler, load_compiler_template_syntax,
    };

    #[test]
    fn load_config_reads_compiler_template_syntax() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("vize.config.json");
        std::fs::write(
            &config_path,
            r#"{ "compiler": { "templateSyntax": "quirks" } }"#,
        )
        .unwrap();

        assert_eq!(
            load_compiler_template_syntax(Some(&config_path)),
            Some("quirks")
        );
    }

    #[test]
    fn load_config_reads_compiler_custom_elements() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("vize.config.json");
        std::fs::write(
            &config_path,
            r#"{ "compiler": { "customElements": ["Tres*", "primitive"] } }"#,
        )
        .unwrap();

        let custom_elements = load_compiler_custom_elements(Some(&config_path));
        assert_eq!(
            custom_elements
                .iter()
                .map(|pattern| pattern.as_str())
                .collect::<Vec<_>>(),
            ["Tres*", "primitive"]
        );
    }

    #[test]
    fn load_compiler_host_compiler_reads_compiler_compatibility_key() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("vize.config.json");
        std::fs::write(
            &config_path,
            r#"{ "compiler": { "compatibility": { "hostCompiler": false } } }"#,
        )
        .unwrap();

        assert_eq!(load_compiler_host_compiler(Some(&config_path)), Some(false));
    }
}
