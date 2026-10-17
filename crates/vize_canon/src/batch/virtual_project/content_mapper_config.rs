//! Project-scoped configuration for the official TypeScript content mapper.

use std::path::Path;

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use vize_l0::{String as CompactString, cstr};

use super::ContentMapperTransformOptions;
use super::tsconfig_gen::snapshot_vue_compiler_options;
use crate::batch::error::CorsaResult;

/// A snapshot selected by TypeScript's explicit `configFileName`.
/// No directory search or Vize-specific configuration is performed.
#[derive(Clone, Debug)]
pub struct ContentMapperProjectConfiguration {
    /// Generation switches derived from the selected project's Vue options.
    pub transform_options: ContentMapperTransformOptions,
    /// Stable identity for effective dynamic options and their resolution closure.
    pub config_identity: CompactString,
    /// Absolute configs, package metadata, and missing candidates affecting resolution.
    pub watched_files: Vec<CompactString>,
}

impl ContentMapperProjectConfiguration {
    /// Read only the host-selected tsconfig and its existing `extends` chain.
    /// An empty path denotes an inferred project and performs no file reads.
    pub fn load(config_file_name: &Path) -> CorsaResult<Self> {
        let (options, dependencies) = if config_file_name.as_os_str().is_empty() {
            (Map::new(), Vec::new())
        } else {
            snapshot_vue_compiler_options(config_file_name)?
        };
        let watched_files: Vec<CompactString> = dependencies
            .iter()
            .map(|path| path.to_string_lossy().into())
            .collect();
        // Fingerprint the effective dynamic options and resolution closure,
        // rather than rereading files after the snapshot has been selected.
        let identity = serde_json::to_vec(&(&options, &watched_files))?;
        let digest = Sha256::digest(identity);
        let transform_options = if config_file_name.as_os_str().is_empty() {
            ContentMapperTransformOptions::default()
        } else {
            ContentMapperTransformOptions::default().with_vue_compiler_options(&options)
        };
        Ok(Self {
            transform_options,
            config_identity: cstr!("vize-content-mapper-v1:{digest:x}"),
            watched_files,
        })
    }
}

impl ContentMapperTransformOptions {
    #[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
    pub(super) fn with_vue_compiler_options(
        mut self,
        options: &Map<std::string::String, Value>,
    ) -> Self {
        let explicit = options.get("checkUnknownProps").and_then(Value::as_bool);
        self.check_unknown_props = explicit
            .or_else(|| options.get("strictTemplates").and_then(Value::as_bool))
            .unwrap_or(false);
        self.check_unknown_fallthrough_props = explicit == Some(true);
        self.strict_css_modules = options
            .get("strictCssModules")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::ContentMapperProjectConfiguration;
    use std::path::Path;

    fn load(path: &Path) -> ContentMapperProjectConfiguration {
        ContentMapperProjectConfiguration::load(path).unwrap()
    }

    #[test]
    fn configured_defaults_and_inferred_defaults_remain_distinct() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("tsconfig.json");
        std::fs::write(&path, "{}").unwrap();
        let configured = load(&path);
        assert!(!configured.transform_options.check_unknown_props);
        assert!(!configured.transform_options.strict_css_modules);
        assert_eq!(configured.watched_files.len(), 1);
        assert_eq!(configured.watched_files[0].as_str(), path.to_string_lossy());
        let inferred = load(Path::new(""));
        assert!(inferred.transform_options.check_unknown_props);
        assert!(!inferred.transform_options.check_unknown_fallthrough_props);
        assert!(!inferred.transform_options.strict_css_modules);
        assert!(inferred.watched_files.is_empty());
        assert_ne!(configured.config_identity, inferred.config_identity);
    }

    #[test]
    fn shared_chain_obeys_jsonc_array_precedence_and_explicit_false() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        std::fs::write(
            root.join("left.json"),
            r#"{
            // A real inherited option, including a trailing comma.
            "vueCompilerOptions": {"checkUnknownProps": true, "strictCssModules": true,},
        }"#,
        )
        .unwrap();
        std::fs::write(
            root.join("right.json"),
            r#"{
            "extends": "./left.json", "vueCompilerOptions": {"strictCssModules": false}
        }"#,
        )
        .unwrap();
        let path = root.join("tsconfig.json");
        std::fs::write(
            &path,
            r#"{
            "extends": ["./left.json", "./right.json"],
            "vueCompilerOptions": {"strictTemplates": true, "checkUnknownProps": false}
        }"#,
        )
        .unwrap();
        let configuration = load(&path);
        assert!(!configuration.transform_options.check_unknown_props);
        assert!(
            !configuration
                .transform_options
                .check_unknown_fallthrough_props
        );
        assert!(!configuration.transform_options.strict_css_modules);
        assert_eq!(configuration.watched_files.len(), 3);
        for name in ["tsconfig.json", "left.json", "right.json"] {
            assert!(
                configuration
                    .watched_files
                    .iter()
                    .any(|path| path.ends_with(name))
            );
        }
    }

    #[test]
    fn package_tsconfig_field_and_real_parent_are_in_the_dynamic_closure() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let package = root.join("node_modules/tsconfig-preset");
        std::fs::create_dir_all(&package).unwrap();
        std::fs::write(package.join("package.json"), r#"{"tsconfig":"vue.json"}"#).unwrap();
        std::fs::write(
            package.join("vue.json"),
            r#"{
            "vueCompilerOptions": {"checkUnknownProps":true,"strictCssModules":true}
        }"#,
        )
        .unwrap();
        let path = root.join("tsconfig.json");
        std::fs::write(&path, r#"{"extends":"tsconfig-preset"}"#).unwrap();
        let configuration = load(&path);
        assert!(configuration.transform_options.check_unknown_props);
        assert!(
            configuration
                .transform_options
                .check_unknown_fallthrough_props
        );
        assert!(configuration.transform_options.strict_css_modules);
        assert!(
            configuration
                .watched_files
                .iter()
                .any(|path| path.ends_with("tsconfig-preset/package.json"))
        );
        assert!(
            configuration
                .watched_files
                .iter()
                .any(|path| path.ends_with("tsconfig-preset/vue.json"))
        );
        assert!(
            configuration
                .watched_files
                .iter()
                .all(|path| Path::new(path.as_str()).is_absolute())
        );
    }

    #[test]
    fn dynamic_identity_is_stable_and_tracks_inherited_option_changes() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let path = root.join("tsconfig.json");
        let base = root.join("base.json");
        std::fs::write(&path, r#"{"extends":"./base.json"}"#).unwrap();
        std::fs::write(&base, r#"{"vueCompilerOptions":{"strictTemplates":true}}"#).unwrap();
        let before = load(&path);
        assert!(before.transform_options.check_unknown_props);
        assert!(!before.transform_options.check_unknown_fallthrough_props);
        assert_eq!(before.config_identity, load(&path).config_identity);
        std::fs::write(&base, r#"{"vueCompilerOptions":{"strictTemplates":true,"checkUnknownProps":false,"strictCssModules":true}}"#).unwrap();
        let after = load(&path);
        assert!(!after.transform_options.check_unknown_props);
        assert!(after.transform_options.strict_css_modules);
        assert_ne!(before.config_identity, after.config_identity);
        assert_eq!(before.watched_files, after.watched_files);
    }

    #[test]
    fn cycle_cut_keeps_the_existing_chain_precedence() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let path = root.join("tsconfig.json");
        std::fs::write(
            &path,
            r#"{"extends":"./base.json","vueCompilerOptions":{"checkUnknownProps":false}}"#,
        )
        .unwrap();
        std::fs::write(root.join("base.json"), r#"{"extends":"./tsconfig.json","vueCompilerOptions":{"checkUnknownProps":true,"strictCssModules":true}}"#).unwrap();
        let configuration = load(&path);
        assert!(!configuration.transform_options.check_unknown_props);
        assert!(configuration.transform_options.strict_css_modules);
        assert_eq!(configuration.watched_files.len(), 2);
    }

    #[test]
    fn malformed_inherited_configuration_is_not_replaced_by_defaults() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("tsconfig.json");
        std::fs::write(&path, r#"{"extends":"./base.json"}"#).unwrap();
        std::fs::write(directory.path().join("base.json"), "{ invalid JSON").unwrap();
        let error = ContentMapperProjectConfiguration::load(&path).unwrap_err();
        assert!(vize_l0::cstr!("{error}").contains("base.json"));
        std::fs::remove_file(directory.path().join("base.json")).unwrap();
        let error = ContentMapperProjectConfiguration::load(&path).unwrap_err();
        assert!(vize_l0::cstr!("{error}").contains("base.json"));
        assert!(vize_l0::cstr!("{error}").contains("Cannot resolve"));
    }
}
