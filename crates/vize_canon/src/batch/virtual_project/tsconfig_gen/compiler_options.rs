//! Canon adapters for the shared selected-project TypeScript option loader.
use super::super::VirtualProject;
use super::compiler_inputs::{CompilerInputs, read_jsonc};
use crate::batch::error::{CorsaError, CorsaResult};
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use vize_carton::typescript_project as shared;

pub(in super::super) use shared::FlattenedCompilerOptions;
pub use shared::UnresolvedExtends;

impl VirtualProject {
    pub(crate) fn source_file_policy_for_config(
        project_root: &Path,
        config: &Path,
    ) -> CorsaResult<crate::batch::source_policy::SourceFilePolicy> {
        let options = CompilerOptionsContext::new(project_root).load_verbatim(config, None)?;
        Ok(crate::batch::source_policy::SourceFilePolicy::from_compiler_options(&options))
    }

    #[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
    pub(in super::super) fn load_compiler_options(
        &self,
        tsconfig_path: Option<&Path>,
    ) -> CorsaResult<Map<std::string::String, Value>> {
        Ok(self.load_compiler_options_flattened(tsconfig_path)?.options)
    }

    /// Flatten the effective options without changing path spellings. The
    /// input-less option probe validates syntax only, so its paths stay inert.
    #[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
    pub(in super::super) fn load_compiler_options_verbatim(
        &self,
        tsconfig_path: Option<&Path>,
    ) -> CorsaResult<Map<std::string::String, Value>> {
        self.load_compiler_options_verbatim_with_inputs(tsconfig_path, None)
    }

    #[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
    pub(in super::super) fn load_compiler_options_verbatim_with_inputs(
        &self,
        tsconfig_path: Option<&Path>,
        inputs: Option<&CompilerInputs>,
    ) -> CorsaResult<Map<std::string::String, Value>> {
        let Some(tsconfig_path) = tsconfig_path else {
            return Ok(Map::new());
        };
        if let Some(snapshot) = self.compiler_configuration.as_ref()
            && snapshot.tsconfig_path() == Some(tsconfig_path)
        {
            return Ok(snapshot.verbatim.clone());
        }
        CompilerOptionsContext::new(&self.project_root).load_verbatim(tsconfig_path, inputs)
    }

    /// Flatten the chain and rebase `paths` targets onto the project root from
    /// the anchor TypeScript actually uses: the effective `baseUrl` when one is
    /// declared, the winning `paths` map's declaring directory otherwise.
    pub(in super::super) fn load_compiler_options_flattened(
        &self,
        tsconfig_path: Option<&Path>,
    ) -> CorsaResult<FlattenedCompilerOptions> {
        self.load_compiler_options_flattened_with_inputs(tsconfig_path, None)
    }

    pub(in super::super) fn load_compiler_options_flattened_with_inputs(
        &self,
        tsconfig_path: Option<&Path>,
        inputs: Option<&CompilerInputs>,
    ) -> CorsaResult<FlattenedCompilerOptions> {
        if let Some(snapshot) = self.compiler_configuration.as_ref()
            && tsconfig_path == snapshot.tsconfig_path()
        {
            return Ok(snapshot.flattened.clone());
        }
        CompilerOptionsContext::new(&self.project_root).load_flattened(tsconfig_path, inputs)
    }

    pub(in super::super) fn checked_compiler_config_paths(
        &self,
        tsconfig_path: &Path,
    ) -> CorsaResult<Vec<PathBuf>> {
        Ok(self
            .load_compiler_options_flattened(Some(tsconfig_path))?
            .input_paths)
    }
}

/// Preserve Canon input observations and error spelling while sharing parsing.
pub(in super::super) struct CanonConfigHost<'a> {
    pub inputs: Option<&'a CompilerInputs>,
}
impl shared::CompilerConfigHost for CanonConfigHost<'_> {
    type Error = CorsaError;
    fn read_jsonc(&self, path: &Path) -> CorsaResult<Value> {
        read_jsonc(path, self.inputs)
    }
    fn read_jsonc_observed(&self, path: &Path) -> CorsaResult<Value> {
        // CompilerInputs already binds the observed identity to consumed bytes.
        // Reusing it avoids a second ancestor-link scan for every config read.
        read_jsonc(path, self.inputs)
    }
    fn configuration_error(&self, message: vize_carton::String) -> CorsaError {
        CorsaError::Configuration(message)
    }
    fn observe(&self, path: &Path) {
        if let Some(inputs) = self.inputs {
            inputs.observe(path);
        }
    }
    fn resolve_extends(&self, path: &Path, specifier: &str) -> Option<std::path::PathBuf> {
        if let Some(inputs) = self.inputs {
            return inputs.resolve("extends", path, specifier, || {
                shared::resolve_extended_tsconfig_path(path, specifier, self)
            });
        }
        shared::resolve_extended_tsconfig_path(path, specifier, self)
    }
}

/// The Canon facade retains selected-root and strict-admission policy.
pub(super) struct CompilerOptionsContext<'a> {
    inner: shared::CompilerOptionsContext<'a>,
}
impl<'a> CompilerOptionsContext<'a> {
    pub(super) fn new(root: &'a Path) -> Self {
        Self {
            inner: shared::CompilerOptionsContext::new(root),
        }
    }
    pub(super) fn observed(root: &'a Path) -> Self {
        Self {
            inner: shared::CompilerOptionsContext::observed(root),
        }
    }
    #[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
    fn load_verbatim(
        &self,
        config: &Path,
        inputs: Option<&CompilerInputs>,
    ) -> CorsaResult<Map<std::string::String, Value>> {
        self.inner
            .load_verbatim(config, &CanonConfigHost { inputs })
    }
    pub(super) fn load_flattened(
        &self,
        config: Option<&Path>,
        inputs: Option<&CompilerInputs>,
    ) -> CorsaResult<FlattenedCompilerOptions> {
        self.inner
            .load_flattened(config, &CanonConfigHost { inputs })
    }
}
