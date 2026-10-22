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
        let digest = super::identity::encode_digest(Sha256::digest(identity));
        let transform_options = if config_file_name.as_os_str().is_empty() {
            ContentMapperTransformOptions::default()
        } else {
            ContentMapperTransformOptions::default()
                .with_vue_compiler_options(config_file_name, &options)?
        };
        Ok(Self {
            transform_options,
            config_identity: cstr!("vize-content-mapper-v1:{digest}"),
            watched_files,
        })
    }
}

impl ContentMapperTransformOptions {
    #[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
    pub(super) fn with_vue_compiler_options(
        mut self,
        config_file_name: &Path,
        options: &Map<std::string::String, Value>,
    ) -> CorsaResult<Self> {
        let strict_templates = options.get("strictTemplates").and_then(Value::as_bool);
        self.check_unknown_components = options
            .get("checkUnknownComponents")
            .and_then(Value::as_bool)
            .or(strict_templates)
            .unwrap_or(false);
        let explicit = options.get("checkUnknownProps").and_then(Value::as_bool);
        self.check_unknown_props = explicit.or(strict_templates).unwrap_or(false);
        self.check_unknown_fallthrough_props = explicit == Some(true);
        self.check_unknown_directives = options
            .get("checkUnknownDirectives")
            .and_then(Value::as_bool)
            .or(strict_templates)
            .unwrap_or(false);
        self.strict_css_modules = options
            .get("strictCssModules")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        self.vue_compiler_target =
            super::tsconfig_gen::vue_compiler_options::vue_target_from_options(
                config_file_name.parent().unwrap_or(Path::new(".")),
                config_file_name,
                options,
                None,
            )?;
        Ok(self)
    }

    pub(super) fn virtual_ts_check_options(self) -> crate::virtual_ts::VirtualTsCheckOptions {
        let mut checks = crate::virtual_ts::VirtualTsCheckOptions {
            check_unknown_components: self.check_unknown_components,
            check_unknown_props: self.check_unknown_props,
            check_unknown_fallthrough_props: self.check_unknown_fallthrough_props,
            check_unknown_directives: self.check_unknown_directives,
            vue_compiler_target: self.vue_compiler_target,
            ..Default::default()
        };
        #[cfg(feature = "native")]
        {
            checks.strict_css_modules = self.strict_css_modules;
        }
        checks
    }
}

#[cfg(test)]
mod tests {
    use super::super::generate_vue_content_mapper_transform_with_options;
    use super::{ContentMapperProjectConfiguration, ContentMapperTransformOptions};
    use crate::batch::error::CorsaError;
    use std::io::ErrorKind;
    use std::path::Path;
    use vize_carton::config::VueVersion;

    fn load(path: &Path) -> ContentMapperProjectConfiguration {
        ContentMapperProjectConfiguration::load(path).unwrap()
    }

    fn expected_options(
        mut options: ContentMapperTransformOptions,
        components: bool,
        unknown: bool,
        fallthrough: bool,
        directives: bool,
        css: bool,
    ) -> ContentMapperTransformOptions {
        options.check_unknown_components = components;
        options.check_unknown_props = unknown;
        options.check_unknown_fallthrough_props = fallthrough;
        options.check_unknown_directives = directives;
        options.strict_css_modules = css;
        options
    }

    fn expected_watches(root: &Path, names: &[&str]) -> Vec<vize_l0::String> {
        let mut paths: Vec<_> = names
            .iter()
            .map(|name| root.join(name).to_string_lossy().into_owned().into())
            .collect();
        paths.sort();
        paths.dedup();
        paths
    }

    fn expected_canonical_watch(path: &Path) -> vize_l0::String {
        let text = path.canonicalize().unwrap().to_string_lossy().into_owned();
        // The host returns ordinary DOS/UNC paths, not Windows verbatim paths.
        // Form the oracle from the OS result without calling the resolver's helper.
        #[cfg(windows)]
        let text = text
            .strip_prefix(r"\\?\UNC\")
            .map(|unc| format!(r"\\{unc}"))
            .or_else(|| text.strip_prefix(r"\\?\").map(str::to_owned))
            .unwrap_or(text);
        text.into()
    }

    #[test]
    fn inferred_identity_retains_the_complete_sha256_digest() {
        let configuration = load(Path::new(""));
        assert_eq!(
            configuration.config_identity.as_str(),
            "vize-content-mapper-v1:7a38905296006c575d7d45d1d881690a3ee87361ed67ce64642e2504482da818"
        );
    }

    #[test]
    fn configured_defaults_and_inferred_defaults_remain_distinct() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("tsconfig.json");
        std::fs::write(&path, "{}").unwrap();
        let configured = load(&path);
        assert_eq!(
            configured.transform_options,
            expected_options(Default::default(), false, false, false, false, false)
        );
        assert_eq!(
            configured.watched_files,
            expected_watches(directory.path(), &["tsconfig.json"])
        );
        let inferred = load(Path::new(""));
        assert_eq!(
            inferred.transform_options,
            expected_options(Default::default(), false, true, false, false, false)
        );
        assert_eq!(
            inferred.watched_files,
            expected_watches(directory.path(), &[])
        );
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
        assert_eq!(
            configuration.transform_options,
            expected_options(Default::default(), true, false, false, true, false)
        );
        assert_eq!(
            configuration.watched_files,
            expected_watches(root, &["tsconfig.json", "left.json", "right.json"])
        );
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
        assert_eq!(
            configuration.transform_options,
            expected_options(Default::default(), false, true, true, false, true)
        );
        let mut expected = expected_watches(
            root,
            &[
                "tsconfig.json",
                "node_modules/tsconfig-preset/package.json",
                "node_modules/tsconfig-preset/vue.json",
            ],
        );
        // Resolution observes the lexical probe, then reads its canonical target.
        // Both are retained when the temporary directory itself has a symlink alias.
        expected.push(expected_canonical_watch(&package.join("vue.json")));
        expected.sort();
        expected.dedup();
        assert_eq!(configuration.watched_files, expected);
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
        assert_eq!(
            before.transform_options,
            expected_options(Default::default(), true, true, false, true, false)
        );
        assert_eq!(before.config_identity, load(&path).config_identity);
        std::fs::write(&base, r#"{"vueCompilerOptions":{"strictTemplates":true,"checkUnknownProps":false,"strictCssModules":true}}"#).unwrap();
        let after = load(&path);
        assert_eq!(
            after.transform_options,
            expected_options(Default::default(), true, false, false, true, true)
        );
        assert_ne!(before.config_identity, after.config_identity);
        assert_eq!(before.watched_files, after.watched_files);
        assert_eq!(
            after.watched_files,
            expected_watches(root, &["tsconfig.json", "base.json"])
        );
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
        assert_eq!(
            configuration.transform_options,
            expected_options(Default::default(), false, false, false, false, true)
        );
        assert_eq!(
            configuration.watched_files,
            expected_watches(root, &["tsconfig.json", "base.json"])
        );
    }

    #[test]
    fn malformed_inherited_configuration_is_not_replaced_by_defaults() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("tsconfig.json");
        std::fs::write(&path, r#"{"extends":"./base.json"}"#).unwrap();
        std::fs::write(directory.path().join("base.json"), "{ invalid JSON").unwrap();
        let CorsaError::Io(error) = ContentMapperProjectConfiguration::load(&path).unwrap_err()
        else {
            panic!("malformed inherited configuration must produce an IO error");
        };
        assert_eq!(error.kind(), ErrorKind::InvalidData);
        assert_eq!(
            error.to_string().as_str(),
            vize_l0::cstr!(
                "Invalid tsconfig '{}': JSON parse error: key must be a string at line 1 column 3",
                directory.path().join("base.json").display()
            )
            .as_str()
        );
        std::fs::remove_file(directory.path().join("base.json")).unwrap();
        let CorsaError::Io(error) = ContentMapperProjectConfiguration::load(&path).unwrap_err()
        else {
            panic!("missing inherited configuration must produce an IO error");
        };
        assert_eq!(error.kind(), ErrorKind::NotFound);
        assert_eq!(
            error.to_string().as_str(),
            vize_l0::cstr!(
                "Cannot resolve extended tsconfig './base.json' from '{}'",
                path.display()
            )
            .as_str()
        );
    }

    fn has_fragment(code: &str, needle: &str) -> bool {
        code.split(needle).nth(1).is_some()
    }

    const INTRINSIC_SFC: &str =
        "<script setup lang=\"ts\"></script>\n<template><div x-small></div></template>\n";

    #[test]
    fn vue3_target_drops_intrinsic_attribute_excess_checks() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("tsconfig.json");
        std::fs::write(
            &path,
            r#"{"vueCompilerOptions":{"target":3,"checkUnknownProps":true}}"#,
        )
        .unwrap();
        let configured = load(&path);
        assert_eq!(
            configured.transform_options.vue_compiler_target,
            Some(VueVersion::V3)
        );
        let generated = generate_vue_content_mapper_transform_with_options(
            Path::new("src/Native.vue"),
            INTRINSIC_SFC,
            configured.transform_options,
        )
        .unwrap();
        let excess = has_fragment(&generated.text, "__vize_native_key_");
        assert!(!excess, "{}", generated.text);

        std::fs::write(
            &path,
            r#"{"vueCompilerOptions":{"checkUnknownProps":true}}"#,
        )
        .unwrap();
        let omitted = load(&path);
        assert_eq!(omitted.transform_options.vue_compiler_target, None);
        let checked = generate_vue_content_mapper_transform_with_options(
            Path::new("src/Native.vue"),
            INTRINSIC_SFC,
            omitted.transform_options,
        )
        .unwrap();
        let still_checked = has_fragment(&checked.text, "__vize_native_key_");
        assert!(still_checked, "{}", checked.text);
    }

    #[test]
    fn unsupported_vue_compiler_target_is_configuration_error() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("tsconfig.json");
        std::fs::write(&path, r#"{"vueCompilerOptions":{"target":1}}"#).unwrap();
        let CorsaError::Configuration(message) =
            ContentMapperProjectConfiguration::load(&path).unwrap_err()
        else {
            panic!("unsupported target must be a configuration error");
        };
        let names_target = has_fragment(message.as_str(), "vueCompilerOptions.target");
        assert!(names_target, "{message}");
    }
}
