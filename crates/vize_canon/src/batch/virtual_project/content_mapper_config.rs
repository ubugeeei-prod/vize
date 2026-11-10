//! Project-scoped configuration for the official TypeScript content mapper.

use std::path::{Path, PathBuf};

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use vize_l0::{String as CompactString, config::VueVersion, cstr};

use super::ContentMapperTransformOptions;
use super::tsconfig_gen::snapshot_vue_compiler_options;
use crate::batch::error::CorsaResult;

mod typed_router;
pub use typed_router::ContentMapperProjectContext;

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
    /// Selected project context; provider observations do not grant native authority.
    pub project_context: ContentMapperProjectContext,
}

impl ContentMapperProjectConfiguration {
    /// Read only the host-selected tsconfig and its existing `extends` chain.
    /// An empty path denotes an inferred project and performs no file reads.
    pub fn load(config_file_name: &Path) -> CorsaResult<Self> {
        Self::load_host_configuration(config_file_name, None)
            .map(|(configuration, _)| configuration)
    }

    /// Admit the host's effective compiler options alongside its selected Vue options.
    /// Compiler options are supplied by TypeScript, including command-line overrides.
    /// Returns the configuration and its owned, explicitly admitted typed-router root.
    #[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
    pub fn load_with_compiler_options(
        config_file_name: &Path,
        compiler_options: &Map<std::string::String, Value>,
    ) -> CorsaResult<(Self, Option<PathBuf>)> {
        vize_l0::profile!(
            "canon.content_mapper.project_configuration",
            Self::load_host_configuration(config_file_name, Some(compiler_options))
        )
    }

    #[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
    fn load_host_configuration(
        config_file_name: &Path,
        compiler_options: Option<&Map<std::string::String, Value>>,
    ) -> CorsaResult<(Self, Option<PathBuf>)> {
        let (options, mut dependencies) = if config_file_name.as_os_str().is_empty() {
            (Map::new(), Vec::new())
        } else {
            snapshot_vue_compiler_options(config_file_name)?
        };
        let (transform_options, vue_manifest) = if config_file_name.as_os_str().is_empty() {
            (ContentMapperTransformOptions::default(), None)
        } else {
            ContentMapperTransformOptions::default().with_vue_compiler_options(
                config_file_name,
                &options,
                &mut dependencies,
            )?
        };
        let mut watched_files: Vec<CompactString> = dependencies
            .iter()
            .map(|path| path.to_string_lossy().into())
            .collect();
        let typed_router_root = compiler_options.and_then(|compiler| {
            if config_file_name.as_os_str().is_empty() {
                return None;
            }
            let project_root = vize_carton::path::canonicalize_non_verbatim(
                config_file_name.parent().unwrap_or(Path::new(".")),
            );
            super::typed_router::configured_root_from_options(
                &project_root,
                Some(&options),
                compiler,
            )
        });
        let (project_context, provider_identity) = ContentMapperProjectContext::capture(
            typed_router_root.clone(),
            config_file_name,
            compiler_options,
            &mut watched_files,
        )?;
        if let Some(manifest) = vue_manifest.as_ref() {
            watched_files.push(manifest.to_string_lossy().into());
        }
        if vue_manifest.is_some() || provider_identity.is_some() {
            watched_files.sort();
            watched_files.dedup();
        }
        // Fingerprint the effective dynamic options and resolution closure.
        // `target: "auto"` also depends on the installed Vue manifest, so the
        // resolved dialect joins the identity. A numeric target is already in
        // the snapshotted options.
        let identity = if vue_manifest.is_some() {
            serde_json::to_vec(&(
                &options,
                &watched_files,
                resolved_target_tag(transform_options.checks.vue_compiler_target),
            ))?
        } else {
            serde_json::to_vec(&(&options, &watched_files))?
        };
        let mut digest = Sha256::new();
        digest.update(identity);
        if let Some(root) = typed_router_root.as_ref() {
            digest.update(b"\0typed-router-root\0");
            digest.update(serde_json::to_vec(root)?);
        }
        if let Some(identity) = provider_identity {
            digest.update(b"\0typed-router-provider-observation\0");
            digest.update(identity);
        }
        let digest = super::identity::encode_digest(digest.finalize());
        Ok((
            Self {
                transform_options,
                config_identity: cstr!("vize-content-mapper-v1:{digest}"),
                watched_files,
                project_context,
            },
            typed_router_root,
        ))
    }
}

impl ContentMapperTransformOptions {
    #[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
    pub(super) fn with_vue_compiler_options(
        mut self,
        config_file_name: &Path,
        options: &Map<std::string::String, Value>,
        dependencies: &mut Vec<std::path::PathBuf>,
    ) -> CorsaResult<(Self, Option<std::path::PathBuf>)> {
        super::vue_check_options::resolve(&mut self.checks, Some(options));
        self.configured_global_template_ref =
            super::tsconfig_gen::vue_compiler_options::configures_global_template_ref(options);
        let (target, manifest) =
            super::tsconfig_gen::vue_compiler_options::vue_target_from_options_with_dependencies(
                config_file_name.parent().unwrap_or(Path::new(".")),
                config_file_name,
                options,
                dependencies,
            )?;
        self.checks.vue_compiler_target = target;
        Ok((self, manifest))
    }

    pub(super) const fn virtual_ts_check_options(self) -> crate::virtual_ts::VirtualTsCheckOptions {
        crate::virtual_ts::VirtualTsCheckOptions {
            unchecked_prop_values: true,
            ..self.checks
        }
    }
}

fn resolved_target_tag(target: Option<VueVersion>) -> &'static str {
    match target {
        Some(VueVersion::V2) => "2",
        Some(VueVersion::V2_7) => "2.7",
        Some(VueVersion::V3) => "3",
        Some(_) | None => "",
    }
}

#[cfg(test)]
mod strict_v_model_tests;
#[cfg(test)]
pub(super) mod typed_router_tests;
#[cfg(test)]
mod vue_options_projection_tests;
