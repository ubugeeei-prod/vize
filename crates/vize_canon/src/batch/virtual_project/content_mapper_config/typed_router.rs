//! Selected router context and provider observations for an opened mapper project.

use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use super::super::tsconfig_gen::compiler_inputs::CompilerInputs;
use super::super::typed_router::{Config, provider};
use crate::batch::error::CorsaResult;

/// Project-owned router context. A filesystem observation does not establish
/// which provider bytes the native project owns, so it never grants route props.
#[derive(Clone, Debug, Default)]
pub struct ContentMapperProjectContext {
    root: Option<PathBuf>,
}

impl ContentMapperProjectContext {
    pub(in crate::batch::virtual_project) fn config(&self) -> Option<Config<'_>> {
        self.root.as_deref().map(Config::files_only)
    }

    #[expect(
        clippy::disallowed_types,
        reason = "host compiler options use serde_json keys"
    )]
    pub(super) fn capture(
        root: Option<PathBuf>,
        config_file_name: &Path,
        compiler: Option<&Map<std::string::String, Value>>,
        watched: &mut Vec<vize_l0::String>,
    ) -> CorsaResult<(Self, Option<[u8; 32]>)> {
        let context = Self { root };
        let Some(compiler) = compiler.filter(|_| context.root.is_some()) else {
            return Ok((context, None));
        };
        // Older callers supply only a root. Preserve their file-only authority
        // without inventing the host's module-resolution defaults.
        if !provider::supports_resolution_context(compiler) {
            return Ok((context, None));
        }
        let inputs = CompilerInputs::default();
        let identity = provider::capture_observation(
            config_file_name.parent().unwrap_or(Path::new(".")),
            compiler,
            &inputs,
        )?;
        inputs.validate()?;
        watched.extend(
            inputs
                .paths()
                .into_iter()
                .map(|path| path.to_string_lossy().into()),
        );
        Ok((context, Some(identity)))
    }
}
