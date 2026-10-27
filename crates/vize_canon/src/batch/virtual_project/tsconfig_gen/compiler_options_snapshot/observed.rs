//! Effective compiler options plus the exact inputs consumed while loading them.

use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use super::super::super::tsconfig_paths::normalize_path_lexically;
use super::super::compiler_inputs::{CompilerInputs, ConfigurationObservations};
use super::super::compiler_options::{CompilerOptionsContext, UnresolvedExtends};
use crate::batch::error::{CorsaError, CorsaResult};

/// Immutable options for a host-selected TypeScript configuration.
///
/// This does not discover the governing configuration, resolve modules, or
/// prove a runtime provider. Hosts retain their ordered configuration search,
/// import occurrence/mode/conditions, source revision, and module-resolution
/// authority alongside it. The options preserve ordered arrays such as paths
/// targets, moduleSuffixes, rootDirs, and customConditions.
///
/// There is no VirtualProject, generated cache directory, or cache usage lock.
#[derive(Clone)]
#[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
pub struct CompilerOptionsSnapshot {
    project_root: PathBuf,
    tsconfig_path: Option<PathBuf>,
    options: Map<std::string::String, Value>,
    unresolved_extends: Vec<UnresolvedExtends>,
    observations: ConfigurationObservations,
}

impl CompilerOptionsSnapshot {
    /// Capture one selected config through the existing chain loader.
    /// All host-supplied paths must be absolute; process cwd is not authority.
    /// The same lexical normalization as the chain loader fixes their identity.
    /// An existing dangling config link is unreadable, not proven absence.
    ///
    /// The host supplies all higher-priority absent config candidates that
    /// influenced its selection. Their absence is checked even when a config
    /// was selected. With no selected config, at least one absence candidate
    /// is required. This function does not infer host search precedence.
    /// Unreadable inputs, malformed JSON, invalid root/options/extends shapes,
    /// cycles, or concurrently changed inputs return an error;
    /// they never become inferred default options. Unresolved extends remain
    /// explicit so a proof consumer can refuse incomplete configuration.
    pub fn capture(
        project_root: &Path,
        tsconfig_path: Option<&Path>,
        absent_candidates: &[PathBuf],
    ) -> CorsaResult<Self> {
        if !project_root.is_absolute()
            || tsconfig_path.is_some_and(|path| !path.is_absolute())
            || absent_candidates.iter().any(|path| !path.is_absolute())
        {
            return Err(CorsaError::Configuration(vize_carton::cstr!(
                "compiler option capture requires absolute host paths"
            )));
        }
        let project_root = vize_carton::path::canonicalize_non_verbatim(project_root);
        let inputs = CompilerInputs::default();
        if tsconfig_path.is_none() && absent_candidates.is_empty() {
            return Err(CorsaError::Configuration(vize_carton::cstr!(
                "configuration absence requires the host's consulted candidates"
            )));
        }
        let tsconfig_path = tsconfig_path.map(normalize_path_lexically);
        for path in absent_candidates
            .iter()
            .map(|path| normalize_path_lexically(path))
        {
            inputs.observe(&path);
            match std::fs::symlink_metadata(&path) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Ok(_) => {
                    return Err(CorsaError::Configuration(vize_carton::cstr!(
                        "configuration {} exists where the host selected absence",
                        path.display()
                    )));
                }
                Err(error) => {
                    return Err(CorsaError::Configuration(vize_carton::cstr!(
                        "cannot observe configuration {}: {error}",
                        path.display()
                    )));
                }
            }
        }
        if let Some(path) = tsconfig_path.as_deref() {
            inputs.observe(path);
        }
        let flattened = CompilerOptionsContext::observed(&project_root)
            .load_flattened(tsconfig_path.as_deref(), Some(&inputs))?;
        let unresolved_extends = flattened.unresolved_extends.clone();
        let options = super::absolute_options(&project_root, flattened);
        inputs.validate()?;
        Ok(Self {
            project_root,
            tsconfig_path,
            options,
            unresolved_extends,
            observations: inputs.observations(),
        })
    }

    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    /// The host-selected path; None means observed absence, not a load failure.
    pub fn tsconfig_path(&self) -> Option<&Path> {
        self.tsconfig_path.as_deref()
    }

    /// Full effective options, with all path-bearing values made absolute.
    #[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
    pub fn options(&self) -> &Map<std::string::String, Value> {
        &self.options
    }

    /// A consumer needing complete authority must refuse a nonempty result.
    pub fn unresolved_extends(&self) -> &[UnresolvedExtends] {
        &self.unresolved_extends
    }

    /// Original consumed/consulted identities, including absent candidates.
    pub fn observations(&self) -> &ConfigurationObservations {
        &self.observations
    }
}

#[cfg(test)]
mod tests;
